/*!
The interface for a [`TransactionHandler`]

Transaction handlers are responsible for processing transactions
and calling event handlers to process the events
inside the transaction.
The transaction handler is only called when a transaction actually has events
that have an event handler associated with them.

We don't generally have to create a separate struct
and implement this trait for it manually, because we can use
the `#[transaction_handler]` macro to generate the
struct and implementation for us. It allows us to write
the handler as an async function, which is a bit more
ergonomic.
To use this macro, the handler function must conform to a predefined
signature.
A transaction handler function must:
- Be an async function
- Take a single argument of type `TransactionHandlerContext<YOUR_STATE>`
- Return a `Result<(), TransactionHandlerError>`

You can use the following template to create a transaction handler:
```ignore
#[transaction_handler]
// Function name is the name of your handler
async fn transaction_handler_name(
    // Context the handler will get from the framework.
    // This includes the current ledger transaction we're in
    // and the global state. It is parametrized by the
    // app state and the transaction context type, but the context is optional,
    // and defaults to the unit type.
    context: TransactionHandlerContext<YOUR_STATE>,
) -> Result<(), TransactionHandlerError> {
    // Do something like start a database transaction
    let mut transaction_context = TransactionContext { tx: start_transaction() }
    // Handle the events inside the incoming transaction.
    // We provide a simple method for this.
    context
        .event_processor
        .process_events(
            context.state,
            context.handler_registry,
            // the transaction context is passed in
            &mut transaction_context,
        )
        // EventHandlerErrors can be cast into TransactionHandlerErrors,
        // and the framework will handle these appropriately.
        // So, best to propagate these with the ? operator.
        .await?;
    // Possible errors to return:
    // Retry handling the current transaction
    return Err(EventHandlerError::TransactionRetryError(
        anyhow!("Retry transaction because of...")
    ));
    // Stop the stream
    return Err(EventHandlerError::UnrecoverableError(
        anyhow!("Stream failed because of...")
    ));
    // Everything's ok!
    Ok(())
}
```
Now, we can simply pass in the handler to the [`TransactionStreamProcessor`][crate::processor::TransactionStreamProcessor].
It is now secretly a struct that implements the [`TransactionHandler`] trait.
*/

/// A trait that defines a transaction handler.
/// A transaction handler is responsible for
/// calling event handlers to process the events
/// in a transaction, and potentially doing
/// database transactions or other atomic operations at the
/// transaction level.
use crate::{
    error::TransactionHandlerError, event_handler::HandlerRegistry,
    models::Transaction, processor::EventProcessor,
};
use async_trait::async_trait;

#[allow(non_camel_case_types)]
#[async_trait]
pub trait TransactionHandler<STATE>: 'static {
    async fn handle(
        &self,
        input: TransactionHandlerContext<'_, STATE>,
    ) -> Result<(), TransactionHandlerError>;
}

#[allow(non_camel_case_types)]
/// A struct that holds the context for a transaction handler,
/// which is passed to the handler when it is called.
pub struct TransactionHandlerContext<'a, STATE> {
    pub state: &'a mut STATE,
    pub transaction: &'a Transaction,
    pub event_processor: &'a mut EventProcessor<'a>,
    pub handler_registry: &'a mut HandlerRegistry,
}
