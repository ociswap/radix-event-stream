//! A transaction stream that fetches transactions from a Radix Gateway PostgreSQL database.

use crate::{
    models::{Event, EventEmitter, Transaction},
    stream::TransactionStream,
};
use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;
use sqlx::{postgres::PgConnectOptions, ConnectOptions};
use std::{str::FromStr, time::Duration};
use tokio::{sync::mpsc::Receiver, time::timeout};

/// A transaction stream that fetches transactions directly from
/// the PostgreSQL database associated with a Radix Gateway.
/// It's more difficult to get access to a Radix Gateway database
/// compared to the Gateway API itself, as Radix does not provide
/// direct access to the database. However, the database allows you
/// to query transactions with a much higher throughput than the
/// Gateway API.
#[derive(Debug)]
pub struct DatabaseTransactionStream {
    state_version: u64,
    join_handle: Option<tokio::task::JoinHandle<()>>,
    limit_per_page: u32,
    buffer_capacity: u64,
    caught_up_timeout: Duration,
    query_timeout: Duration,
    database_url: String,
}

impl Default for DatabaseTransactionStream {
    fn default() -> Self {
        Self {
            state_version: 1,
            limit_per_page: 100_000,
            join_handle: None,
            buffer_capacity: 1_000_000,
            caught_up_timeout: Duration::from_millis(500),
            query_timeout: Duration::from_secs(30),
            database_url: "".to_string(),
        }
    }
}

impl DatabaseTransactionStream {
    /// Creates a new DatabaseTransactionStream with default settings and the given database URL.
    pub fn new(database_url: String) -> Self {
        Self {
            database_url,
            ..Self::default()
        }
    }

    /// Sets the state version to start fetching transactions from.
    pub fn from_state_version(mut self, state_version: u64) -> Self {
        self.state_version = state_version;
        self
    }

    /// Sets the max number of transactions to fetch per query.
    pub fn limit_per_page(mut self, limit_per_page: u32) -> Self {
        self.limit_per_page = limit_per_page;
        self
    }

    /// Sets the buffer capacity of the channel through which transactions are sent to the transaction processor.
    /// This is the maximum number of transactions that can be buffered before the stream starts to block.
    /// If the stream is producing transactions faster than the transaction processor can consume them,
    /// this buffer will fill up.
    /// You may want to play with this value, based on the performance of the database and the transaction processor.
    pub fn buffer_capacity(mut self, capacity: u64) -> Self {
        self.buffer_capacity = capacity;
        self
    }

    /// Sets the timeout to wait for after each poll of the database when the stream is caught up.
    pub fn caught_up_timeout(mut self, timeout: Duration) -> Self {
        self.caught_up_timeout = timeout;
        self
    }

    /// Sets a duration after which a query will time out and be retried.
    pub fn query_timeout(mut self, timeout: Duration) -> Self {
        self.query_timeout = timeout;
        self
    }
}

/// A helper which is passed to the new task created by the stream.
/// It keeps track of the current state version and fetches transactions
/// from the database in batches. It sends the transactions to the
/// processor through a channel.
struct DatabaseFetcher {
    connection: sqlx::Pool<sqlx::Postgres>,
    limit_per_page: u32,
    state_version: u64,
    caught_up_timeout: Duration,
    query_timeout: Duration,
    tx: tokio::sync::mpsc::Sender<Transaction>,
}

impl DatabaseFetcher {
    async fn new(
        database_url: String,
        limit_per_page: u32,
        state_version: u64,
        caught_up_timeout: Duration,
        query_timeout: Duration,
        tx: tokio::sync::mpsc::Sender<Transaction>,
    ) -> Result<Self, anyhow::Error> {
        let options = PgConnectOptions::from_str(&database_url)
            .map_err(|err| anyhow::anyhow!("Invalid database URL: {}", err))?
            .disable_statement_logging();
        let connection = sqlx::postgres::PgPool::connect_with(options).await?;
        Ok(Self {
            connection,
            limit_per_page,
            state_version,
            caught_up_timeout,
            query_timeout,
            tx,
        })
    }

