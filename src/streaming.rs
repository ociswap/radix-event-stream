use std::fmt::Debug;

use crate::handler::HandlerRegistry;
use log::{info, warn};

/// A trait that abstracts an event coming from any source,
/// like a gateway, database, or file.
pub trait Event: Debug {
    fn name(&self) -> &str;
    fn binary_sbor_data(&self) -> Vec<u8>;
    fn emitter(&self)
        -> &radix_client::gateway::models::EventEmitterIdentifier;
}

/// A trait that abstracts a transaction coming from any source,
/// like a gateway, database, or file.
pub trait Transaction: Debug {
    fn intent_hash(&self) -> String;
    fn state_version(&self) -> u64;
    fn events(&self) -> Vec<Box<dyn Event>>;
}

/// A trait that abstracts a stream of transactions coming
/// from any source, like a gateway, database, or file.
pub trait TransactionStream: Debug {
    fn next(
        &mut self,
    ) -> Result<Vec<Box<dyn Transaction>>, TransactionStreamError>;
}

#[derive(Debug)]
pub enum TransactionStreamError {
    NoMoreTransactions,
    Finished,
    Error(String),
}

/// Uses a `TransactionStream` to process transactions and
/// events using a `HandlerRegistry`. Register event handlers
/// using the `HandlerRegistry` and then call `run` to start
/// processing transactions.
pub struct TransactionStreamProcessor<T>
where
    T: TransactionStream,
{
    pub transaction_stream: T,
    pub handler_registry: HandlerRegistry,
    pub time_last_state_version_reported: Option<std::time::Instant>,
}

impl<T> TransactionStreamProcessor<T>
where
    T: TransactionStream,
{
    pub fn new(
        transaction_stream: T,
        handler_registry: HandlerRegistry,
    ) -> Self {
        TransactionStreamProcessor {
            transaction_stream,
            handler_registry,
            time_last_state_version_reported: None,
        }
    }

    pub fn run(&mut self) {
        loop {
            let transactions = match self.transaction_stream.next() {
                Err(error) => match error {
                    TransactionStreamError::NoMoreTransactions => {
                        info!("No more transactions, sleeping for 1 second...");
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        continue;
                    }
                    TransactionStreamError::Finished => {
                        info!("Finished processing transactions.");
                        break;
                    }
                    TransactionStreamError::Error(error) => {
                        warn!("Error while getting transactions: {}", error);
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        continue;
                    }
                },
                Ok(transactions) => transactions,
            };

            transactions.iter().for_each(|transaction| {
                match self.time_last_state_version_reported {
                    Some(last_reported) => {
                        if last_reported.elapsed().as_secs() > 1 {
                            info!(
                                "State version: {}",
                                transaction.state_version()
                            );
                            self.time_last_state_version_reported =
                                Some(std::time::Instant::now());
                        }
                    }
                    None => {
                        info!("State version: {}", transaction.state_version());
                        self.time_last_state_version_reported =
                            Some(std::time::Instant::now());
                    }
                };
                let events = transaction.events();
                events.iter().for_each(|event| {
                    self.handler_registry.handle(transaction, event).unwrap();
                })
            });
        }
    }

    pub fn run_with(transaction_stream: T, handler_registry: HandlerRegistry) {
        let mut processor = TransactionStreamProcessor::new(
            transaction_stream,
            handler_registry,
        );
        processor.run();
    }
}
