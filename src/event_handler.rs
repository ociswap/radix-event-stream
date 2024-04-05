use dyn_clone::DynClone;
use scrypto::prelude::*;
use std::collections::HashMap;

use crate::{
    error::EventHandlerError,
    models::{IncomingEvent, IncomingTransaction},
};

/// A registry that stores event handlers.
/// each event handler is identified by the emitter and the event name.
/// As an example, the emitter could be the address of a component ("component_..."), and the
/// event name could be "SwapEvent".
#[allow(non_camel_case_types)]
#[derive(Default, Clone)]
pub struct HandlerRegistry<STATE, TRANSACTION_HANDLE = ()>
where
    STATE: Clone,
{
    pub handlers: HashMap<
        (String, String),
        Box<dyn EventHandler<STATE, TRANSACTION_HANDLE>>,
    >,
}

#[allow(non_camel_case_types)]
impl<STATE, TRANSACTION_HANDLE> HandlerRegistry<STATE, TRANSACTION_HANDLE>
where
    STATE: Clone,
{
    pub fn new() -> Self {
        HandlerRegistry {
            handlers: HashMap::new(),
        }
    }

    pub fn add_handler(
        &mut self,
        emitter: &str,
        name: &str,
        handler: impl EventHandler<STATE, TRANSACTION_HANDLE> + 'static,
    ) {
        self.handlers
            .insert((emitter.to_string(), name.to_string()), Box::new(handler));
    }

    pub fn get_handler(
        &self,
        emitter: &str,
        name: &str,
    ) -> Option<&Box<dyn EventHandler<STATE, TRANSACTION_HANDLE>>> {
        self.handlers.get(&(emitter.to_string(), name.to_string()))
    }
}

/// A trait that abstracts an event handler.
#[allow(non_camel_case_types)]
pub trait EventHandler<STATE, TRANSACTION_HANDLE>: DynClone
where
    STATE: Clone,
{
    fn handle(
        &self,
        input: EventHandlerContext<STATE, TRANSACTION_HANDLE>,
        event: Vec<u8>,
    ) -> Result<(), EventHandlerError>;
}

// Implement EventHandler for all functions that have the correct signature F
#[allow(non_camel_case_types)]
impl<STATE, TRANSACTION_HANDLE, F> EventHandler<STATE, TRANSACTION_HANDLE> for F
where
    F: Fn(
            EventHandlerContext<STATE, TRANSACTION_HANDLE>,
            Vec<u8>,
        ) -> Result<(), EventHandlerError>
        + Clone,
    STATE: Clone,
{
    fn handle(
        &self,
        input: EventHandlerContext<STATE, TRANSACTION_HANDLE>,
        event: Vec<u8>,
    ) -> Result<(), EventHandlerError> {
        self(input, event)
    }
}

#[allow(non_camel_case_types)]
impl<STATE, TRANSACTION_HANDLE> Clone
    for Box<dyn EventHandler<STATE, TRANSACTION_HANDLE>>
where
    STATE: Clone,
{
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

/// A struct that holds the context for an event handler,
/// which is passed to the handler when it is called.
#[allow(non_camel_case_types)]
pub struct EventHandlerContext<'a, STATE, TRANSACTION_HANDLE = ()>
where
    STATE: Clone,
{
    pub app_state: &'a mut STATE,
    pub transaction: &'a IncomingTransaction,
    pub transaction_handle: &'a mut TRANSACTION_HANDLE,
    pub event: &'a IncomingEvent,
    pub handler_registry: &'a mut HandlerRegistry<STATE, TRANSACTION_HANDLE>,
}
