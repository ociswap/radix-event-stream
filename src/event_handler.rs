use async_trait::async_trait;
use dyn_clone::DynClone;
// use scrypto::prelude::*;
use std::collections::HashMap;

use crate::{
    error::EventHandlerError,
    models::{Event, Transaction},
};

/// A registry that stores event handlers.
/// each event handler is identified by the emitter and the event name.
/// As an example, the emitter could be the address of a component ("component_..."), and the
/// event name could be "SwapEvent".
#[allow(non_camel_case_types)]
#[derive(Default, Clone)]
pub struct HandlerRegistry<STATE, TRANSACTION_CONTEXT = ()>
where
    STATE: Clone,
{
    pub handlers: HashMap<
        (String, String),
        Box<dyn EventHandler<STATE, TRANSACTION_CONTEXT>>,
    >,
}

#[allow(non_camel_case_types)]
impl<STATE, TRANSACTION_CONTEXT> HandlerRegistry<STATE, TRANSACTION_CONTEXT>
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
        handler: impl EventHandler<STATE, TRANSACTION_CONTEXT> + 'static,
    ) {
        self.handlers
            .insert((emitter.to_string(), name.to_string()), Box::new(handler));
    }

    pub fn get_handler(
        &self,
        emitter: &str,
        name: &str,
    ) -> Option<&Box<dyn EventHandler<STATE, TRANSACTION_CONTEXT>>> {
        self.handlers.get(&(emitter.to_string(), name.to_string()))
    }
}

/// A trait that abstracts an event handler.
#[allow(non_camel_case_types)]
#[async_trait]
pub trait EventHandler<STATE, TRANSACTION_CONTEXT>:
    DynClone + Send + Sync
where
    STATE: Clone,
{
    async fn handle(
        &self,
        input: EventHandlerContext<'_, STATE, TRANSACTION_CONTEXT>,
        event: Vec<u8>,
    ) -> Result<(), EventHandlerError>;
}

#[allow(non_camel_case_types)]
impl<STATE, TRANSACTION_CONTEXT> Clone
    for Box<dyn EventHandler<STATE, TRANSACTION_CONTEXT>>
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
pub struct EventHandlerContext<'a, STATE, TRANSACTION_CONTEXT = ()>
where
    STATE: Clone,
{
    pub state: &'a mut STATE,
    pub transaction: &'a Transaction,
    pub event: &'a Event,
    pub transaction_context: &'a mut TRANSACTION_CONTEXT,
    pub handler_registry: &'a mut HandlerRegistry<STATE, TRANSACTION_CONTEXT>,
}
