use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::errors::{Base64Err, Result};

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
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
    pub fn as_str(&self) -> &str { self.0.as_str() }

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

#[cfg(test)]
mod test {
    use crate::encoding::Base64;

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
}
