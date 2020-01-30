use std::fmt;

use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use snafu::ResultExt;

use crate::errors::{Base64Err, Result};

#[derive(Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct Base64(pub Vec<u8>);

// Base64 is guaranteed to be a valid Base64 string.
// This is meant to be converted to-and-from raw bytes, but can also be json serialized as a string
impl Base64 {
    /// take an (untrusted) string and decode it into bytes.
    /// fails if it is not valid base64
    pub fn decode(encoded: &str) -> Result<Self> {
        let binary = base64::decode(&encoded).context(Base64Err {})?;
        Ok(Base64(binary))
    }

    /// encode to string (guaranteed to be success as we control the data inside).
    /// this returns normalized form (with trailing = if needed)
    pub fn encode(&self) -> String {
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

impl fmt::Display for Base64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.encode())
    }
}

impl From<&[u8]> for Base64 {
    fn from(data: &[u8]) -> Self {
        Self(data.to_vec())
    }
}

impl Serialize for Base64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.encode())
    }
}

// decode base64 string to binary
impl<'de> Deserialize<'de> for Base64 {
    fn deserialize<D>(deserializer: D) -> Result<Base64, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(Base64Visitor)
    }
}

struct Base64Visitor;

impl<'de> de::Visitor<'de> for Base64Visitor {
    type Value = Base64;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("valid base64 encoded string")
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match Base64::decode(v) {
            Ok(b64) => Ok(b64),
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
        let data: &[u8] = b"hello";
        let encoded = Base64::from(data).encode();
        assert_eq!(8, encoded.len());
        let decoded = Base64::decode(&encoded).unwrap();
        assert_eq!(data, decoded.as_slice());
    }

    #[test]
    fn encode_decode_non_ascii() {
        let data = vec![12u8, 187, 0, 17, 250, 1];
        let encoded = Base64(data.clone()).encode();
        assert_eq!(8, encoded.len());
        let decoded = Base64::decode(&encoded).unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[test]
    fn from_valid_string() {
        let valid = "cmFuZG9taVo=";
        let decoded = Base64::decode(valid).unwrap();
        assert_eq!(b"randomiZ", decoded.as_slice());
    }

    // this accepts input without a trailing = but outputs normal form
    #[test]
    fn from_shortened_string() {
        let short = "cmFuZG9taVo";
        let long = "cmFuZG9taVo=";
        let decoded = Base64::decode(short).unwrap();
        assert_eq!(b"randomiZ", decoded.as_slice());
        assert_eq!(long, decoded.encode());
    }

    #[test]
    fn from_invalid_string() {
        let valid = "cm%uZG9taVo";
        let res = Base64::decode(valid);
        assert!(res.is_err());
    }

    #[test]
    fn serialization_works() {
        let data = vec![0u8, 187, 61, 11, 250, 0];
        let encoded = Base64(data);

        let serialized = to_vec(&encoded).unwrap();
        let deserialized: Base64 = from_slice(&serialized).unwrap();

        assert_eq!(encoded, deserialized);
    }

    #[test]
    fn deserialize_from_valid_string() {
        let b64_str = "ALs9C/oA";
        // this is the binary behind above string
        let expected = vec![0u8, 187, 61, 11, 250, 0];

        let serialized = to_vec(&b64_str).unwrap();
        let deserialized: Base64 = from_slice(&serialized).unwrap();
        assert_eq!(expected, deserialized.as_slice());
    }

    #[test]
    fn deserialize_from_invalid_string() {
        let invalid_str = "**BAD!**";
        let serialized = to_vec(&invalid_str).unwrap();
        let deserialized = from_slice::<Base64>(&serialized);
        assert!(deserialized.is_err());
    }
}
