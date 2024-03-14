use alloc::vec::Vec;
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

pub fn from_base64<I>(input: I) -> CoreResult<Vec<u8>>
where
    I: AsRef<[u8]>,
{
    B64_ENGINE.decode(input).map_err(CoreError::invalid_base64)
}

pub fn to_base64<I>(input: I) -> String
where
    I: AsRef<[u8]>,
{
    B64_ENGINE.encode(input)
}

pub fn from_hex<I>(input: I) -> CoreResult<Vec<u8>>
where
    I: AsRef<[u8]>,
{
    hex::decode(input).map_err(CoreError::invalid_hex)
}

pub fn to_hex<I>(input: I) -> String
where
    I: AsRef<[u8]>,
{
    hex::encode(input)
}
