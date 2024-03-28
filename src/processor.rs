use crate::{
    error::{EventHandlerError, TransactionHandlerError},
    event_handler::{EventHandlerContext, HandlerRegistry},
    models::IncomingTransaction,
    stream::{TransactionStream, TransactionStreamError},
    transaction_handler::{TransactionHandler, TransactionHandlerContext},
};
use colored::Colorize;
use log::{error, info};
use std::thread::sleep;

/// Uses a `TransactionStream` to process transactions and
/// events using a `HandlerRegistry`. Register event handlers
/// using the `HandlerRegistry` and then call `run` to start
/// processing transactions.
#[allow(non_camel_case_types)]
pub struct TransactionStreamProcessor<STREAM, STATE>
where
    STREAM: TransactionStream,
    STATE: Clone,
{
    pub transaction_stream: STREAM,
    pub handler_registry: HandlerRegistry<STATE>,
    pub transaction_handler: Box<dyn TransactionHandler<STATE>>,
    pub app_state: STATE,
}

#[allow(non_camel_case_types)]
impl<STREAM, STATE> TransactionStreamProcessor<STREAM, STATE>
where
    STREAM: TransactionStream,
    STATE: Clone,
{
    /// Creates a new `TransactionStreamProcessor` with the given
    /// `TransactionStream`, `HandlerRegistry`, `TransactionHandler`
    /// and initial `STATE`.
    pub fn new(
        transaction_stream: STREAM,
        handler_registry: HandlerRegistry<STATE>,
        transaction_handler: impl TransactionHandler<STATE> + 'static,
        state: STATE,
    ) -> Self {
        TransactionStreamProcessor {
            transaction_stream,
            handler_registry,
            transaction_handler: Box::new(transaction_handler),
            app_state: state,
        }
    }

    /// Starts processing transactions from the `TransactionStream`.
    pub fn run(&mut self) {
        // Keep processing in an infinite loop.
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
                        return;
                    }
                    TransactionStreamError::Error(error) => {
                        error!("Error while getting transactions: {}", error);
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        continue;
                    }
                },
                Ok(transactions) => transactions,
            };

            // Process each transaction.
            for transaction in transactions.iter() {
                // Find out if there are any events inside this transaction
                // that have a handler registered.
                let handler_exists = transaction.events.iter().any(|event| {
                    self.handler_registry
                        .get_handler(event.emitter.address(), &event.name)
                        .is_some()
                });
                if !handler_exists {
                    // If there are no handlers for any of the events in this transaction,
                    // we can skip processing it.
                    continue;
                }
                info!(
                    "{}",
                    "--------------------------------------------------------"
                        .bright_blue()
                );
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

                // Keep trying to handle the transaction in case
                // the user requests this through a TransactionHandlerError.
                while let Err(err) =
                    self.transaction_handler.handle(TransactionHandlerContext {
                        app_state: &mut self.app_state,
                        transaction,
                        handler_registry: &mut self.handler_registry,
                    })
                {
                    match err {
                        TransactionHandlerError::TransactionRetryError(e) => {
                            error!(
                                "{}",
                                format!("ERROR HANDLING TRANSACTION: {}", e)
                                    .bright_red()
                            );
                            info!(
                                "{}",
                                format!("RETRYING TRANSACTION IN 10 SECONDS\n")
                                    .bright_yellow()
                            );
                            sleep(std::time::Duration::from_secs(10));
                            info!(
                                "{}",
                                format!(
                                    "RETRYING TRANSACTION - {:#?} - {}",
                                    transaction.state_version,
                                    transaction.confirmed_at
                                        .expect("When handling a transaction it should always have a timestamp")
                                        .to_rfc3339()
                                )
                                .bright_yellow()
                            );
                            continue;
                        }
                        TransactionHandlerError::UnrecoverableError(err) => {
                            error!(
                                "{}",
                                format!(
                                    "FATAL ERROR HANDLING TRANSACTION: {}\n",
                                    err
                                )
                                .bright_red()
                            );
                            std::process::exit(1);
                        }
                    }
                }
                info!("{}", "###### END TRANSACTION ######".bright_green());
                info!(
                    "{}",
                    "--------------------------------------------------------"
                        .bright_blue()
                );
            }
        }
    }

    // Shorthand for running the processor with the required parameters.
    pub fn run_with(
        transaction_stream: STREAM,
        handler_registry: HandlerRegistry<STATE>,
        transaction_handler: impl TransactionHandler<STATE> + 'static,
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

/// A simple wrapper around `TransactionStreamProcessor` that uses
/// a default transaction handler that simply calls `handle_events`
/// on the transaction. This is useful for simple use cases where
/// you don't need any custom transaction handling logic.
pub struct SimpleTransactionStreamProcessor<STREAM, STATE>
where
    STREAM: TransactionStream,
    STATE: Clone,
{
    processor: TransactionStreamProcessor<STREAM, STATE>,
}

#[allow(non_camel_case_types)]
impl<STREAM, STATE> SimpleTransactionStreamProcessor<STREAM, STATE>
where
    STREAM: TransactionStream,
    STATE: Clone + 'static,
{
    pub fn new(
        transaction_stream: STREAM,
        handler_registry: HandlerRegistry<STATE>,
        state: STATE,
    ) -> Self {
        let processor: TransactionStreamProcessor<STREAM, STATE> =
            TransactionStreamProcessor::new(
                transaction_stream,
                handler_registry,
                default_transaction_handler,
                state,
            );
        SimpleTransactionStreamProcessor { processor }
    }

    pub fn run(&mut self) {
        self.processor.run();
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

/// A default transaction handler that simply calls `handle_events`
/// on the transaction, without any custom logic.
fn default_transaction_handler<STATE: Clone>(
    context: TransactionHandlerContext<STATE>,
) -> Result<(), TransactionHandlerError> {
    context
        .transaction
        .handle_events(context.app_state, context.handler_registry)
        .unwrap();
    Ok(())
}

#[allow(non_camel_case_types)]
impl IncomingTransaction {
    /// Convenience method which iterates over the events in the
    /// transaction and calls the appropriate event handler
    /// for events which have a handler
    /// registered in the `HandlerRegistry`.
    ///
    /// When event handlers return an `EventHandlerError::EventRetryError`,
    /// this method will keep retrying handling the event until it succeeds.
    /// Please consider that event handlers may be called multiple times
    /// in this case, so they must be idempotent at least up to the point
    /// where the error occurred.
    pub fn handle_events<STATE>(
        &self,
        app_state: &mut STATE,
        handler_registry: &mut HandlerRegistry<STATE>,
    ) -> Result<(), EventHandlerError>
    where
        STATE: Clone,
    {
        for event in self.events.iter() {
            let handler_registry_clone = handler_registry.clone();
            let event_handler = match handler_registry_clone
                .get_handler(event.emitter.address(), &event.name)
            {
                Some(handler) => handler,
                None => continue,
            };

            info!(
                "{}",
                format!("HANDLING EVENT: {}", event.name).bright_yellow()
            );
            while let Err(err) = event_handler.handle(
                EventHandlerContext {
                    app_state,
                    transaction: self,
                    event,
                    handler_registry,
                },
                event.binary_sbor_data.clone(),
            ) {
                match err {
                    EventHandlerError::EventRetryError(e) => {
                        error!(
                            "{}",
                            format!("ERROR HANDLING EVENT: {}", e).bright_red()
                        );

                        info!(
                            "{}",
                            format!("RETRYING IN 10 SECONDS\n").bright_yellow()
                        );
                        sleep(std::time::Duration::from_secs(10));
                        info!(
                            "{}",
                            format!("RETRYING HANDLING EVENT: {}", event.name)
                                .bright_yellow()
                        );
                        continue;
                    }
                    _ => {
                        return Err(err);
                    }
                }
            }
        }
        Ok(())
    }
}
