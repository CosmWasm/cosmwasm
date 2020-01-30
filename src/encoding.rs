use std::fmt;

use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use snafu::ResultExt;

use crate::errors::{Base64Err, Result};

/// Binary is a wrapper around Vec<u8> to add base64 de/serialization
/// with serde. It also adds some helper methods to help encode inline.
///
/// This is only needed as serde-json-{core,wasm} has a horrible encoding for Vec<u8>
#[derive(Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct Binary(pub Vec<u8>);

impl Binary {
    /// take an (untrusted) string and decode it into bytes.
    /// fails if it is not valid base64
    pub fn from_base64(encoded: &str) -> Result<Self> {
        let binary = base64::decode(&encoded).context(Base64Err {})?;
        Ok(Binary(binary))
    }
    /// encode to base64 string (guaranteed to be success as we control the data inside).
    /// this returns normalized form (with trailing = if needed)
    pub fn to_base64(&self) -> String {
        base64::encode(&self.0)
    }
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for Binary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_base64())
    }
}

impl From<&[u8]> for Binary {
    fn from(binary: &[u8]) -> Self {
        Self(binary.to_vec())
    }
}

/// Serializes as a base64 string
impl Serialize for Binary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_base64())
    }
}

/// Deserializes as a base64 string
impl<'de> Deserialize<'de> for Binary {
    fn deserialize<D>(deserializer: D) -> Result<Binary, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(Base64Visitor)
    }
}

struct Base64Visitor;

impl<'de> de::Visitor<'de> for Base64Visitor {
    type Value = Binary;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("valid base64 encoded string")
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match Binary::from_base64(v) {
            Ok(binary) => Ok(binary),
            Err(_) => Err(E::custom(format!("invalid base64: {}", v))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::serde::{from_slice, to_vec};

    #[test]
    fn encode_decode() {
        let binary: &[u8] = b"hello";
        let encoded = Binary::from(binary).to_base64();
        assert_eq!(8, encoded.len());
        let decoded = Binary::from_base64(&encoded).unwrap();
        assert_eq!(binary, decoded.as_slice());
    }

    #[test]
    fn encode_decode_non_ascii() {
        let binary = vec![12u8, 187, 0, 17, 250, 1];
        let encoded = Binary(binary.clone()).to_base64();
        assert_eq!(8, encoded.len());
        let decoded = Binary::from_base64(&encoded).unwrap();
        assert_eq!(binary.as_slice(), decoded.as_slice());
    }

    #[test]
    fn from_valid_string() {
        let valid_base64 = "cmFuZG9taVo=";
        let binary = Binary::from_base64(valid_base64).unwrap();
        assert_eq!(b"randomiZ", binary.as_slice());
    }

    // this accepts input without a trailing = but outputs normal form
    #[test]
    fn from_shortened_string() {
        let short = "cmFuZG9taVo";
        let long = "cmFuZG9taVo=";
        let binary = Binary::from_base64(short).unwrap();
        assert_eq!(b"randomiZ", binary.as_slice());
        assert_eq!(long, binary.to_base64());
    }

    #[test]
    fn from_invalid_string() {
        let invalid_base64 = "cm%uZG9taVo";
        let res = Binary::from_base64(invalid_base64);
        assert!(res.is_err());
    }

    #[test]
    fn serialization_works() {
        let binary = Binary(vec![0u8, 187, 61, 11, 250, 0]);

        let json = to_vec(&binary).unwrap();
        let deserialized: Binary = from_slice(&json).unwrap();

        assert_eq!(binary, deserialized);
    }

    #[test]
    fn deserialize_from_valid_string() {
        let b64_str = "ALs9C/oA";
        // this is the binary behind above string
        let expected = vec![0u8, 187, 61, 11, 250, 0];

        let serialized = to_vec(&b64_str).unwrap();
        let deserialized: Binary = from_slice(&serialized).unwrap();
        assert_eq!(expected, deserialized.as_slice());
    }

    #[test]
    fn deserialize_from_invalid_string() {
        let invalid_str = "**BAD!**";
        let serialized = to_vec(&invalid_str).unwrap();
        let res = from_slice::<Binary>(&serialized);
        assert!(res.is_err());
    }
}
