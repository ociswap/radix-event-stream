use crate::{
    encodings::programmatic_json_to_bytes,
    models::{Event, EventEmitter, Transaction},
    stream::TransactionStream,
};

use async_trait::async_trait;
use radix_client::{
    gateway::{
        models::{CommittedTransactionInfo, EventEmitterIdentifier},
        stream::stream_client::TransactionStreamAsync,
    },
    GatewayClientAsync,
};
use tokio::sync::mpsc::{Receiver, Sender};

const CAUGHT_UP_TIMEOUT_MS: u64 = 500;

impl Into<Event> for radix_client::gateway::models::Event {
    fn into(self) -> Event {
        let emitter = match self.emitter {
            EventEmitterIdentifier::Method { entity, .. } => {
                EventEmitter::Method {
                    entity_address: entity.entity_address,
                }
            }
            EventEmitterIdentifier::Function {
                package_address,
                blueprint_name,
            } => EventEmitter::Function {
                package_address,
                blueprint_name,
            },
        };
        Event {
            name: self.name,
            emitter,
            binary_sbor_data: programmatic_json_to_bytes(&self.data).unwrap(),
        }
    }
}

impl Into<Transaction> for CommittedTransactionInfo {
    fn into(self) -> Transaction {
        Transaction {
            intent_hash: self.intent_hash.unwrap(),
            state_version: self.state_version,
            confirmed_at: self.confirmed_at,
            events: self
                .receipt
                .unwrap()
                .events
                .unwrap()
                .into_iter()
                .map(|event| event.into())
                .collect(),
        }
    }
}
#[derive(Debug)]
pub struct GatewayTransactionStream {
    gateway_url: String,
    from_state_version: u64,
    limit_per_page: u32,
    capacity: u64,
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl GatewayTransactionStream {
    pub fn new(
        from_state_version: u64,
        gateway_url: String,
        limit_per_page: u32,
        capacity: u64,
    ) -> Self {
        GatewayTransactionStream {
            gateway_url,
            from_state_version,
            limit_per_page,
            capacity,
            handle: None,
        }
    }
}

struct GatewayFetcher {
    stream: TransactionStreamAsync,
    tx: Sender<Transaction>,
}

impl GatewayFetcher {
    pub fn new(
        gateway_url: String,
        from_state_version: u64,
        limit_per_page: u32,
        tx: Sender<Transaction>,
    ) -> Self {
        let client = GatewayClientAsync::new(gateway_url);
        let stream = TransactionStreamAsync::new(
            &client,
            from_state_version,
            limit_per_page,
        );
        GatewayFetcher { stream, tx }
    }

    async fn run(&mut self) {
        loop {
            let mut response = self.stream.next().await;
            while let Err(err) = response {
                log::error!("Error fetching transactions: {:?}", err);
                response = self.stream.next().await;
            }
            let response = response.unwrap();
            if response.items.is_empty() {
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    CAUGHT_UP_TIMEOUT_MS,
                ))
                .await;
            }
            let transactions: Vec<Transaction> =
                response.items.into_iter().map(|item| item.into()).collect();
            for transaction in transactions {
                // Stop fetching if the receiving end is closed
                if self.tx.send(transaction).await.is_err() {
                    return;
                }
            }
        }
    }
}

#[async_trait]
impl TransactionStream for GatewayTransactionStream {
    async fn start(&mut self) -> Result<Receiver<Transaction>, anyhow::Error> {
        let (tx, rx) = tokio::sync::mpsc::channel(self.capacity as usize);
        let mut fetcher = GatewayFetcher::new(
            self.gateway_url.clone(),
            self.from_state_version,
            self.limit_per_page,
            tx,
        );
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
