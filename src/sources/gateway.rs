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

const PUBLIC_MAINNET_GATEWAY_URL: &str = "https://mainnet.radixdlt.com";
const DEFAULT_STATE_VERSION: u64 = 1;
const DEFAULT_PAGE_SIZE: u32 = 100;
const DEFAULT_BUFFER_CAPACITY: u64 = 10000;

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
            binary_sbor_data: programmatic_json_to_bytes(&self.data).expect(
                "Should always able to convert Programmatic JSON to binary SBOR",
            ),
        }
    }
}

impl Into<Transaction> for CommittedTransactionInfo {
    fn into(self) -> Transaction {
        Transaction {
            intent_hash: self
                .intent_hash
                .expect("Transaction should have tx id"),
            state_version: self.state_version,
            confirmed_at: self.confirmed_at,
            events: self
                .receipt
                .expect("Transaction should have receipt")
                .events
                .expect("Transaction receipt should have events")
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
    buffer_capacity: u64,
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl GatewayTransactionStream {
    pub fn new() -> Self {
        GatewayTransactionStream {
            gateway_url: PUBLIC_MAINNET_GATEWAY_URL.to_string(),
            from_state_version: DEFAULT_STATE_VERSION,
            limit_per_page: DEFAULT_PAGE_SIZE,
            buffer_capacity: DEFAULT_BUFFER_CAPACITY,
            handle: None,
        }
    }

    pub fn from_state_version(mut self, from_state_version: u64) -> Self {
        self.from_state_version = from_state_version;
        self
    }

    pub fn gateway_url(mut self, gateway_url: String) -> Self {
        self.gateway_url = gateway_url;
        self
    }

    pub fn limit_per_page(mut self, limit_per_page: u32) -> Self {
        self.limit_per_page = limit_per_page;
        self
    }

    pub fn buffer_capacity(mut self, buffer_capacity: u64) -> Self {
        self.buffer_capacity = buffer_capacity;
        self
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
        let (tx, rx) =
            tokio::sync::mpsc::channel(self.buffer_capacity as usize);
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
