use crate::streaming::{Event, Transaction};
use scrypto::prelude::*;
use std::error::Error;

/// A trait that defines a decoder for for an event type.
/// To be able to decode an event, create a struct `MyEventDecoder`
/// that implements this trait. The `decode` method should return
/// the decoded event as a `Box<dyn ProcessableEvent>`. This
/// allows you to return different event types from the `decode`
/// method. The `ProcessableEvent` trait is implemented by all
/// event types.
// Adjusted EventProcessor trait to return Box<dyn ProcessableEvent>
pub trait EventHandler: Debug {
    fn identify(&self, event: &Box<dyn Event>) -> Option<Box<dyn Debug>>;
    fn process(
        &self,
        event: &Box<dyn Event>,
        transaction: &Box<dyn Transaction>,
    ) -> Result<(), Box<dyn Error>>;
    fn handle(
        &self,
        event: &Box<dyn Event>,
        transaction: &Box<dyn Transaction>,
    ) -> Result<(), Box<dyn Error>> {
        let identified = self.identify(event);
        match identified {
            Some(identified) => {
                log::info!("{:#?}", identified);
                let processed = self.process(event, transaction);
                processed
            }
            None => return Ok(()),
        }
    }
}

/// A registry of decoders that can be used to decode events
/// coming from the Radix Gateway. You can register your own
/// decoders using the `add_decoder` method. Each decoder
/// is a trait object that implements the `EventDecoder` trait.
/// Typicaly you would create a new decoder type per event type.
#[derive(Default, Debug)]
pub struct HandlerRegistry {
    pub handlers: Vec<Box<dyn EventHandler>>,
}

impl HandlerRegistry {
    pub fn new() -> Self {
        HandlerRegistry {
            handlers: Vec::new(),
        }
    }

    pub fn add_handler(&mut self, decoder: Box<dyn EventHandler>) {
        self.handlers.push(decoder);
    }

    pub fn handle(
        &self,
        transaction: &Box<dyn Transaction>,
        event: &Box<dyn Event>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for handler in &self.handlers {
            handler.handle(event, transaction)?;
        }
        Ok(())
    }
}
