use dyn_clone::DynClone;
use scrypto::prelude::*;
use std::collections::HashMap;

use crate::models::{EventHandlerInput, Transaction};

/// A registry of handlers that can be used to decode events
/// coming from the Radix Gateway. You can register your own
/// handlers using the `add_decoder` method. Each handler
/// is a trait object that implements the `EventHandler` trait.
/// Typicaly you would create a new decoder type per event type.
/// The `handle` method will call the `handle` method on each
/// handler in the registry. If correctly set up,
/// only up to one handler should actually be able to identify
/// and process an event.
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
    // EVENT_HANDLER: EventHandler<STATE>,
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
    fn handle(&self, input: EventHandlerInput<STATE>, event: Vec<u8>);
}

// Implement EventHandler for all functions that have the correct signature F
impl<STATE, F> EventHandler<STATE> for F
where
    F: Fn(EventHandlerInput<STATE>, Vec<u8>) + Clone,
    STATE: Clone,
{
    fn handle(&self, input: EventHandlerInput<STATE>, event: Vec<u8>) {
        self(input, event);
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

pub trait TransactionHandler {
    fn handle(&self, transaction: Transaction);
}
