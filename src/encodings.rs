use radix_engine_common::{
    address::{AddressBech32EncodeError, AddressBech32Encoder},
    data::scrypto::{scrypto_decode, ScryptoDecode},
    network::NetworkDefinition,
};
use radix_engine_toolkit::functions::scrypto_sbor::{
    encode_string_representation, ScryptoSborError, StringRepresentation,
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

/// Some representations of transactions only come with programmatic
/// json data. This function converts the programmatic json data into
/// binary SBOR representation.
pub fn programmatic_json_to_bytes(
    data: &serde_json::Value,
) -> Result<Vec<u8>, ScryptoSborError> {
    let string_data = data.to_string();
    let string_representation = encode_string_representation(
        StringRepresentation::ProgrammaticJson(string_data),
    )?;
    Ok(string_representation.to_vec())
}

/// Encode a byte slice into a bech32m string representation.
/// Useful for easily encoding addresses to the proper network
/// string format.e
pub fn encode_bech32m(
    data: &[u8],
    network: &NetworkDefinition,
) -> Result<String, AddressBech32EncodeError> {
    AddressBech32Encoder::new(network).encode(data)
}
