use crate::{
    error::{
        EventHandlerError, TransactionHandlerError,
        TransactionStreamProcessorError,
    },
    event_handler::{EventHandlerContext, HandlerRegistry},
    models::IncomingTransaction,
    stream::{TransactionStream, TransactionStreamError},
    transaction_handler::{TransactionHandler, TransactionHandlerContext},
};

use async_trait::async_trait;
use colored::Colorize;
use log::{error, info};
use std::thread::sleep;

/// Uses a `TransactionStream` to process transactions and
/// events using a `HandlerRegistry`. Register event handlers
/// using the `HandlerRegistry` and then call `run` to start
/// processing transactions.
#[allow(non_camel_case_types)]
pub struct TransactionStreamProcessor<STREAM, STATE, TRANSACTION_HANDLE>
where
    STREAM: TransactionStream,
    STATE: Clone,
{
    pub transaction_stream: STREAM,
    pub handler_registry: HandlerRegistry<STATE, TRANSACTION_HANDLE>,
    pub transaction_handler:
        Box<dyn TransactionHandler<STATE, TRANSACTION_HANDLE>>,
    pub app_state: STATE,
}

#[allow(non_camel_case_types)]
impl<STREAM, STATE, TRANSACTION_HANDLE>
    TransactionStreamProcessor<STREAM, STATE, TRANSACTION_HANDLE>
