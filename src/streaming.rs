use std::fmt::Debug;

use crate::handler::{EventHandler, HandlerRegistry};
use chrono::Utc;
use colored::Colorize;
use log::{debug, info, warn};

#[derive(Debug)]
pub struct Event {
    pub name: String,
    pub binary_sbor_data: Vec<u8>,
    pub emitter: EventEmitter,
}

#[derive(Debug, Clone)]
pub enum EventEmitter {
    Method {
        entity_address: String,
    },
    Function {
        package_address: String,
        blueprint_name: String,
    },
}

impl EventEmitter {
    pub fn address(&self) -> &str {
        match self {
            EventEmitter::Method { entity_address } => entity_address,
            EventEmitter::Function {
                package_address, ..
            } => package_address,
        }
    }
}

#[derive(Debug)]
pub struct Transaction {
    pub intent_hash: String,
    pub state_version: u64,
    pub confirmed_at: Option<chrono::DateTime<Utc>>,
    pub events: Vec<Event>,
}

#[allow(non_camel_case_types)]
impl Transaction {
    pub fn handle_events<EVENT_HANDLER, STATE>(
        &self,
        app_state: &mut STATE,
        handler_registry: &mut HandlerRegistry<EVENT_HANDLER, STATE>,
    ) where
        EVENT_HANDLER: EventHandler<EVENT_HANDLER, STATE>,
        STATE: Clone,
    {
        self.events.iter().for_each(|event| {
            let event_handlers_clone = handler_registry.clone();
            let event_handlers = match event_handlers_clone
                .handlers
                .get(event.emitter.address())
            {
                Some(handlers) => handlers,
                None => return,
            };

            event_handlers.iter().for_each(|handler| {
                if handler.match_variant(&event.name) {
                    info!(
                        "{}",
                        format!("HANDLING EVENT: {:?}", handler)
                            .bright_yellow()
                    );
                    handler.handle(app_state, event, self, handler_registry);
                }
            });
        });
    }
}

#[allow(non_camel_case_types)]
pub struct EventHandlerInput<'a, STATE, EVENT_HANDLER, DECODED_EVENT>
where
    EVENT_HANDLER: EventHandler<EVENT_HANDLER, STATE>,
    STATE: Clone,
{
    pub app_state: &'a mut STATE,
    pub event: &'a DECODED_EVENT,
    pub transaction: &'a Transaction,
    pub handler_registry: &'a mut HandlerRegistry<EVENT_HANDLER, STATE>,
}

/// A trait that abstracts a stream of transactions coming
/// from any source, like a gateway, database, or file.
pub trait TransactionStream: Debug {
    fn next(&mut self) -> Result<Vec<Transaction>, TransactionStreamError>;
}

#[derive(Debug)]
pub enum TransactionStreamError {
    CaughtUp,
    Finished,
    Error(String),
}

/// Uses a `TransactionStream` to process transactions and
/// events using a `HandlerRegistry`. Register event handlers
/// using the `HandlerRegistry` and then call `run` to start
/// processing transactions.
#[allow(non_camel_case_types)]
pub struct TransactionStreamProcessor<
    STREAM,
    EVENT_HANDLER,
    TRANSACTION_HANDLER,
    STATE,
> where
    STREAM: TransactionStream,
    EVENT_HANDLER: EventHandler<EVENT_HANDLER, STATE>,
    STATE: Clone,
    TRANSACTION_HANDLER: FnMut(
        &mut STATE,
        &Transaction,
        &mut HandlerRegistry<EVENT_HANDLER, STATE>,
    ),
{
    pub transaction_stream: STREAM,
    pub handler_registry: HandlerRegistry<EVENT_HANDLER, STATE>,
    pub transaction_handler: TRANSACTION_HANDLER,
    pub state_version_report_interval: Option<std::time::Duration>,
    pub time_last_state_version_reported: std::time::Instant,
    pub app_state: STATE,
}

#[allow(non_camel_case_types)]
impl<STREAM, EVENT_HANDLER, TRANSACTION_HANDLER, STATE>
    TransactionStreamProcessor<
        STREAM,
        EVENT_HANDLER,
        TRANSACTION_HANDLER,
        STATE,
    >
where
    STREAM: TransactionStream,
    EVENT_HANDLER: EventHandler<EVENT_HANDLER, STATE>,
    STATE: Clone,
    TRANSACTION_HANDLER: FnMut(
        &mut STATE,
        &Transaction,
        &mut HandlerRegistry<EVENT_HANDLER, STATE>,
    ),
{
    pub fn new(
        transaction_stream: STREAM,
        handler_registry: HandlerRegistry<EVENT_HANDLER, STATE>,
        transaction_handler: TRANSACTION_HANDLER,
        state_version_report_interval: Option<std::time::Duration>,
        state: STATE,
    ) -> Self {
        TransactionStreamProcessor {
            transaction_stream,
            handler_registry,
            transaction_handler,
            state_version_report_interval,
            time_last_state_version_reported: std::time::Instant::now(),
            app_state: state,
        }
    }

    pub fn run(&mut self) {
        loop {
            let transactions = match self.transaction_stream.next() {
                Err(error) => match error {
                    TransactionStreamError::CaughtUp => {
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
                // match self.state_version_report_interval {
                //     Some(interval) => {
                //         if self.time_last_state_version_reported.elapsed()
                //             > interval
                //         {
                //             info!(
                //                 "State version: {} - Date: {:?}",
                //                 transaction.state_version,
                //                 transaction.confirmed_at.unwrap_or(Utc::now())
                //             );
                //             self.time_last_state_version_reported =
                //                 std::time::Instant::now();
                //         }
                //     }
                //     None => {
                //         info!("State version: {}", transaction.state_version);
                //     }
                // };
                // Check if any of the events in the transaction
                // have a handler registered.
                if !transaction.events.iter().any(|event| {
                    let event_handlers = match self
                        .handler_registry
                        .handlers
                        .get(event.emitter.address())
                    {
                        Some(handlers) => handlers,
                        None => return false,
                    };
                    event_handlers
                        .iter()
                        .any(|handler| handler.match_variant(&event.name))
                }) {
                    // No handlers for this transaction
                    return;
                }
                info!(
                    "{}",
                    format!(
                        "NEW TRANSACTION - {:#?} - {}",
                        transaction.state_version,
                        transaction.confirmed_at
                            .expect("When handling a transaction it should always have a timestamp")
                            .to_rfc3339()
                    )
                    .green()
                );
                (self.transaction_handler)(
                    &mut self.app_state,
                    transaction,
                    &mut self.handler_registry,
                );
                info!("{}", "###### END TRANSACTION ######\n".green());
            });
        }
    }

    pub fn run_with(
        transaction_stream: STREAM,
        handler_registry: HandlerRegistry<EVENT_HANDLER, STATE>,
        transaction_handler: TRANSACTION_HANDLER,
        state_version_report_interval: Option<std::time::Duration>,
        state: STATE,
    ) {
        let mut processor = TransactionStreamProcessor::new(
            transaction_stream,
            handler_registry,
            transaction_handler,
            state_version_report_interval,
            state,
        );
        processor.run();
    }
}
