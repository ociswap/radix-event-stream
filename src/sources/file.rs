use std::{fs::File, path::Path};

use async_trait::async_trait;
use serde::Deserialize;
use tokio::sync::mpsc::Receiver;

use crate::{models::Transaction, stream::TransactionStream};

#[derive(Debug, Deserialize, Clone)]
pub struct FileTransaction {
    pub intent_hash: String,
    pub state_version: u64,
    pub unix_timestamp_nanos: i64,
    pub events: Vec<radix_client::gateway::models::Event>,
}

impl Into<Transaction> for FileTransaction {
    fn into(self) -> Transaction {
        Transaction {
            intent_hash: self.intent_hash,
            state_version: self.state_version,
            confirmed_at: Some(chrono::DateTime::from_timestamp_nanos(
                self.unix_timestamp_nanos,
            )),
            events: self.events.into_iter().map(|event| event.into()).collect(),
        }
    }
}

#[derive(Debug)]
pub struct FileTransactionStream {
    transactions: Vec<FileTransaction>,
}

impl FileTransactionStream {
    pub fn new(file_path: String) -> Self {
        let file = File::open(&file_path).expect("Unable to open file");

        // Determine file extension
        let extension = Path::new(&file_path)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("");

        let transactions: Vec<FileTransaction> = match extension {
            "json" => serde_json::from_reader(&file)
                .expect("Error deserializing JSON"),
            "yaml" | "yml" => serde_yaml::from_reader(&file)
                .expect("Error deserializing YAML"),
            _ => panic!("Unsupported file type"),
        };

        FileTransactionStream { transactions }
    }
}

#[async_trait]
impl TransactionStream for FileTransactionStream {
    async fn start(&mut self) -> Result<Receiver<Transaction>, anyhow::Error> {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let mut transactions = self.transactions.clone();
        tokio::spawn(async move {
            for transaction in transactions.drain(..) {
                if tx.send(transaction.into()).await.is_err() {
                    break;
                }
            }
        });
        Ok(rx)
    }
    // no task is spawned, so no need to do anything on stop
    async fn stop(&mut self) {}
}
