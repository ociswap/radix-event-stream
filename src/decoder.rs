use radix_client::gateway::models::Event;
use radix_engine_toolkit::functions::scrypto_sbor::{
    encode_string_representation, ScryptoSborError, StringRepresentation,
};
use scrypto::prelude::*;

use crate::eventstream::{DecodableEvent, Transaction};
use std::{error::Error, fmt::Debug};

/// A trait that defines a decoder for for an event type.
/// To be able to decode an event, create a struct `MyEventDecoder`
/// that implements this trait. The `decode` method should return
/// the decoded event as a `Box<dyn ProcessableEvent>`. This
/// allows you to return different event types from the `decode`
/// method. The `ProcessableEvent` trait is implemented by all
/// event types.
// Adjusted EventProcessor trait to return Box<dyn ProcessableEvent>
pub trait EventHandler<E, T>
where
    E: DecodableEvent,
    T: Transaction<E>,
{
    fn identify(&self, event: &E) -> bool;
    fn process(&self, event: &E, transaction: &T)
        -> Result<(), Box<dyn Error>>;
    fn handle(&self, event: &E, transaction: &T) -> Result<(), Box<dyn Error>> {
        if self.identify(event) {
            self.process(event, transaction)
        } else {
            Ok(())
        }
    }
}

/// A registry of decoders that can be used to decode events
/// coming from the Radix Gateway. You can register your own
/// decoders using the `add_decoder` method. Each decoder
/// is a trait object that implements the `EventDecoder` trait.
/// Typicaly you would create a new decoder type per event type.
pub struct HandlerRegistry<E, T>
where
    E: DecodableEvent,
    T: Transaction<E>,
{
    pub handlers: Vec<Box<dyn EventHandler<E, T>>>,
}

impl<E, T> HandlerRegistry<E, T>
where
    E: DecodableEvent,
    T: Transaction<E>,
{
    pub fn new() -> Self {
        HandlerRegistry {
            handlers: Vec::new(),
        }
    }

    pub fn add_handler(&mut self, decoder: Box<dyn EventHandler<E, T>>) {
        self.handlers.push(decoder);
    }

    pub fn handle(
        &self,
        transaction: &T,
        event: &E,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for handler in &self.handlers {
            handler.handle(event, transaction)?;
        }
        Ok(())
    }
}

/// Decode a serde json value containing programmatic json
/// into a type that implements the `ScryptoDecode` trait.
pub fn decode_programmatic_json<T: ScryptoDecode>(
    data: &serde_json::Value,
) -> Result<T, ScryptoSborError> {
    let start_decode = std::time::Instant::now();
    let string_data = data.to_string();
    let string_representation = match encode_string_representation(
        StringRepresentation::ProgrammaticJson(string_data),
    ) {
        Ok(string_representation) => string_representation,
        Err(error) => return Err(error),
    };
    let decoded = scrypto_decode::<T>(string_representation.as_slice())
        .map_err(|error| ScryptoSborError::DecodeError(error));
    println!("decode_programmatic_json took {:?}", start_decode.elapsed());
    decoded
}

pub fn encode_bech32(
    data: &[u8],
    network: &NetworkDefinition,
) -> Option<String> {
    AddressBech32Encoder::new(network).encode(&data).ok()
}
