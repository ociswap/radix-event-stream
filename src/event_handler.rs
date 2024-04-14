use async_trait::async_trait;
use dyn_clone::DynClone;
// use scrypto::prelude::*;
use std::{any::Any, collections::HashMap};

use crate::{
    error::EventHandlerError,
    models::{Event, Transaction},
};

/// typeErasedHandlerRegistry is a type-erased version of HandlerRegistry.

#[derive(Default)]
pub struct HandlerRegistry {
    pub handlers: HashMap<(String, String), Box<dyn Any + Send + Sync>>,
}

#[allow(non_camel_case_types)]
impl HandlerRegistry {
    pub fn new() -> Self {
        HandlerRegistry {
            handlers: HashMap::new(),
        }
    }

    pub fn handler_exists(&self, emitter: &str, name: &str) -> bool {
        self.handlers
            .contains_key(&(emitter.to_string(), name.to_string()))
    }

    pub fn add_handler<STATE: Clone + 'static, TRANSACTION_CONTEXT: 'static>(
        &mut self,
        emitter: &str,
        name: &str,
        handler: impl EventHandler<STATE, TRANSACTION_CONTEXT> + 'static,
    ) {
        let boxed: Box<dyn EventHandler<STATE, TRANSACTION_CONTEXT> + 'static> =
            Box::new(handler);
        self.handlers
            .insert((emitter.to_string(), name.to_string()), Box::new(boxed));
    }

    pub fn get_handler<STATE: Clone + 'static, TRANSACTION_CONTEXT: 'static>(
        &self,
        emitter: &str,
        name: &str,
    ) -> Option<&Box<dyn EventHandler<STATE, TRANSACTION_CONTEXT>>> {
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
    pub handler_registry: &'a mut HandlerRegistry,
}
