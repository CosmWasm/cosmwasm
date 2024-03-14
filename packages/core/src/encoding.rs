use alloc::{string::String, vec::Vec};
use base64::{engine::GeneralPurpose, Engine};

use crate::{CoreError, CoreResult};

/// Base64 encoding engine used in conversion to/from base64.
///
/// The engine adds padding when encoding and accepts strings with or
/// without padding when decoding.
const B64_ENGINE: GeneralPurpose = GeneralPurpose::new(
    &base64::alphabet::STANDARD,
    base64::engine::GeneralPurposeConfig::new()
        .with_decode_padding_mode(base64::engine::DecodePaddingMode::Indifferent),
);

/// Deserialize a bag of bytes from Base64 into a vector of bytes
pub fn from_base64<I>(input: I) -> CoreResult<Vec<u8>>
where
    I: AsRef<[u8]>,
{
    B64_ENGINE.decode(input).map_err(CoreError::invalid_base64)
}

/// Encode a bag of bytes into the Base64 format
pub fn to_base64<I>(input: I) -> String
where
    I: AsRef<[u8]>,
{
    B64_ENGINE.encode(input)
}

/// Decode a bag of bytes from hex into a vector of bytes
pub fn from_hex<I>(input: I) -> CoreResult<Vec<u8>>
where
    I: AsRef<[u8]>,
{
    hex::decode(input).map_err(CoreError::invalid_hex)
}

/// Encode a bag of bytes into the hex format
pub fn to_hex<I>(input: I) -> String
where
    I: AsRef<[u8]>,
{
    hex::encode(input)
}
