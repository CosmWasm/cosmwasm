use std::fmt;

use schemars::JsonSchema;
use serde::{de, Deserialize, Deserializer, Serialize};
use snafu::ResultExt;

use crate::errors::{Base64Err, Result};

#[derive(Serialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct Base64(String);

// Base64 is guaranteed to be a valid Base64 string.
// This is meant to be converted to-and-from raw bytes, but can also be json serialized as a string
impl Base64 {
    // encode raw data (binary -> base64 string)
    pub fn new(data: &[u8]) -> Self {
        Base64(base64::encode(data))
    }

    // take an (untrusted) string and assert it is valid base64 before casting it
    // fail here, so decode is ensured to succeed.
    //
    // We also want to normalize it (to ensure trailing =), so we do a full decode-encode here
    // FIXME: We can optimize this later.
    pub fn from_encoded(encoded: &str) -> Result<Self> {
        let binary = base64::decode(&encoded).context(Base64Err {})?;
        Ok(Base64::new(&binary))
    }

    // this returns the base64 string
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    // decode the raw data (guaranteed to be success as we control the data inside)
    pub fn decode(&self) -> Vec<u8> {
        base64::decode(&self.0).unwrap()
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
        write!(f, "{}", self.as_str())
    }
}

// all this to enforce json is correct when decoding
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
        match Base64::from_encoded(v) {
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
        let data = b"hello";
        let encoded = Base64::new(data);
        assert_eq!(8, encoded.len());
        let decoded = encoded.decode();
        assert_eq!(data, decoded.as_slice());
    }

    #[test]
    fn encode_decode_non_ascii() {
        let data = vec![12u8, 187, 0, 17, 250, 1];
        let encoded = Base64::new(&data);
        assert_eq!(8, encoded.len());
        let decoded = encoded.decode();
        assert_eq!(data, decoded);
    }

    #[test]
    fn from_valid_string() {
        let valid = "cmFuZG9taVo=";
        let encoded = Base64::from_encoded(valid).unwrap();
        assert_eq!(12, encoded.len());
        assert_eq!(valid, encoded.as_str());
        let decoded = encoded.decode();
        assert_eq!(b"randomiZ", decoded.as_slice());
    }

    #[test]
    // this must be normalized form (with trailing =)
    fn from_shortened_string() {
        let valid = "cmFuZG9taVo";
        let encoded = Base64::from_encoded(valid).unwrap();
        assert_eq!(12, encoded.len());
        let decoded = encoded.decode();
        assert_eq!(b"randomiZ", decoded.as_slice());
    }

    #[test]
    fn from_invalid_string() {
        let valid = "cm%uZG9taVo";
        let res = Base64::from_encoded(valid);
        assert!(res.is_err());
    }

    #[test]
    fn serialization_works() {
        let data = vec![0u8, 187, 61, 11, 250, 0];
        let encoded = Base64::new(&data);

        let serialized = to_vec(&encoded).unwrap();
        let deserialized: Base64 = from_slice(&serialized).unwrap();

        assert_eq!(encoded, deserialized);
        assert_eq!(data, deserialized.decode());
    }

    #[test]
    fn deserialize_from_valid_string() {
        let b64_str = "ALs9C/oA";
        // this is the binary behind above string
        let expected = vec![0u8, 187, 61, 11, 250, 0];

        let serialized = to_vec(&b64_str).unwrap();
        let deserialized: Base64 = from_slice(&serialized).unwrap();
        assert_eq!(expected, deserialized.decode());
    }

    #[test]
    fn deserialize_from_invalid_string() {
        let invalid_str = "**BAD!**";
        let serialized = to_vec(&invalid_str).unwrap();
        let deserialized = from_slice::<Base64>(&serialized);
        assert!(deserialized.is_err());
    }
}