    /// Fetches the next batch of transactions from the database.
    async fn next_batch(&mut self) -> Result<Vec<Transaction>, anyhow::Error> {
        let query = sqlx::query_as::<_, TransactionRecord>(
            r#"
                SELECT
                    state_version,
                    round_timestamp,
                    receipt_event_emitters,
                    receipt_event_sbors,
                    receipt_event_names,
                    intent_hash
                FROM
                    ledger_transactions
                WHERE
                    discriminator = 'user' AND receipt_status != 'failed' AND state_version >= $2
                ORDER BY
                    state_version ASC
                LIMIT
                    $1
            "#
        )
        .bind(self.limit_per_page as i32)
        .bind(self.state_version as i64);

        let transactions: Vec<TransactionRecord> =
            timeout(self.query_timeout, query.fetch_all(&self.connection))
                .await??;

        // Convert the database records to the Transaction model
        let transactions: Vec<_> = transactions
            .into_iter()
            .map(|db_transaction| {
                let events = db_transaction
                    .receipt_event_emitters
                    .into_iter()
                    .zip(db_transaction.receipt_event_sbors.into_iter())
                    .zip(db_transaction.receipt_event_names.into_iter())
                    .map(|((emitter, sbor), name)| Event {
                        name,
                        binary_sbor_data: sbor,
                        emitter:
                            serde_json::from_value::<EventEmitterIdentifier>(
                                emitter,
                            )
                            .expect("Should be able to decode event emitter")
                            .into(),
                    })
                    .collect();
                Transaction {
                    state_version: db_transaction.state_version as u64,
                    intent_hash: db_transaction.intent_hash.unwrap(),
                    confirmed_at: Some(db_transaction.round_timestamp),
                    events,
                }
            })
            .collect();

        // Update the state version
        self.state_version = transactions
            .last()
            .map(|transaction| transaction.state_version + 1)
            .unwrap_or(self.state_version);

        Ok(transactions)
    }

    async fn run(&mut self) {
        loop {
            let mut response = self.next_batch().await;
            while let Err(err) = response {
                log::warn!(
                    "Error fetching transactions: {:?}\n Trying again...",
                    err
                );
                response = self.next_batch().await;
            }
            let transactions = response.unwrap();
            if transactions.is_empty() {
                tokio::time::sleep(self.caught_up_timeout).await;
            }

            for transaction in transactions {
                if self.tx.send(transaction).await.is_err() {
                    return;
                }
            }
        }
    }
}

#[async_trait]
impl TransactionStream for DatabaseTransactionStream {
    async fn start(&mut self) -> Result<Receiver<Transaction>, anyhow::Error> {
        let (tx, rx) =
            tokio::sync::mpsc::channel(self.buffer_capacity as usize);
        let mut fetcher = DatabaseFetcher::new(
            self.database_url.clone(),
            self.limit_per_page,
            self.state_version,
            self.caught_up_timeout,
            self.query_timeout,
            tx,
        )
        .await?;
        let handle = tokio::spawn(async move { fetcher.run().await });
        self.join_handle = Some(handle);
        Ok(rx)
    }

    async fn stop(&mut self) {
        if let Some(handle) = self.join_handle.take() {
            handle.abort();
        }
    }
}

#[derive(sqlx::FromRow, Debug)] // Ensure this derive to work with sqlx queries
struct TransactionRecord {
    state_version: i64,
    round_timestamp: chrono::DateTime<Utc>,
    receipt_event_emitters: Vec<serde_json::Value>,
    receipt_event_sbors: Vec<Vec<u8>>,
    receipt_event_names: Vec<String>,
    intent_hash: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum EventEmitterIdentifier {
    Method {
        entity: EntityReference,
    },
    Function {
        package_address: String,
        blueprint_name: String,
    },
}

#[derive(Deserialize, Debug, Clone)]
pub struct EntityReference {
    pub entity_address: String,
}

impl From<EventEmitterIdentifier> for EventEmitter {
    fn from(identifier: EventEmitterIdentifier) -> Self {
        match identifier {
            EventEmitterIdentifier::Method { entity } => Self::Method {
                entity_address: entity.entity_address,
            },
            EventEmitterIdentifier::Function {
                package_address,
                blueprint_name,
            } => Self::Function {
                package_address,
                blueprint_name,
            },
        }
    }
}
