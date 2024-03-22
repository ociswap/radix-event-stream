use crate::streaming::{Event, Transaction};
use scrypto::prelude::*;
use std::error::Error;

/// A trait that defines a handler for for an event type.
/// To be able to handle an event, there are two
/// main things we must do. We must first identify the event,
/// making sure it is the type of event we are interested in,
/// and making sure it is relevant. Then, we may want to process
/// the event or do some kind of task. Use this Trait to
/// implement both the identification step and the processing
/// step. Leave the handle method as is, as it will call
/// identify and process for you.
#[allow(clippy::borrowed_box)]
pub trait EventHandler: Debug + CloneBox {
    /// Implement this by checking if the event is the type
    /// of event you are interested in. For example, do this
    /// by checking the event name. Note that anyone can create
    /// an event with any name, so you should also check the
    /// emitter of the event to make sure it is legitimate.
    /// For this, you can store application state in the struct
    /// that implements this trait and access it here.
    /// If the event is the type you are interested in, decode
    /// the event using the `decode_programmatic_json` function
    /// and return it Boxed.
    fn identify(&self, event: &Box<dyn Event>) -> Option<Box<dyn Debug>>;
    /// Implement this by processing the event. This is where
    /// you would do any kind of task you want to do with the
    /// event. For example, you could update a database, or
    /// send a message to a queue. You can also call other
    /// services or APIs. You can also do nothing,
    /// to ignore the event. Note again that you can access
    /// application state by storing it in the struct that
    /// implements this trait and passing that into the handler
    /// when you register it.
    fn process(
        &self,
        event: &Box<dyn Event>,
        transaction: &Box<dyn Transaction>,
    ) -> Result<(), Box<dyn Error>>;
    /// This method will call identify and process for you.
    /// Most often you will not need to override this method.
    fn handle(
        &self,
        event: &Box<dyn Event>,
        transaction: &Box<dyn Transaction>,
    ) -> Result<(), Box<dyn Error>> {
        let identified = self.identify(event);
        match identified {
            Some(identified) => {
                log::info!("{:#?}", identified);
                self.process(event, transaction)
            }
            None => Ok(()),
        }
    }
}

/// A registry of handlers that can be used to decode events
/// coming from the Radix Gateway. You can register your own
/// handlers using the `add_decoder` method. Each handler
/// is a trait object that implements the `EventHandler` trait.
/// Typicaly you would create a new decoder type per event type.
/// The `handle` method will call the `handle` method on each
/// handler in the registry. If correctly set up,
/// only up to one handler should actually be able to identify
/// and process an event.
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

    #[allow(clippy::borrowed_box)]
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

pub trait CloneBox {
    fn clone_box(&self) -> Box<dyn EventHandler>;
}

impl<T> CloneBox for T
where
    T: 'static + EventHandler + Clone,
{
    fn clone_box(&self) -> Box<dyn EventHandler> {
        Box::new(self.clone())
    }
}

impl Clone for HandlerRegistry {
    fn clone(&self) -> Self {
        HandlerRegistry {
            handlers: self.handlers.iter().map(|h| h.clone_box()).collect(),
        }
    }
}
