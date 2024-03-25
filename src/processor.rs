use log::{info, warn};

use crate::{
    handler::HandlerRegistry,
    models::Transaction,
    stream::{TransactionStream, TransactionStreamError},
};
use colored::Colorize;

/// Uses a `TransactionStream` to process transactions and
/// events using a `HandlerRegistry`. Register event handlers
/// using the `HandlerRegistry` and then call `run` to start
/// processing transactions.
#[allow(non_camel_case_types)]
pub struct TransactionStreamProcessor<STREAM, TRANSACTION_HANDLER, STATE>
where
    STREAM: TransactionStream,
    STATE: Clone,
    TRANSACTION_HANDLER:
        FnMut(&mut STATE, &Transaction, &mut HandlerRegistry<STATE>),
{
    pub transaction_stream: STREAM,
    pub handler_registry: HandlerRegistry<STATE>,
    pub transaction_handler: TRANSACTION_HANDLER,
    pub app_state: STATE,
}

#[allow(non_camel_case_types)]
impl<STREAM, TRANSACTION_HANDLER, STATE>
    TransactionStreamProcessor<STREAM, TRANSACTION_HANDLER, STATE>
where
    STREAM: TransactionStream,
    STATE: Clone,
    TRANSACTION_HANDLER:
        FnMut(&mut STATE, &Transaction, &mut HandlerRegistry<STATE>),
{
    pub fn new(
        transaction_stream: STREAM,
        handler_registry: HandlerRegistry<STATE>,
        transaction_handler: TRANSACTION_HANDLER,
        state: STATE,
    ) -> Self {
        TransactionStreamProcessor {
            transaction_stream,
            handler_registry,
            transaction_handler,
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
                        info!(
                            "{}",
                            format!("Finished processing transactions")
                                .bright_red()
                        );
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
                if !transaction.events.iter().any(|event| {
                     self
                        .handler_registry
                        .handlers
                        .get(&(event.emitter.address().to_string(), event.name.clone())).is_some()
                }) {
                    // No handlers for this transaction
                    return;
                }
                info!(
                    "{}",
                    format!(
                        "HANDLING TRANSACTION - {:#?} - {}",
                        transaction.state_version,
                        transaction.confirmed_at
                            .expect("When handling a transaction it should always have a timestamp")
                            .to_rfc3339()
                    )
                    .bright_green()
                );
                (self.transaction_handler)(
                    &mut self.app_state,
                    transaction,
                    &mut self.handler_registry,
                );
                info!("{}", "###### END TRANSACTION ######\n".bright_green());
            });
        }
    }

    pub fn run_with(
        transaction_stream: STREAM,
        handler_registry: HandlerRegistry<STATE>,
        transaction_handler: TRANSACTION_HANDLER,
        state: STATE,
    ) {
        let mut processor = TransactionStreamProcessor::new(
            transaction_stream,
            handler_registry,
            transaction_handler,
            state,
        );
        processor.run();
    }
}

pub struct SimpleTransactionStreamProcessor<STREAM, STATE>
where
    STREAM: TransactionStream,
    STATE: Clone,
{
    pub transaction_stream: STREAM,
    pub handler_registry: HandlerRegistry<STATE>,
    pub app_state: STATE,
}

impl<STREAM, STATE> SimpleTransactionStreamProcessor<STREAM, STATE>
where
    STREAM: TransactionStream,
    STATE: Clone,
{
    pub fn new(
        transaction_stream: STREAM,
        handler_registry: HandlerRegistry<STATE>,
        state: STATE,
    ) -> Self {
        SimpleTransactionStreamProcessor {
            transaction_stream,
            handler_registry,
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
                        info!(
                            "{}",
                            format!("Finished processing transactions")
                                .bright_red()
                        );
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
                if !transaction.events.iter().any(|event| {
                     self
                        .handler_registry
                        .handlers
                        .get(&(event.emitter.address().to_string(), event.name.clone())).is_some()
                }) {
                    // No handlers for this transaction
                    return;
                }
                info!(
                    "{}",
                    format!(
                        "HANDLING TRANSACTION - {:#?} - {}",
                        transaction.state_version,
                        transaction.confirmed_at
                            .expect("When handling a transaction it should always have a timestamp")
                            .to_rfc3339()
                    )
                    .bright_green()
                );
                transaction.handle_events(&mut self.app_state, &mut self.handler_registry);
                info!("{}", "###### END TRANSACTION ######\n".bright_green());
            });
        }
    }

    pub fn run_with(
        transaction_stream: STREAM,
        handler_registry: HandlerRegistry<STATE>,
        state: STATE,
    ) {
        let mut processor = SimpleTransactionStreamProcessor::new(
            transaction_stream,
            handler_registry,
            state,
        );
        processor.run();
    }
}
