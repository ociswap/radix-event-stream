use std::fmt::Debug;

use crate::handler::HandlerRegistry;
use log::{info, warn};

/// A trait that abstracts an event coming from any source,
/// like a gateway, database, or file.
pub trait Event: Debug {
    fn name(&self) -> &str;
    fn programmatic_json(&self) -> &serde_json::Value;
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
    fn next(&mut self) -> Option<Vec<Box<dyn Transaction>>>;
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
    pub last_reported: std::time::Instant,
}

impl<T> TransactionStreamProcessor<T>
where
    T: TransactionStream,
{
    pub fn new(event_stream: T, handler_registry: HandlerRegistry) -> Self {
        TransactionStreamProcessor {
            transaction_stream: event_stream,
            handler_registry,
            last_reported: std::time::Instant::now(),
        }
    }

    pub fn run(&mut self) {
        loop {
            let transactions = match self.transaction_stream.next() {
                Some(transactions) => {
                    if transactions.is_empty() {
                        info!("No more transactions, sleeping for 1 second...");
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        continue;
                    }
                    transactions
                }
                // If we get None, we should try again,
                None => {
                    warn!("Error while getting transactions, trying again in 1 second...");
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    continue;
                }
            };

            transactions.iter().for_each(|transaction| {
                if self.last_reported.elapsed().as_secs() > 1 {
                    info!("State version: {}", transaction.state_version());
                    self.last_reported = std::time::Instant::now();
                }
                let events = transaction.events();
                events.iter().for_each(|event| {
                    self.handler_registry.handle(transaction, event).unwrap();
                })
            });
        }
    }

    pub fn run_with(event_stream: T, handler_registry: HandlerRegistry) {
        let mut processor =
            TransactionStreamProcessor::new(event_stream, handler_registry);
        processor.run();
    }
}
