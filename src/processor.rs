use crate::{
    error::{
        EventHandlerError, TransactionHandlerError,
        TransactionStreamProcessorError,
    },
    event_handler::{EventHandlerContext, HandlerRegistry},
    models::{Event, Transaction},
    stream::TransactionStream,
    transaction_handler::{TransactionHandler, TransactionHandlerContext},
};
use async_trait::async_trait;
use colored::Colorize;
use log::{error, info, Log};
use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

const CURRENT_STATE_REPORT_INTERVAL: u64 = 60;
const TRANSACTION_RETRY_INTERVAL: u64 = 10;
const EVENT_RETRY_INTERVAL: u64 = 10;

pub trait Logger: Send + Sync {
    fn before_handle_transaction(&self, transaction: &Transaction);
    fn after_handle_transaction(
        &self,
        transaction: &Transaction,
        time_spent: Duration,
    );
    fn before_handle_event(&self, transaction: &Transaction, event: &Event);
    fn after_handle_event(
        &self,
        transaction: &Transaction,
        event: &Event,
        time_spent: Duration,
    );
    fn event_retry_error(
        &self,
        transaction: &Transaction,
        event: &Event,
        error: &anyhow::Error,
        timeout: Duration,
    );
    fn transaction_retry_error(
        &self,
        transaction: &Transaction,
        error: &anyhow::Error,
        timeout: Duration,
    );
    fn unrecoverable_error(&self, error: &anyhow::Error);
}

pub struct DefaultLogger;

impl Logger for DefaultLogger {
    fn before_handle_transaction(&self, transaction: &Transaction) {
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
    }

    fn after_handle_transaction(
        &self,
        transaction: &Transaction,
        time_spent: Duration,
    ) {
        info!(
            "{}",
            format!(
                "###### END TRANSACTION - HANDLED IN {}ms ######",
                time_spent.as_millis()
            )
            .bright_green()
        );
        info!(
            "{}",
            "--------------------------------------------------------"
                .bright_blue()
        );
    }

    fn before_handle_event(&self, transaction: &Transaction, event: &Event) {
        info!(
            "{}",
            format!("HANDLING EVENT: {}", event.name).bright_yellow()
        );
    }

    fn after_handle_event(
        &self,
        transaction: &Transaction,
        event: &Event,
        time_spent: Duration,
    ) {
    }

    fn event_retry_error(
        &self,
        transaction: &Transaction,
        event: &Event,
        error: &anyhow::Error,
        timeout: Duration,
    ) {
        error!(
            "{}",
            format!("ERROR HANDLING EVENT: {} - {:?}", event.name, error)
                .bright_red()
        );
        info!(
            "{}",
            format!("RETRYING IN {:.2} SECONDS\n", timeout.as_secs_f32())
                .bright_yellow()
        );
    }

    fn transaction_retry_error(
        &self,
        transaction: &Transaction,
        error: &anyhow::Error,
        timeout: Duration,
    ) {
        error!(
            "{}",
            format!("FATAL ERROR HANDLING TRANSACTION: {:?}\n", error)
                .bright_red()
        );
        info!(
            "{}",
            format!("RETRYING IN {:.2} SECONDS\n", timeout.as_secs_f32())
                .bright_yellow()
        );
    }

    fn unrecoverable_error(&self, error: &anyhow::Error) {
        error!(
            "{}",
            format!("UNRECOVERABLE ERROR: {:?}", error).bright_red()
        );
    }
}

/// Uses a `TransactionStream` to procoess transactions and
/// events using a `HandlerRegistry`. Register event handlers
/// using the `HandlerRegistry` and then call `run` to start
/// processing transactions.
#[allow(non_camel_case_types)]
pub struct TransactionStreamProcessor<STREAM, STATE>
where
    STREAM: TransactionStream,
{
    transaction_stream: STREAM,
    handler_registry: HandlerRegistry,
    transaction_handler: Box<dyn TransactionHandler<STATE>>,
    state: STATE,
    state_version_last_reported: Instant,
    transaction_retry_delay: Duration,
    event_retry_delay: Duration,
    current_state_report_interval: Duration,
    current_state: Arc<RwLock<Option<u64>>>,
    logger: Option<Box<dyn Logger>>,
}

