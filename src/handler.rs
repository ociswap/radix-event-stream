use crate::streaming::{Event, Transaction};
use scrypto::prelude::*;
use std::collections::HashMap;

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
#[derive(Default, Debug, Clone)]
pub struct HandlerRegistry<EVENT_HANDLER, STATE>
where
    EVENT_HANDLER: EventHandler<EVENT_HANDLER, STATE>,
    STATE: Clone,
{
    pub handlers: HashMap<String, Vec<EVENT_HANDLER>>,
    _marker: std::marker::PhantomData<STATE>,
}

#[allow(non_camel_case_types)]
impl<EVENT_HANDLER, STATE> HandlerRegistry<EVENT_HANDLER, STATE>
where
    EVENT_HANDLER: EventHandler<EVENT_HANDLER, STATE>,
    STATE: Clone,
{
    pub fn new() -> Self {
        HandlerRegistry {
            handlers: HashMap::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn add_handler(&mut self, emitter: String, handler: EVENT_HANDLER) {
        self.handlers
            .entry(emitter)
            .or_insert_with(Vec::new)
            .push(handler);
    }
}

#[allow(non_camel_case_types)]
pub trait EventHandler<EVENT_HANDLER, STATE>: Clone + Debug
where
    EVENT_HANDLER: EventHandler<EVENT_HANDLER, STATE>,
    STATE: Clone,
{
    fn handle(
        &self,
        app_state: &mut STATE,
        event: &Event,
        transaction: &Transaction,
        handler_registry: &mut HandlerRegistry<EVENT_HANDLER, STATE>,
    );

    // Add this method to the trait
    fn match_variant(&self, name: &str) -> bool;
}

pub trait TransactionHandler {
    fn handle(&self, transaction: Transaction);
}
