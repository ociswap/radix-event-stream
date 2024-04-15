use crate::{
    error::{
        EventHandlerError, TransactionHandlerError,
        TransactionStreamProcessorError,
    },
    event_handler::{EventHandlerContext, HandlerRegistry},
    models::Transaction,
    stream::TransactionStream,
    transaction_handler::{TransactionHandler, TransactionHandlerContext},
};
use async_trait::async_trait;
use colored::Colorize;
use log::{error, info};
use std::{thread::sleep, time::Instant};

const CURRENT_STATE_REPORT_INTERVAL: u64 = 60;
const TRANSACTION_RETRY_INTERVAL: u64 = 10;
const EVENT_RETRY_INTERVAL: u64 = 10;

/// Uses a `TransactionStream` to process transactions and
/// events using a `HandlerRegistry`. Register event handlers
/// using the `HandlerRegistry` and then call `run` to start
/// processing transactions.
#[allow(non_camel_case_types)]
pub struct TransactionStreamProcessor<STREAM, STATE>
where
    STREAM: TransactionStream,
{
    pub transaction_stream: STREAM,
    pub handler_registry: HandlerRegistry,
    pub transaction_handler: Box<dyn TransactionHandler<STATE>>,
    pub state: STATE,
    pub state_version_last_reported: Instant,
}

#[allow(non_camel_case_types)]
impl<STREAM, STATE> TransactionStreamProcessor<STREAM, STATE>
where
    STREAM: TransactionStream,
{
    /// Creates a new `TransactionStreamProcessor` with the given
    /// `TransactionStream`, `HandlerRegistry`, `TransactionHandler`
    /// and initial `STATE`.
    pub fn new(
        transaction_stream: STREAM,
        handler_registry: HandlerRegistry,
        transaction_handler: impl TransactionHandler<STATE> + 'static,
        state: STATE,
    ) -> Self {
        TransactionStreamProcessor {
            transaction_stream,
            handler_registry,
            transaction_handler: Box::new(transaction_handler),
            state,
            state_version_last_reported: Instant::now(),
        }
    }

    pub async fn process_transaction(
        &mut self,
        transaction: &Transaction,
    ) -> Result<(), TransactionStreamProcessorError> {
        let before = Instant::now();
        // Find out if there are any events inside this transaction
        // that have a handler registered.
        let handler_exists = transaction.events.iter().any(|event| {
            self.handler_registry
                .handler_exists(event.emitter.address(), &event.name)
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
                state: &mut self.state,
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
                        format!("RETRYING TRANSACTION IN {TRANSACTION_RETRY_INTERVAL} SECONDS\n")
                            .bright_yellow()
                    );
                    sleep(std::time::Duration::from_secs(
                        TRANSACTION_RETRY_INTERVAL,
                    ));
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
        info!(
            "{}",
            format!(
                "###### END TRANSACTION - HANDLED IN {}ms ######",
                before.elapsed().as_millis()
            )
            .bright_green()
        );
        info!(
            "{}",
            "--------------------------------------------------------"
                .bright_blue()
        );
        Ok(())
    }

    /// Starts processing transactions from the `TransactionStream`.
    pub async fn run(&mut self) -> Result<(), TransactionStreamProcessorError> {
        // Start the transaction stream and get a receiver.
        // This often involves starting a task that fetches transactions
        // from a remote source and sends them to the receiver.
        let mut receiver =
            self.transaction_stream.start().await.map_err(|error| {
                TransactionStreamProcessorError::UnrecoverableError(error)
            })?;
        // Process transactions as they arrive.
        while let Some(transaction) = receiver.recv().await {
            if self.state_version_last_reported.elapsed().as_secs()
                > CURRENT_STATE_REPORT_INTERVAL
            {
                info!(
                    "{}",
                    format!(
                        "HANDLED UP TO: {} - {}",
                        transaction.state_version,
                        transaction.confirmed_at
                            .expect("When handling a transaction it should always have a timestamp")
                            .format("%a %d-%m-%Y %H:%M")
                    )
                    .bright_blue()
                );
                self.state_version_last_reported = Instant::now();
            }
            self.process_transaction(&transaction).await?;
        }
        // If the transmitting half of the channel is dropped,
        // the receiver will return None and we will exit the loop.
        // The processor will exit gracefully.
        Ok(())
    }

    // Shorthand for running the processor with the required parameters.
    pub async fn run_with(
        transaction_stream: STREAM,
        handler_registry: HandlerRegistry,
        transaction_handler: impl TransactionHandler<STATE> + 'static,
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
/// a default transaction handler that simply calls `process_events`
/// on the transaction. This is useful for simple use cases where
/// you don't need any custom transaction handling logic.
#[allow(non_camel_case_types)]
pub struct SimpleTransactionStreamProcessor<STREAM, STATE>
where
    STREAM: TransactionStream,
{
    processor: TransactionStreamProcessor<STREAM, STATE>,
}

#[allow(non_camel_case_types)]
impl<STREAM, STATE> SimpleTransactionStreamProcessor<STREAM, STATE>
where
    STREAM: TransactionStream,
    STATE: 'static + Send + Sync,
{
    pub fn new(
        transaction_stream: STREAM,
        handler_registry: HandlerRegistry,
        state: STATE,
    ) -> Self {
        let processor: TransactionStreamProcessor<STREAM, STATE> =
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
        handler_registry: HandlerRegistry,
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

/// A default transaction handler that simply calls `process_events`
/// on the transaction, without any custom logic.
#[derive(Clone)]
struct DefaultTransactionHandler;

#[async_trait]
impl<STATE> TransactionHandler<STATE> for DefaultTransactionHandler
where
    STATE: Send + Sync + 'static,
{
    async fn handle(
        &self,
        input: TransactionHandlerContext<'_, STATE>,
    ) -> Result<(), TransactionHandlerError> {
        input
            .transaction
            .process_events(input.state, input.handler_registry, &mut ())
            .await?;
        Ok(())
    }
}

#[allow(non_camel_case_types)]
impl Transaction {
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
    pub async fn process_events<
        STATE: 'static,
        TRANSACTION_CONTEXT: 'static,
    >(
        &self,
        state: &mut STATE,
        handler_registry: &mut HandlerRegistry,
        transaction_context: &mut TRANSACTION_CONTEXT,
    ) -> Result<(), EventHandlerError> {
        for event in self.events.iter() {
            let event_handler = {
                if !handler_registry
                    .handler_exists(event.emitter.address(), &event.name)
                {
                    continue;
                }
                handler_registry
                    .get_handler::<STATE, TRANSACTION_CONTEXT>(
                        event.emitter.address(),
                        &event.name,
                    )
                    .unwrap()
            };
            let event_handler = event_handler.clone();
            info!(
                "{}",
                format!("HANDLING EVENT: {}", event.name).bright_yellow()
            );
            while let Err(err) = event_handler
                .handle(
                    EventHandlerContext {
                        state,
                        transaction: self,
                        event,
                        handler_registry,
                        transaction_context,
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

                        info!(
                            "{}",
                            format!(
                                "RETRYING IN {EVENT_RETRY_INTERVAL} SECONDS\n"
                            )
                            .bright_yellow()
                        );
                        sleep(std::time::Duration::from_secs(
                            EVENT_RETRY_INTERVAL,
                        ));
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