#[allow(non_camel_case_types)]
impl<STREAM, STATE> TransactionStreamProcessor<STREAM, STATE>
where
    STREAM: TransactionStream,
    STATE: Send + Sync + 'static,
{
    pub fn new(
        transaction_stream: STREAM,
        handler_registry: HandlerRegistry,
        state: STATE,
    ) -> Self {
        TransactionStreamProcessor {
            transaction_stream,
            handler_registry,
            transaction_handler: Box::new(DefaultTransactionHandler {}),
            state_version_last_reported: Instant::now(),
            state: state,
            transaction_retry_delay: Duration::from_millis(
                TRANSACTION_RETRY_INTERVAL,
            ),
            event_retry_delay: Duration::from_millis(EVENT_RETRY_INTERVAL),
            current_state_report_interval: Duration::from_millis(
                CURRENT_STATE_REPORT_INTERVAL,
            ),
            current_state: Arc::new(RwLock::new(None)),
            logger: None,
        }
    }

    pub fn transaction_handler(
        self,
        transaction_handler: impl TransactionHandler<STATE> + 'static,
    ) -> Self {
        TransactionStreamProcessor {
            transaction_handler: Box::new(transaction_handler),
            ..self
        }
    }

    pub fn transaction_retry_delay_ms(
        self,
        transaction_retry_delay_ms: u64,
    ) -> Self {
        TransactionStreamProcessor {
            transaction_retry_delay: Duration::from_millis(
                transaction_retry_delay_ms,
            ),
            ..self
        }
    }

    pub fn event_retry_delay_ms(self, event_retry_delay_ms: u64) -> Self {
        TransactionStreamProcessor {
            event_retry_delay: Duration::from_millis(event_retry_delay_ms),
            ..self
        }
    }

    pub fn current_state_report_interval_ms(
        self,
        current_state_report_interval_ms: u64,
    ) -> Self {
        TransactionStreamProcessor {
            current_state_report_interval: Duration::from_millis(
                current_state_report_interval_ms,
            ),
            ..self
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

        if let Some(logger) = &self.logger {
            logger.before_handle_transaction(transaction);
        }

        // Keep trying to handle the transaction in case
        // the handler requests this through a TransactionHandlerError.
        while let Err(err) = self
            .transaction_handler
            .handle(TransactionHandlerContext {
                state: &mut self.state,
                transaction,
                event_processor: &mut EventProcessor {
                    event_retry_interval: self.event_retry_delay,
                    transaction,
                    logger: &self.logger,
                },
                handler_registry: &mut self.handler_registry,
            })
            .await
        {
            match err {
                TransactionHandlerError::TransactionRetryError(e) => {
                    if let Some(logger) = &self.logger {
                        logger.transaction_retry_error(
                            transaction,
                            &e,
                            self.transaction_retry_delay,
                        );
                    }
                    tokio::time::sleep(self.transaction_retry_delay).await;
                    if let Some(logger) = &self.logger {
                        logger.before_handle_transaction(transaction);
                    }
                    continue;
                }
                TransactionHandlerError::UnrecoverableError(e) => {
                    if let Some(logger) = &self.logger {
                        logger.unrecoverable_error(&e);
                    }
                    return Err(
                        TransactionStreamProcessorError::UnrecoverableError(e),
                    );
                }
            }
        }
        if let Some(logger) = &self.logger {
            logger.after_handle_transaction(transaction, before.elapsed());
        }
        self.current_state
            .write()
            .expect("Should be able to write the current state")
            .replace(transaction.state_version);
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
            if self.state_version_last_reported.elapsed()
                > self.current_state_report_interval
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
            .event_processor
            .process_events(input.state, input.handler_registry, &mut ())
            .await?;
        Ok(())
    }
}

pub struct EventProcessor<'a> {
    event_retry_interval: Duration,
    transaction: &'a Transaction,
    logger: &'a Option<Box<dyn Logger>>,
}

#[allow(non_camel_case_types)]
impl<'a> EventProcessor<'a> {
    pub async fn process_events<
        STATE: 'static,
        TRANSACTION_CONTEXT: 'static,
    >(
        &self,
        state: &mut STATE,
        handler_registry: &mut HandlerRegistry,
        transaction_context: &mut TRANSACTION_CONTEXT,
    ) -> Result<(), EventHandlerError> {
        for event in self.transaction.events.iter() {
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
            let mut before: Option<Instant> = None;
            if let Some(logger) = self.logger {
                before = Some(Instant::now());
                logger.before_handle_event(self.transaction, event);
            }
            while let Err(err) = event_handler
                .handle(
                    EventHandlerContext {
                        state: state,
                        transaction: self.transaction,
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
                        if let Some(logger) = self.logger {
                            logger.event_retry_error(
                                self.transaction,
                                event,
                                &e,
                                self.event_retry_interval,
                            );
                        }
                        tokio::time::sleep(self.event_retry_interval).await;
                        if let Some(logger) = self.logger {
                            logger.before_handle_event(self.transaction, event);
                        }
                        continue;
                    }
                    _ => {
                        return Err(err);
                    }
                }
            }
            if let Some(logger) = self.logger {
                logger.after_handle_event(
                    self.transaction,
                    event,
                    before.unwrap().elapsed(),
                );
            }
        }
        Ok(())
    }
}
