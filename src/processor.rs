use crate::{
    error::{EventHandlerError, TransactionHandlerError},
    event_handler::{EventHandlerContext, HandlerRegistry},
    models::IncomingTransaction,
    stream::{TransactionStream, TransactionStreamError},
    transaction_handler::{TransactionHandler, TransactionHandlerContext},
};
use colored::Colorize;
use log::{error, info, warn};
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    thread::sleep,
};

/// Uses a `TransactionStream` to process transactions and
/// events using a `HandlerRegistry`. Register event handlers
/// using the `HandlerRegistry` and then call `run` to start
/// processing transactions.
#[allow(non_camel_case_types)]
pub struct TransactionStreamProcessor<STREAM, STATE, TX_CTX>
where
    STREAM: TransactionStream,
    STATE: Clone,
{
    pub transaction_stream: STREAM,
    pub handler_registry: Arc<Mutex<HandlerRegistry<STATE, TX_CTX>>>,
    pub transaction_handler: Box<dyn TransactionHandler<STATE, TX_CTX>>,
    pub app_state: STATE,
}

#[allow(non_camel_case_types)]
impl<STREAM, STATE, TX_CTX> TransactionStreamProcessor<STREAM, STATE, TX_CTX>
where
    STREAM: TransactionStream,
    STATE: Clone,
{
    /// Creates a new `TransactionStreamProcessor` with the given
    /// `TransactionStream`, `HandlerRegistry`, `TransactionHandler`
    /// and initial `STATE`.
    pub fn new(
        transaction_stream: STREAM,
        handler_registry: HandlerRegistry<STATE, TX_CTX>,
        transaction_handler: impl TransactionHandler<STATE, TX_CTX> + 'static,
        state: STATE,
    ) -> Self {
        TransactionStreamProcessor {
            transaction_stream,
            handler_registry: Arc::new(Mutex::new(handler_registry)),
            transaction_handler: Box::new(transaction_handler),
            app_state: state,
        }
    }

    /// Starts processing transactions from the `TransactionStream`.
    pub async fn run(&mut self) {
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
                        .lock()
                        .unwrap()
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
                while let Err(err) = self
                    .transaction_handler
                    .handle(TransactionHandlerContext {
                        app_state: &mut self.app_state,
                        transaction,
                        handler_registry: &self.handler_registry,
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
            }
        }
    }

    // Shorthand for running the processor with the required parameters.
    pub async fn run_with(
        transaction_stream: STREAM,
        handler_registry: HandlerRegistry<STATE, TX_CTX>,
        transaction_handler: impl TransactionHandler<STATE, TX_CTX> + 'static,
        state: STATE,
    ) {
        let mut processor = TransactionStreamProcessor::new(
            transaction_stream,
            handler_registry,
            transaction_handler,
            state,
        );
        processor.run().await;
    }
}

/// A simple wrapper around `TransactionStreamProcessor` that uses
/// a default transaction handler that simply calls `handle_events`
/// on the transaction. This is useful for simple use cases where
/// you don't need any custom transaction handling logic.
pub struct SimpleTransactionStreamProcessor<STREAM, STATE, TX_CTX>
where
    STREAM: TransactionStream,
    STATE: Clone,
{
    processor: TransactionStreamProcessor<STREAM, STATE, TX_CTX>,
}

#[allow(non_camel_case_types)]
impl<STREAM, STATE, TX_CTX>
    SimpleTransactionStreamProcessor<STREAM, STATE, TX_CTX>
where
    STREAM: TransactionStream,
    STATE: Send + Sync + Clone + 'static,
    TX_CTX: Send + Sync + 'static,
{
    pub fn new(
        transaction_stream: STREAM,
        handler_registry: HandlerRegistry<STATE, TX_CTX>,
        state: STATE,
    ) -> Self {
        let processor: TransactionStreamProcessor<STREAM, STATE, TX_CTX> =
            TransactionStreamProcessor::new(
                transaction_stream,
                handler_registry,
                default_transaction_handler,
                state,
            );
        SimpleTransactionStreamProcessor { processor }
    }

    pub async fn run(&mut self) {
        self.processor.run().await;
    }

    pub async fn run_with(
        transaction_stream: STREAM,
        handler_registry: HandlerRegistry<STATE, TX_CTX>,
        state: STATE,
    ) {
        let mut processor = SimpleTransactionStreamProcessor::new(
            transaction_stream,
            handler_registry,
            state,
        );
        processor.run().await;
    }
}

/// A default transaction handler that simply calls `handle_events`
/// on the transaction, without any custom logic.
fn default_transaction_handler<STATE, TX_CTX>(
    context: TransactionHandlerContext<'_, STATE, TX_CTX>,
) -> Pin<
    Box<dyn Future<Output = Result<(), TransactionHandlerError>> + Send + '_>,
>
where
    STATE: Clone + Send + Sync,
    TX_CTX: Send + Sync,
{
    Box::pin(async {
        context
            .transaction
            .handle_events(context.app_state, context.handler_registry, None)
            .await
            .unwrap();
        Ok(())
    })
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
    pub async fn handle_events<STATE, TX_CTX>(
        &self,
        app_state: &mut STATE,
        handler_registry: &Arc<Mutex<HandlerRegistry<STATE, TX_CTX>>>,
        tx_ctx: Option<&TX_CTX>,
    ) -> Result<(), EventHandlerError>
    where
        STATE: Clone,
    {
        for event in self.events.iter() {
            let event_handler = {
                let handler_registry_clone = handler_registry.clone();
                let guard = handler_registry_clone.lock().unwrap();
                let event_handler = match guard
                    .get_handler(event.emitter.address(), &event.name)
                {
                    Some(handler) => handler,
                    None => continue,
                };
                event_handler.clone()
            };

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
                    },
                    event.binary_sbor_data.clone(),
                    tx_ctx,
                )
                .await
            {
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
