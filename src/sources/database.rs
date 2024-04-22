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

const DEFAULT_CAUGHT_UP_TIMEOUT_MS: u64 = 500;
const DEFAULT_QUERY_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_STATE_VERSION: u64 = 1;
const DEFAULT_PAGE_SIZE: u32 = 100_000;
const DEFAULT_BUFFER_CAPACITY: u64 = 1_000_000;

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
    handle: Option<tokio::task::JoinHandle<()>>,
    limit_per_page: u32,
    buffer_capacity: u64,
    caught_up_timeout_ms: u64,
    query_timeout_ms: u64,
    database_url: String,
}

impl DatabaseTransactionStream {
    pub fn new(database_url: String) -> Self {
        Self {
            state_version: DEFAULT_STATE_VERSION,
            limit_per_page: DEFAULT_PAGE_SIZE,
            handle: None,
            buffer_capacity: DEFAULT_BUFFER_CAPACITY,
            caught_up_timeout_ms: DEFAULT_CAUGHT_UP_TIMEOUT_MS,
            query_timeout_ms: DEFAULT_QUERY_TIMEOUT_MS,
            database_url,
        }
    }

    pub fn from_state_version(mut self, state_version: u64) -> Self {
        self.state_version = state_version;
        self
    }

    pub fn limit_per_page(mut self, limit_per_page: u32) -> Self {
        self.limit_per_page = limit_per_page;
        self
    }

    pub fn buffer_capacity(mut self, capacity: u64) -> Self {
        self.buffer_capacity = capacity;
        self
    }

    pub fn caught_up_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.caught_up_timeout_ms = timeout_ms;
        self
    }

    pub fn query_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.query_timeout_ms = timeout_ms;
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
    caught_up_timeout_ms: u64,
    query_timeout_ms: u64,
    tx: tokio::sync::mpsc::Sender<Transaction>,
}

impl DatabaseFetcher {
    async fn new(
        database_url: String,
        limit_per_page: u32,
        state_version: u64,
        caught_up_timeout_ms: u64,
        query_timeout_ms: u64,
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
            caught_up_timeout_ms,
            query_timeout_ms,
            tx,
        })
    }

    /// Fetches the next batch of transactions from the database.
    async fn next_batch(&mut self) -> Result<Vec<Transaction>, anyhow::Error> {
        let transactions: Vec<TransactionRecord> = timeout(Duration::from_millis(self.query_timeout_ms), sqlx::query_as!(
            TransactionRecord,
            r#"
                select
                    state_version,
                    round_timestamp,
                    receipt_event_emitters,
                    receipt_event_sbors,
                    receipt_event_names,
                    intent_hash
                from
                    ledger_transactions
                where discriminator = 'user' and receipt_status != 'failed' and state_version >= $2
                order by state_version asc
                limit
                $1
            "#,
            self.limit_per_page as i32,
            self.state_version as i64
        )
        .fetch_all(&self.connection)).await??;

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
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    self.caught_up_timeout_ms,
                ))
                .await;
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
            self.caught_up_timeout_ms,
            self.query_timeout_ms,
            tx,
        )
        .await?;
        let handle = tokio::spawn(async move { fetcher.run().await });
        self.handle = Some(handle);
        Ok(rx)
    }

    async fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
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
