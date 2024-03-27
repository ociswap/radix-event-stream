use dyn_clone::DynClone;
use scrypto::prelude::*;
use std::collections::HashMap;

use crate::{
    error::EventHandlerError,
    models::{IncomingEvent, IncomingTransaction},
};

/// A registry that stores event handlers.
#[allow(non_camel_case_types)]
#[derive(Default, Clone)]
pub struct HandlerRegistry<STATE>
where
    STATE: Clone,
{
    pub handlers: HashMap<(String, String), Box<dyn EventHandler<STATE>>>,
}

#[allow(non_camel_case_types)]
impl<STATE> HandlerRegistry<STATE>
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
        handler: impl EventHandler<STATE> + 'static,
    ) {
        self.handlers
            .insert((emitter.to_string(), name.to_string()), Box::new(handler));
    }
}

#[allow(non_camel_case_types)]
pub trait EventHandler<STATE>: DynClone
where
    STATE: Clone,
{
    fn handle(
        &self,
        input: EventHandlerContext<STATE>,
        event: Vec<u8>,
    ) -> Result<(), EventHandlerError>;
}

// Implement EventHandler for all functions that have the correct signature F
impl<STATE, F> EventHandler<STATE> for F
where
    F: Fn(EventHandlerContext<STATE>, Vec<u8>) -> Result<(), EventHandlerError>
        + Clone,
    STATE: Clone,
{
    fn handle(
        &self,
        input: EventHandlerContext<STATE>,
        event: Vec<u8>,
    ) -> Result<(), EventHandlerError> {
        self(input, event)
    }
}

impl<STATE> Clone for Box<dyn EventHandler<STATE>>
where
    STATE: Clone,
{
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

#[allow(non_camel_case_types)]
pub struct EventHandlerContext<'a, STATE>
where
    STATE: Clone,
{
    pub app_state: &'a mut STATE,
    pub transaction: &'a IncomingTransaction,
    pub event: &'a IncomingEvent,
    pub handler_registry: &'a mut HandlerRegistry<STATE>,
}
