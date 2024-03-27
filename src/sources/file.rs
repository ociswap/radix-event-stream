// use std::{fs::File, path::Path};

// use serde::Deserialize;

// use crate::{
//     models::IncomingTransaction,
//     stream::{TransactionStream, TransactionStreamError},
// };

// #[derive(Debug, Deserialize, Clone)]
// pub struct FileTransaction {
//     pub intent_hash: String,
//     pub state_version: u64,
//     pub unix_timestamp_nanos: i64,
//     pub events: Vec<radix_client::gateway::models::Event>,
// }

// impl Into<IncomingTransaction> for FileTransaction {
//     fn into(self) -> IncomingTransaction {
//         IncomingTransaction {
//             intent_hash: self.intent_hash,
//             state_version: self.state_version,
//             confirmed_at: Some(chrono::DateTime::from_timestamp_nanos(
//                 self.unix_timestamp_nanos,
//             )),
//             events: self.events.into_iter().map(|event| event.into()).collect(),
//         }
//     }
// }

// #[derive(Debug)]
// pub struct FileTransactionStream {
//     transactions: Vec<FileTransaction>,
// }

// impl FileTransactionStream {
//     pub fn new(file_path: String) -> Self {
//         let file = File::open(&file_path).expect("Unable to open file");

//         // Determine file extension
//         let extension = Path::new(&file_path)
//             .extension()
//             .and_then(std::ffi::OsStr::to_str)
//             .unwrap_or("");

//         let transactions: Vec<FileTransaction> = match extension {
//             "json" => serde_json::from_reader(&file)
//                 .expect("Error deserializing JSON"),
//             "yaml" | "yml" => serde_yaml::from_reader(&file)
//                 .expect("Error deserializing YAML"),
//             _ => panic!("Unsupported file type"),
//         };

//         FileTransactionStream { transactions }
//     }
// }

// impl TransactionStream for FileTransactionStream {
//     fn next(
//         &mut self,
//     ) -> Result<Vec<IncomingTransaction>, TransactionStreamError> {
//         if self.transactions.is_empty() {
//             return Err(TransactionStreamError::Finished);
//         }

//         let transactions = self.transactions.clone();
//         self.transactions.clear();
//         let transactions: Vec<IncomingTransaction> = transactions
//             .into_iter()
//             .map(|transaction| transaction.into())
//             .collect();
//         Ok(transactions)
//     }
// }