where
    STREAM: TransactionStream,
    STATE: Clone,
{
    /// Creates a new `TransactionStreamProcessor` with the given
    /// `TransactionStream`, `HandlerRegistry`, `TransactionHandler`
    /// and initial `STATE`.
    pub fn new(
        transaction_stream: STREAM,
        handler_registry: HandlerRegistry<STATE, TRANSACTION_HANDLE>,
        transaction_handler: impl TransactionHandler<STATE, TRANSACTION_HANDLE>
            + 'static,
        state: STATE,
    ) -> Self {
        TransactionStreamProcessor {
            transaction_stream,
            handler_registry,
            transaction_handler: Box::new(transaction_handler),
            app_state: state,
        }
    }

    pub async fn process_transaction(
        &mut self,
        transaction: &IncomingTransaction,
    ) -> Result<(), TransactionStreamProcessorError> {
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
            return Ok(());
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
                    .format("%a %d-%m-%Y %H:%M")
            )
            .bright_green()
        );

        // Keep trying to handle the transaction in case
        // the user requests this through a TransactionHandlerError.
        while let Err(err) = self
            .transaction_handler
            .handle(TransactionHandlerContext {
                app_state: &mut self.app_state,
                transaction,
                handler_registry: &mut self.handler_registry,
            })
            .await
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
                        "RETRYING TRANSACTION IN 10 SECONDS\n".bright_yellow()
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
                        format!("FATAL ERROR HANDLING TRANSACTION: {}\n", err)
                            .bright_red()
                    );
                    return Err(
                        TransactionStreamProcessorError::UnrecoverableError(
                            err,
                        ),
                    );
                }
            }
        }
        info!("{}", "###### END TRANSACTION ######".bright_green());
        info!(
            "{}",
            "--------------------------------------------------------"
                .bright_blue()
        );
        Ok(())
    }

    /// Starts processing transactions from the `TransactionStream`.
    pub async fn run(&mut self) -> Result<(), TransactionStreamProcessorError> {
        // Keep processing in an infinite loop.
        loop {
            let transactions = match self.transaction_stream.next().await {
                Err(error) => match error {
                    TransactionStreamError::CaughtUp => {
                        info!("No more transactions, sleeping for 1 second...");
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        continue;
                    }
                    TransactionStreamError::Finished => {
                        info!(
                            "{}",
                            "Finished processing transactions".bright_red()
                        );
                        return Ok(());
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
                self.process_transaction(transaction).await?;
            }
        }
    }

    // Shorthand for running the processor with the required parameters.
    pub async fn run_with(
        transaction_stream: STREAM,
        handler_registry: HandlerRegistry<STATE, TRANSACTION_HANDLE>,
        transaction_handler: impl TransactionHandler<STATE, TRANSACTION_HANDLE>
            + 'static,
        state: STATE,
    ) -> Result<(), TransactionStreamProcessorError> {
        let mut processor = TransactionStreamProcessor::new(
            transaction_stream,
            handler_registry,
            transaction_handler,
            state,
        );
        processor.run().await
    }
}

/// A simple wrapper around `TransactionStreamProcessor` that uses
/// a default transaction handler that simply calls `handle_events`
/// on the transaction. This is useful for simple use cases where
/// you don't need any custom transaction handling logic.
#[allow(non_camel_case_types)]
pub struct SimpleTransactionStreamProcessor<STREAM, STATE>
where
    STREAM: TransactionStream,
    STATE: Clone,
{
    processor: TransactionStreamProcessor<STREAM, STATE, ()>,
}

#[allow(non_camel_case_types)]
impl<STREAM, STATE> SimpleTransactionStreamProcessor<STREAM, STATE>
where
    STREAM: TransactionStream,
    STATE: Clone + 'static + Send + Sync,
{
    pub fn new(
        transaction_stream: STREAM,
        handler_registry: HandlerRegistry<STATE, ()>,
        state: STATE,
    ) -> Self {
        let processor: TransactionStreamProcessor<STREAM, STATE, ()> =
            TransactionStreamProcessor::new(
                transaction_stream,
                handler_registry,
                DefaultTransactionHandler {},
                state,
            );
        SimpleTransactionStreamProcessor { processor }
    }

    pub async fn run(&mut self) -> Result<(), TransactionStreamProcessorError> {
        self.processor.run().await
    }

    pub async fn run_with(
        transaction_stream: STREAM,
        handler_registry: HandlerRegistry<STATE, ()>,
        state: STATE,
    ) -> Result<(), TransactionStreamProcessorError> {
        let mut processor = SimpleTransactionStreamProcessor::new(
            transaction_stream,
            handler_registry,
            state,
        );
        processor.run().await
    }
}

/// A default transaction handler that simply calls `handle_events`
/// on the transaction, without any custom logic.
#[derive(Clone)]
struct DefaultTransactionHandler;

#[async_trait]
impl<STATE> TransactionHandler<STATE, ()> for DefaultTransactionHandler
where
    STATE: Clone + Send + Sync,
{
    async fn handle(
        &self,
        input: TransactionHandlerContext<'_, STATE, ()>,
    ) -> Result<(), TransactionHandlerError> {
        input
            .transaction
            .handle_events(input.app_state, input.handler_registry, &mut ())
            .await
            .unwrap();
        Ok(())
    }
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
    pub async fn handle_events<STATE, TRANSACTION_HANDLE>(
        &self,
        app_state: &mut STATE,
        handler_registry: &mut HandlerRegistry<STATE, TRANSACTION_HANDLE>,
        transaction_handle: &mut TRANSACTION_HANDLE,
    ) -> Result<(), EventHandlerError>
    where
        STATE: Clone,
    {
        for event in self.events.iter() {
            let event_handler = {
                let handler = match handler_registry
                    .get_handler(event.emitter.address(), &event.name)
                {
                    Some(handler) => handler,
                    None => continue,
                };
                handler
            };
            let event_handler = event_handler.clone();
            info!(
                "{}",
                format!("HANDLING EVENT: {}", event.name).bright_yellow()
            );
            while let Err(err) = event_handler
                .handle(
                    EventHandlerContext {
                        app_state,
                        transaction: self,
                        event,
                        handler_registry,
                        transaction_handle,
                    },
                    event.binary_sbor_data.clone(),
                )
                .await
            {
                match err {
                    EventHandlerError::EventRetryError(e) => {
                        error!(
                            "{}",
                            format!("ERROR HANDLING EVENT: {}", e).bright_red()
                        );

                        info!("{}", "RETRYING IN 10 SECONDS\n".bright_yellow());
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
