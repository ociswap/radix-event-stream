//! Some code copied from https://github.com/radixdlt/radix-engine-toolkit
//! I cannot include all of radix-engine-toolkit as a dependency because
//! it uses some experimental features, and I want to keep supporting
//! stable Rust.

use sbor::{DecodeError, EncodeError};
use sbor_json::scrypto::programmatic::{
    utils::value_contains_network_mismatch, value::ProgrammaticScryptoValue,
};
// Import the git versions with aliases
use radix_common_git;
use sbor_git;

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

// Convert from git version's EncodeError to crates.io version
fn convert_encode_error(error: sbor_git::EncodeError) -> EncodeError {
    match error {
        sbor_git::EncodeError::MaxDepthExceeded(depth) => 
            EncodeError::MaxDepthExceeded(depth),
        sbor_git::EncodeError::SizeTooLarge { actual, max_allowed } => 
            EncodeError::SizeTooLarge { actual, max_allowed },
        sbor_git::EncodeError::MismatchingArrayElementValueKind { element_value_kind, actual_value_kind } => 
            EncodeError::MismatchingArrayElementValueKind { element_value_kind, actual_value_kind },
        sbor_git::EncodeError::MismatchingMapKeyValueKind { key_value_kind, actual_value_kind } => 
            EncodeError::MismatchingMapKeyValueKind { key_value_kind, actual_value_kind },
        sbor_git::EncodeError::MismatchingMapValueValueKind { value_value_kind, actual_value_kind } => 
            EncodeError::MismatchingMapValueValueKind { value_value_kind, actual_value_kind },
    }
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

            // This returns a Value from the git version
            let scrypto_value = value.to_scrypto_value();
            
            // Use the git version's encoder directly
            radix_common_git::data::scrypto::scrypto_encode(&scrypto_value)
                .map_err(|e| ScryptoSborError::EncodeError(convert_encode_error(e)))
        }
    }
}
