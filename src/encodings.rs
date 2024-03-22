use radix_engine_toolkit::functions::scrypto_sbor::{
    encode_string_representation, ScryptoSborError, StringRepresentation,
};
use scrypto::{
    address::AddressBech32Encoder,
    data::scrypto::{scrypto_decode, ScryptoDecode},
    network::NetworkDefinition,
};

/// Decode a serde json value containing programmatic json
/// into a type that implements the `ScryptoDecode` trait.
pub fn decode_programmatic_json<T: ScryptoDecode>(
    data: &serde_json::Value,
) -> Result<T, ScryptoSborError> {
    let string_data = data.to_string();
    let string_representation = match encode_string_representation(
        StringRepresentation::ProgrammaticJson(string_data),
    ) {
        Ok(string_representation) => string_representation,
        Err(error) => return Err(error),
    };
    let decoded = scrypto_decode::<T>(string_representation.as_slice())
        .map_err(|error| ScryptoSborError::DecodeError(error));
    decoded
}

pub fn encode_bech32(
    data: &[u8],
    network: &NetworkDefinition,
) -> Option<String> {
    AddressBech32Encoder::new(network).encode(&data).ok()
}
