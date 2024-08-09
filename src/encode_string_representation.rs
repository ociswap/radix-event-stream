//! Some code copied from https://github.com/radixdlt/radix-engine-toolkit
//! I cannot include all of radix-engine-toolkit as a dependency because
//! it uses some experimental features, and I want to keep supporting
//! stable Rust.

use radix_common::prelude::scrypto_encode;
use sbor::{DecodeError, EncodeError};
use sbor_json::scrypto::programmatic::{
    utils::value_contains_network_mismatch, value::ProgrammaticScryptoValue,
};

#[derive(Debug, Clone)]
pub enum StringRepresentation {
    ProgrammaticJson(String),
}

#[derive(Debug)]
pub enum ScryptoSborError {
    SchemaValidationError,
    DecodeError(DecodeError),
    EncodeError(EncodeError),
    SerdeDeserializationFailed(serde_json::Error),
    ValueContainsNetworkMismatch,
}

pub fn encode_string_representation(
    representation: StringRepresentation,
) -> Result<Vec<u8>, ScryptoSborError> {
    match representation {
        StringRepresentation::ProgrammaticJson(value) => {
            let value =
                serde_json::from_str::<ProgrammaticScryptoValue>(&value)
                    .map_err(ScryptoSborError::SerdeDeserializationFailed)?;
            if value_contains_network_mismatch(&value) {
                return Err(ScryptoSborError::ValueContainsNetworkMismatch);
            }

            let value = value.to_scrypto_value();
            scrypto_encode(&value).map_err(ScryptoSborError::EncodeError)
        }
    }
}
