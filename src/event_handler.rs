/*!
The interface for an [`EventHandler`]

Event handlers are responsible for processing single event types.
We don't generally have to create a separate struct
and implement this trait for it manually, because we can use
the `#[event_handler]` macro to generate the
struct and implementation for us. It allows us to write
the handler as an async function, which is a bit more
ergonomic.

To use this macro, the handler function must conform to a predefined
signature.
An event handler function must:
- Be an async function
- Take a `context` parameter of type [`EventHandlerContext<YOUR_STATE>`]
- Take an `event` parameter of the type of your Radix Engine event struct which derives [`radix_engine_common::ScryptoSbor`]
- Return a `Result<(), EventHandlerError>`

You can use the following template to create an event handler:

```ignore
// A macro from the crate which transforms the handler function
// into a representation that is usable for the framework.
#[event_handler]
// The function name is the name of your handler
async fn event_handler_name(
    // Context the handler will get from the framework.
    // This includes the current ledger transaction we're in,
    // the raw event, the global state, and the transaction context.
    context: EventHandlerContext<YOUR_STATE>,
    // The decoded event struct as defined in your smart contract.
    event: EVENT_STRUCT,
) -> Result<(), EventHandlerError> {
    // Handle the event here.

    // Possible errors to return:
    // Retry handling the current event
    return Err(EventHandlerError::EventRetryError(
        anyhow!("Retry event because of...")
    ));
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

*/

use async_trait::async_trait;
use dyn_clone::DynClone;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::{
    error::EventHandlerError,
    models::{Event, Transaction},
};

/// A shorthand trait for a state type that can be used in event handlers.
/// It's used to enforce that the state type is Send + Sync + 'static without having
/// to write it out every time.
pub trait State: Send + Sync + 'static {}

// Implement the State trait for all types that are Send + Sync + 'static.
impl<T> State for T where T: Send + Sync + 'static {}

/// A type-erased registry of event handlers. It is not parametrized by the
/// state and transaction context types, which is a nice property
/// that allows some other types to be a bit simpler.
/// It can only contain event handlers of one specific type, which is
/// implicitly determined by the first handler that is added to the registry.
#[derive(Default)]
pub struct HandlerRegistry {
    handlers: HashMap<(String, String), Box<dyn Any + Send + Sync>>,
    type_id: Option<TypeId>,
}

#[allow(non_camel_case_types)]
impl HandlerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handler_exists(&self, emitter: &str, name: &str) -> bool {
        self.handlers
            .contains_key(&(emitter.to_string(), name.to_string()))
    }

    /// Add an event handler to the registry.
    /// It is only possible to add handlers with the same signature.
    /// The signature is determined by the first handler that is added to the registry.
    ///
    /// # Panics
    ///
    /// Panics if the added handler has a different signature than the
    /// handlers already in the registry.
    pub fn add_handler<STATE: State, TRANSACTION_CONTEXT: 'static>(
        &mut self,
        emitter: &str,
        name: &str,
        handler: impl EventHandler<STATE, TRANSACTION_CONTEXT> + 'static,
    ) {
        // Get the type ID of the handler
        let type_id =
            TypeId::of::<Box<dyn EventHandler<STATE, TRANSACTION_CONTEXT>>>();
        match self.type_id {
            // If there is already a type ID, we check if it matches the handler
            // we're trying to add.
            Some(existing_type_id) => {
                if existing_type_id != type_id {
                    panic!("HandlerRegistry already contains a handler with a different signature");
                }
            }
            // If there is no type ID yet, we implicitly set it here.
            None => {
                self.type_id = Some(type_id);
            }
        }
        // Box the handler and insert it into the registry.
        let boxed: Box<dyn EventHandler<STATE, TRANSACTION_CONTEXT> + 'static> =
            Box::new(handler);
        self.handlers
            .insert((emitter.to_string(), name.to_string()), Box::new(boxed));
    }

    /// Get an event handler from the registry.
    /// The handler is downcast to the correct type.
    ///
    /// # Panics
    ///
    /// This function panics if the type parameters used to call it
    /// don't match the ones used to add the handler to the registry.
    #[allow(clippy::borrowed_box)]
    pub fn get_handler<STATE: State, TRANSACTION_CONTEXT: 'static>(
        &self,
        emitter: &str,
        name: &str,
    ) -> Option<&Box<dyn EventHandler<STATE, TRANSACTION_CONTEXT>>> {
        // Get the type id of the handler we're trying to get.
        let type_id =
            TypeId::of::<Box<dyn EventHandler<STATE, TRANSACTION_CONTEXT>>>();

        // Check if the type ID matches the ones stored in the registry.
        // If they don't match, we can't downcast the handler and there must be a bug somewhere.
        if self.type_id != Some(type_id) {
            panic!("Trying to get handler with different signature than the ones stored in the registry");
        }

        // Get the handler from the registry and downcast it to the correct type.
        let handler =
            self.handlers.get(&(emitter.to_string(), name.to_string()));
        handler.map(|handler| {
            handler
                .downcast_ref::<Box<dyn EventHandler<STATE, TRANSACTION_CONTEXT>>>()
                .expect("Failed to downcast handler")
        })
    }
}

