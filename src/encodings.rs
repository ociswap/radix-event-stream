use radix_engine_toolkit::functions::scrypto_sbor::{
    encode, encode_string_representation, ScryptoSborError,
    StringRepresentation,
};

use scrypto::{
    address::AddressBech32Encoder,
    data::scrypto::{scrypto_decode, ScryptoDecode},
    network::NetworkDefinition,
};

/// Decode a serde json value containing programmatic json
/// into a type that implements the `ScryptoDecode` trait.
#[allow(clippy::redundant_closure)]
pub fn decode_programmatic_json<T: ScryptoDecode>(
    data: &serde_json::Value,
) -> Result<T, ScryptoSborError> {
    let bytes = programmatic_json_to_bytes(data)?;
    scrypto_decode::<T>(&bytes)
        .map_err(|error| ScryptoSborError::DecodeError(error))
}

pub fn programmatic_json_to_bytes(
    data: &serde_json::Value,
) -> Result<Vec<u8>, ScryptoSborError> {
    let string_data = data.to_string();
    let string_representation = encode_string_representation(
        StringRepresentation::ProgrammaticJson(string_data),
    )?;
    Ok(string_representation.to_vec())
}

pub fn encode_bech32(
    data: &[u8],
    network: &NetworkDefinition,
) -> Option<String> {
    AddressBech32Encoder::new(network).encode(data).ok()
}
