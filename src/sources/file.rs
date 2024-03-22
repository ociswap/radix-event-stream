use std::{fs::File, path::Path};

use crate::streaming::{Event, Transaction, TransactionStream};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct FileTransaction {
    pub intent_hash: String,
    pub state_version: u64,
    pub unix_timestamp_nanos: u64,
    pub events: Vec<radix_client::gateway::models::Event>,
}

impl Transaction for FileTransaction {
    fn intent_hash(&self) -> String {
        self.intent_hash.clone()
    }
    fn state_version(&self) -> u64 {
        self.state_version
    }
    fn confirmed_at(&self) -> Option<chrono::DateTime<chrono::prelude::Utc>> {
        Some(chrono::DateTime::from_timestamp_nanos(
            self.unix_timestamp_nanos as i64,
        ))
    }
    fn events(&self) -> Vec<Box<dyn Event>> {
        self.events
            .iter()
            .map(|event| Box::new(event.clone()) as Box<dyn Event>)
            .collect()
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

impl TransactionStream for FileTransactionStream {
    fn next(
        &mut self,
    ) -> Result<
        Vec<Box<dyn Transaction>>,
        crate::streaming::TransactionStreamError,
    > {
        if self.transactions.is_empty() {
            return Err(crate::streaming::TransactionStreamError::Finished);
        }

        let transactions = self.transactions.clone();
        self.transactions.clear();
        let transactions: Vec<Box<dyn Transaction>> = transactions
            .into_iter()
            .map(|transaction| Box::new(transaction) as Box<dyn Transaction>)
            .collect();
        Ok(transactions)
    }
}
