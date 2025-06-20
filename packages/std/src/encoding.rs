use alloc::{string::String, vec::Vec};
use base64::{engine::GeneralPurpose, Engine};

use crate::StdResult;

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
pub fn from_base64<I>(input: I) -> StdResult<Vec<u8>>
where
    I: AsRef<[u8]>,
{
    Ok(B64_ENGINE.decode(input)?)
}

/// Encode a bag of bytes into the Base64 format
pub fn to_base64<I>(input: I) -> String
where
    I: AsRef<[u8]>,
{
    B64_ENGINE.encode(input)
}

/// Decode a bag of bytes from hex into a vector of bytes
pub fn from_hex<I>(input: I) -> StdResult<Vec<u8>>
where
    I: AsRef<[u8]>,
{
    Ok(hex::decode(input)?)
}

/// Encode a bag of bytes into the hex format
pub fn to_hex<I>(input: I) -> String
where
    I: AsRef<[u8]>,
{
    hex::encode(input)
}

#[cfg(test)]
mod test {
    use crate::{from_base64, from_hex, to_base64, to_hex};

    const BASE64_FOOBAR: &str = "Zm9vYmFy"; // utf-8 encoded "foobar"
    const HEX_FOOBAR: &str = "666f6f626172"; // utf-8 encoded "foobar"

    #[test]
    fn from_base64_works() {
        let decoded = from_base64(BASE64_FOOBAR).unwrap();
        assert_eq!(decoded, b"foobar");
    }

    #[test]
    fn to_base64_works() {
        let encoded = to_base64("foobar");
        assert_eq!(encoded, BASE64_FOOBAR);
    }

    #[test]
    fn base64_roundtrip_works() {
        let decoded = from_base64(BASE64_FOOBAR).unwrap();
        assert_eq!(decoded, b"foobar");
        let encoded = to_base64(decoded);
        assert_eq!(encoded, BASE64_FOOBAR);
    }

    #[test]
    fn from_hex_works() {
        let decoded = from_hex(HEX_FOOBAR).unwrap();
        assert_eq!(decoded, b"foobar");
    }

    #[test]
    fn to_hex_works() {
        let encoded = to_hex("foobar");
        assert_eq!(encoded, HEX_FOOBAR);
    }

    #[test]
    fn hex_roundtrip_works() {
        let decoded = from_hex(HEX_FOOBAR).unwrap();
        assert_eq!(decoded, b"foobar");
        let encoded = to_hex(decoded);
        assert_eq!(encoded, HEX_FOOBAR);
    }
}
