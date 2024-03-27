use std::thread::sleep;

use log::{error, info, warn};
use radix_engine_toolkit::schema_visitor::core::error;

use crate::{
    error::{EventHandlerError, TransactionHandlerError},
    event_handler::{EventHandlerContext, HandlerRegistry},
    models::IncomingTransaction,
    stream::{TransactionStream, TransactionStreamError},
    transaction_handler::{TransactionHandler, TransactionHandlerContext},
};
use colored::Colorize;

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
                        return;
                    }
                    TransactionStreamError::Error(error) => {
                        warn!("Error while getting transactions: {}", error);
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        continue;
                    }
                },
                Ok(transactions) => transactions,
            };

            for transaction in transactions.iter() {
                let handler_exists = transaction.events.iter().any(|event| {
                    self.handler_registry
                        .handlers
                        .get(&(
                            event.emitter.address().to_string(),
                            event.name.clone(),
                        ))
                        .is_some()
                });
                if !handler_exists {
                    continue;
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
                                format!("RETRYING IN 10 SECONDS\n")
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
                info!("{}", "###### END TRANSACTION ######\n".bright_green());
            }
        }
    }

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
                .handlers
                .get(&(event.emitter.address().to_string(), event.name.clone()))
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
