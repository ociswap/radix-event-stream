use async_trait::async_trait;
use dyn_clone::DynClone;
// use scrypto::prelude::*;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::{
    error::EventHandlerError,
    models::{Event, Transaction},
};

/// A type-erased registry of event handlers. It is not parametrized by the
/// state and transaction context types, which is a nice property
/// that allows some other types to be a bit simpler.
pub struct HandlerRegistry {
    pub handlers: HashMap<(String, String), Box<dyn Any + Send + Sync>>,
    pub type_id: Option<TypeId>,
}

#[allow(non_camel_case_types)]
impl HandlerRegistry {
    pub fn new() -> Self {
        HandlerRegistry {
            handlers: HashMap::new(),
            type_id: None,
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

    pub fn get_handler<STATE: Clone + 'static, TRANSACTION_CONTEXT: 'static>(
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
