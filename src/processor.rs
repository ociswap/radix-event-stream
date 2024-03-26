use std::thread::sleep;

use log::{error, info, warn};

use crate::{
    error::EventHandlerError,
    handler::HandlerRegistry,
    models::{EventHandlerContext, IncomingTransaction},
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
        FnMut(&mut STATE, &IncomingTransaction, &mut HandlerRegistry<STATE>),
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
        FnMut(&mut STATE, &IncomingTransaction, &mut HandlerRegistry<STATE>),
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
                let handler_exists = transaction.events.iter().any(|event| {
                    self
                        .handler_registry
                        .handlers
                        .get(&(event.emitter.address().to_string(), event.name.clone()))
                        .is_some()
                });
                if !handler_exists {
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
    processor: TransactionStreamProcessor<
        STREAM,
        fn(&mut STATE, &IncomingTransaction, &mut HandlerRegistry<STATE>),
        STATE,
    >,
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
        let processor = TransactionStreamProcessor::new(
            transaction_stream,
            handler_registry,
            default_transaction_handler
                as fn(
                    &mut STATE,
                    &IncomingTransaction,
                    &mut HandlerRegistry<STATE>,
                ),
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
    app_state: &mut STATE,
    transaction: &IncomingTransaction,
    handler_registry: &mut HandlerRegistry<STATE>,
) {
    transaction
        .handle_events(app_state, handler_registry)
        .unwrap();
}

#[allow(non_camel_case_types)]
impl IncomingTransaction {
    fn event_loop<STATE: Clone>(
        &self,
        app_state: &mut STATE,
        handler_registry: &mut HandlerRegistry<STATE>,
    ) -> Result<(), EventHandlerError> {
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
                            format!("ERROR HANDLING EVENT: {}", event.name)
                                .bright_red()
                        );
                        error!("{}", format!("{:?}", e).bright_red());
                        sleep(std::time::Duration::from_secs(10));
                        info!(
                            "{}",
                            format!(
                                "RETRYING HANDLING EVENT: {}\n",
                                event.name
                            )
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
    pub fn handle_events<STATE>(
        &self,
        app_state: &mut STATE,
        handler_registry: &mut HandlerRegistry<STATE>,
    ) -> Result<(), EventHandlerError>
    where
        STATE: Clone,
    {
        while let Err(err) = self.event_loop(app_state, handler_registry) {
            match err {
                EventHandlerError::TransactionRetryError(err) => {
                    error!(
                        "{}",
                        format!("ERROR HANDLING TRANSACTION: {}\n", err)
                            .bright_red()
                    );
                    std::thread::sleep(std::time::Duration::from_secs(10));
                    info!(
                        "{}",
                        format!("RETRYING HANDLING TRANSACTION")
                            .bright_yellow()
                    );
                    continue;
                }
                _ => {
                    return Err(err);
                }
            }
        }
        Ok(())
    }
}