/// A trait that abstracts an event handler.
#[allow(non_camel_case_types)]
#[async_trait]
pub trait EventHandler<STATE, TRANSACTION_CONTEXT>:
    DynClone + Send + Sync
{
    async fn handle(
        &self,
        input: EventHandlerContext<'_, STATE, TRANSACTION_CONTEXT>,
        event: &[u8],
    ) -> Result<(), EventHandlerError>;
}

#[allow(non_camel_case_types)]
impl<STATE, TRANSACTION_CONTEXT> Clone
    for Box<dyn EventHandler<STATE, TRANSACTION_CONTEXT>>
{
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

/// A struct that holds the context for an event handler,
/// which is passed to the handler when it is called.
///
/// STATE: The global state of the application.
/// TRANSACTION_CONTEXT: A type containing context of a current transaction, like
/// a database transaction handle. This is optional and defaults to the unit type.
#[allow(non_camel_case_types)]
pub struct EventHandlerContext<'a, STATE, TRANSACTION_CONTEXT = ()> {
    /// The global state.
    pub state: &'a mut STATE,
    /// Raw transaction data coming from ledger.
    pub transaction: &'a Transaction,
    /// Raw event data coming from ledger.
    pub event: &'a Event,
    /// Zero-based index of the event in the transaction.
    pub event_index: u16,
    /// Context of the current transaction, like a database transaction handle.
    pub transaction_context: &'a mut TRANSACTION_CONTEXT,
    /// Handler registry of event handlers.
    pub handler_registry: &'a mut HandlerRegistry,
}

#[cfg(test)]
mod tests {

    use radix_engine_common::ScryptoSbor;

    use crate::macros::event_handler;

    use super::*;
    use crate as radix_event_stream;

    struct TestState;
    struct TestState2;

    #[derive(ScryptoSbor)]
    struct TestEventType;

    #[derive(ScryptoSbor)]
    struct TestEventType2;

    #[event_handler]
    async fn test_handler(
        context: EventHandlerContext<TestState>,
        event: TestEventType,
    ) -> Result<(), EventHandlerError> {
        Ok(())
    }

    #[event_handler]
    async fn test_handler2(
        context: EventHandlerContext<TestState2>,
        event: TestEventType2,
    ) -> Result<(), EventHandlerError> {
        Ok(())
    }

    #[test]
    fn add_and_retrieve_handler() {
        let mut registry = HandlerRegistry::new();
        registry.add_handler("1", "1", test_handler);

        let handler = registry.get_handler::<TestState, ()>("1", "1");
        assert!(handler.is_some());
    }

    #[test]
    #[should_panic]
    fn add_handler_of_different_type() {
        let mut registry = HandlerRegistry::new();
        registry.add_handler("1", "1", test_handler);

        registry.add_handler("1", "2", test_handler2);
    }

    #[test]
    #[should_panic]
    fn get_handler_as_different_type() {
        let mut registry = HandlerRegistry::new();
        registry.add_handler("1", "1", test_handler);

        registry.get_handler::<TestState2, ()>("1", "1");
    }

    #[test]
    fn test_handler_exists() {
        let mut registry = HandlerRegistry::new();
        registry.add_handler("1", "1", test_handler);

        assert!(registry.handler_exists("1", "1"));
        assert!(!registry.handler_exists("1", "2"));
    }
}
