use async_trait::async_trait;
use dyn_clone::DynClone;
use scrypto::prelude::*;
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

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
pub struct HandlerRegistry<STATE, TX_CTX>
where
    STATE: Clone,
{
    pub handlers:
        HashMap<(String, String), Box<dyn EventHandler<STATE, TX_CTX>>>,
}

#[allow(non_camel_case_types)]
impl<STATE, TX_CTX> HandlerRegistry<STATE, TX_CTX>
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
        handler: impl EventHandler<STATE, TX_CTX> + 'static,
    ) {
        self.handlers
            .insert((emitter.to_string(), name.to_string()), Box::new(handler));
    }

    pub fn get_handler(
        &self,
        emitter: &str,
        name: &str,
    ) -> Option<&Box<dyn EventHandler<STATE, TX_CTX>>> {
        self.handlers.get(&(emitter.to_string(), name.to_string()))
    }
}

/// A trait that abstracts an event handler.
#[allow(non_camel_case_types)]
#[async_trait]
pub trait EventHandler<STATE, TX_CTX>: DynClone + Send + Sync
where
    STATE: Clone,
{
    async fn handle(
        &self,
        input: EventHandlerContext<'_, STATE, TX_CTX>,
        event: Vec<u8>,
        tx_ctx: Option<&TX_CTX>,
    ) -> Result<(), EventHandlerError>;
}

// Implement EventHandler for all functions that have the correct signature F
#[async_trait]
impl<STATE, TX_CTX, F> EventHandler<STATE, TX_CTX> for F
where
    F: Fn(
            EventHandlerContext<STATE, TX_CTX>,
            Vec<u8>,
            Option<&TX_CTX>,
        ) -> Pin<
            Box<
                dyn Future<Output = Result<(), EventHandlerError>>
                    + Send
                    + 'static,
            >,
        >
        + Send
        + Sync
        + 'static
        + DynClone,
    STATE: Clone + Send + Sync + 'static,
    TX_CTX: Send + Sync + 'static,
{
    async fn handle(
        &self,
        input: EventHandlerContext<'_, STATE, TX_CTX>,
        event: Vec<u8>,
        tx_ctx: Option<&TX_CTX>,
    ) -> Result<(), EventHandlerError> {
        self(input, event, tx_ctx).await
    }
}

impl<STATE, TX_CTX> Clone for Box<dyn EventHandler<STATE, TX_CTX>>
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
pub struct EventHandlerContext<'a, STATE, TX_CTX>
where
    STATE: Clone,
{
    pub app_state: &'a mut STATE,
    pub transaction: &'a IncomingTransaction,
    pub event: &'a IncomingEvent,
    pub handler_registry: &'a Arc<Mutex<HandlerRegistry<STATE, TX_CTX>>>,
}
