use std::str::FromStr;

use crate::{
    models::{Event, EventEmitter, Transaction},
    stream::TransactionStream,
};

use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;
use sqlx::{postgres::PgConnectOptions, ConnectOptions};
use tokio::sync::mpsc::Receiver;

const CAUGHT_UP_TIMEOUT_MS: u64 = 500;

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
    capacity: u64,
    database_url: String,
}

impl DatabaseTransactionStream {
    pub async fn new(
        database_url: String,
        from_state_version: u64,
        limit_per_page: u32,
        capacity: u64,
    ) -> Self {
        DatabaseTransactionStream {
            state_version: from_state_version,
            limit_per_page,
            handle: None,
            capacity,
            database_url,
        }
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
    tx: tokio::sync::mpsc::Sender<Transaction>,
}

impl DatabaseFetcher {
    async fn new(
        database_url: String,
        limit_per_page: u32,
        state_version: u64,
        tx: tokio::sync::mpsc::Sender<Transaction>,
    ) -> Result<Self, anyhow::Error> {
        let options = PgConnectOptions::from_str(&database_url)
            .expect("Failed to parse database URL")
            .disable_statement_logging();
        let connection = sqlx::postgres::PgPool::connect_with(options).await?;
        Ok(DatabaseFetcher {
            connection,
            limit_per_page,
            state_version,
            tx: tx,
        })
    }

    /// Fetches the next batch of transactions from the database.
    async fn next_batch(&mut self) -> Result<Vec<Transaction>, anyhow::Error> {
        let transactions = sqlx::query_as!(
            TransactionRecord,
            r#"
                select
                state_version,
                round_timestamp,
                receipt_event_emitters,
                receipt_event_sbors,
                receipt_event_names,
                transaction_tree_hash
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
        .fetch_all(&self.connection)
        .await?;

        let transactions: Vec<_> = transactions
            .into_iter()
            .map(|db_transaction| {
                let events = db_transaction
                    .receipt_event_emitters
                    .iter()
                    .zip(db_transaction.receipt_event_sbors.iter())
                    .zip(db_transaction.receipt_event_names.iter())
                    .map(|((emitter, sbor), name)| Event {
                        name: name.clone(),
                        binary_sbor_data: sbor.clone(),
                        emitter:
                            serde_json::from_value::<EventEmitterIdentifier>(
                                emitter.clone(),
                            )
                            .unwrap()
                            .into(),
                    })
                    .collect();
                Transaction {
                    state_version: db_transaction.state_version as u64,
                    intent_hash: db_transaction.transaction_tree_hash,
                    confirmed_at: Some(db_transaction.round_timestamp),
                    events: events,
                }
            })
            .collect();

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
                log::error!("Error fetching transactions: {:?}", err);
                response = self.next_batch().await;
            }
            let transactions = response.unwrap();
            if transactions.is_empty() {
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    CAUGHT_UP_TIMEOUT_MS,
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
        let (tx, rx) = tokio::sync::mpsc::channel(self.capacity as usize);
        let mut fetcher = DatabaseFetcher::new(
            self.database_url.clone(),
            self.limit_per_page,
            self.state_version,
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
    transaction_tree_hash: String,
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

impl From<EventEmitterIdentifier> for EventEmitter {
    fn from(identifier: EventEmitterIdentifier) -> Self {
        match identifier {
            EventEmitterIdentifier::Method { entity } => EventEmitter::Method {
                entity_address: entity.entity_address,
            },
            EventEmitterIdentifier::Function {
                package_address,
                blueprint_name,
            } => EventEmitter::Function {
                package_address,
                blueprint_name,
            },
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct EntityReference {
    pub entity_type: EntityType,
    pub is_global: bool,
    pub entity_address: String,
}

#[derive(Deserialize, Debug, Clone)]
pub enum EntityType {
    GlobalPackage,
    GlobalConsensusManager,
    GlobalValidator,
    GlobalGenericComponent,
    GlobalAccount,
    GlobalIdentity,
    GlobalAccessController,
    GlobalVirtualSecp256k1Account,
    GlobalVirtualSecp256k1Identity,
    GlobalVirtualEd25519Account,
    GlobalVirtualEd25519Identity,
    GlobalFungibleResource,
    InternalFungibleVault,
    GlobalNonFungibleResource,
    InternalNonFungibleVault,
    InternalGenericComponent,
    InternalKeyValueStore,
    GlobalOneResourcePool,
    GlobalTwoResourcePool,
    GlobalMultiResourcePool,
    GlobalTransactionTracker,
}
