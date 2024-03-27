use async_trait::async_trait;
use dyn_clone::DynClone;
use scrypto::prelude::*;
use std::collections::HashMap;

use crate::{
    error::EventHandlerError,
    models::{EventHandlerContext, IncomingTransaction},
};

/// A registry that stores event handlers.
#[allow(non_camel_case_types)]
#[derive(Default, Clone)]
pub struct HandlerRegistry<STATE>
where
    STATE: AppState,
{
    pub handlers: HashMap<(String, String), Box<dyn EventHandler<STATE>>>,
}

#[allow(non_camel_case_types)]
impl<STATE> HandlerRegistry<STATE>
where
    STATE: AppState,
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
#[async_trait]
pub trait EventHandler<STATE>: DynClone + Send + Sync
where
    STATE: AppState,
{
    async fn handle(
        &self,
        input: EventHandlerContext<'async_trait, STATE>,
        event: Vec<u8>,
    ) -> Result<(), EventHandlerError>;
}

// Implement EventHandler for all functions that have the correct signature F
#[async_trait]
impl<STATE, F> EventHandler<STATE> for F
where
    F: Fn(EventHandlerContext<STATE>, Vec<u8>) -> Result<(), EventHandlerError>
        + Send
        + Sync
        + 'static
        + Clone,
    STATE: AppState,
{
    async fn handle(
        &self,
        input: EventHandlerContext<'async_trait, STATE>,
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

pub trait AppState: Clone + Send + Sync {}
impl<T> AppState for T where T: Clone + Send + Sync {}
