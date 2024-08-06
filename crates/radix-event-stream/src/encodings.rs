//! Some utility functions for encoding and decoding data
//! using the Scrypto SBOR encoding.

use crate::encode_string_representation::{
    encode_string_representation, StringRepresentation,
};
use radix_common::{
    address::{AddressBech32EncodeError, AddressBech32Encoder},
    data::scrypto::{scrypto_decode, ScryptoDecode},
    network::NetworkDefinition,
};

/// Decode a [`serde_json::Value`] containing programmatic json
/// into a type that implements the [`ScryptoDecode`] trait.
#[allow(clippy::redundant_closure)]
pub fn decode_programmatic_json<T: ScryptoDecode>(
    data: &serde_json::Value,
) -> anyhow::Result<T> {
    let bytes = programmatic_json_to_bytes(data)?;
    scrypto_decode::<T>(&bytes)
        .map_err(|err| anyhow::anyhow!("Could not decode: {:#?}", err))
}

/// Some representations of transactions only come with programmatic
/// json data. This function converts a [`serde_json::Value`] containing
/// programmatic json into a binary SBOR representation.
pub fn programmatic_json_to_bytes(
    data: &serde_json::Value,
) -> anyhow::Result<Vec<u8>> {
    let string_data = data.to_string();
    let string_representation = encode_string_representation(
        StringRepresentation::ProgrammaticJson(string_data),
    )
    .map_err(|err| anyhow::anyhow!("Could not encode: {:#?}", err))?;
    Ok(string_representation.to_vec())
}

/// Encode a byte slice into a bech32m string representation.
/// Useful for easily encoding addresses to the proper network
/// string format.
pub fn encode_bech32m(
    data: &[u8],
    network: &NetworkDefinition,
) -> Result<String, AddressBech32EncodeError> {
    AddressBech32Encoder::new(network).encode(data)
}
