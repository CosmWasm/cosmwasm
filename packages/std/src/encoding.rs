use std::fmt;

use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};

use crate::errors::{StdError, StdResult};
use std::ops::Deref;

/// Binary is a wrapper around Vec<u8> to add base64 de/serialization
/// with serde. It also adds some helper methods to help encode inline.
///
/// This is only needed as serde-json-{core,wasm} has a horrible encoding for Vec<u8>
#[derive(Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct Binary(#[schemars(with = "String")] pub Vec<u8>);

impl Binary {
    /// take an (untrusted) string and decode it into bytes.
    /// fails if it is not valid base64
    pub fn from_base64(encoded: &str) -> StdResult<Self> {
        let binary = base64::decode(&encoded).map_err(StdError::invalid_base64)?;
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

    /// Converts a `Binary` into a vector of bytes.
    ///
    /// This consumes the `Binary`, so we do not need to copy its contents.
    /// It is equivalent to both `Vec::<u8>::from(binary)` and `let v: Vec<u8> = binary.into()` and just a matter of taste which one you use.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use cosmwasm_std::Binary;
    /// let binary = Binary::from_base64("ALs9C/oA").unwrap();
    /// let bytes = binary.into_vec();
    ///
    /// assert_eq!(bytes, &[0, 187, 61, 11, 250, 0]);
    /// ```
    pub fn into_vec(self) -> Vec<u8> {
        self.into()
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

impl Deref for Binary {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

// Macro needed until https://rust-lang.github.io/rfcs/2000-const-generics.html is stable.
// See https://users.rust-lang.org/t/how-to-implement-trait-for-fixed-size-array-of-any-size/31494
macro_rules! implement_from_for_fixed_length_arrays {
    ($($N:literal)+) => {
        $(
            // Reference
            impl From<&[u8; $N]> for Binary {
                fn from(source: &[u8; $N]) -> Self {
                    Self(source.to_vec())
                }
            }

            // Owned
            impl From<[u8; $N]> for Binary {
                fn from(source: [u8; $N]) -> Self {
                    // Implementation available for $N <= 32.
                    // Requires https://caniuse.rs/features/vec_from_array, avaiable since Rust 1.44.0.
                    Self(source.into())
                }
            }
        )+
    }
}

implement_from_for_fixed_length_arrays! {
     0  1  2  3  4  5  6  7  8  9
    10 11 12 13 14 15 16 17 18 19
    20 21 22 23 24 25 26 27 28 29
    30 31 32
}

impl From<Vec<u8>> for Binary {
    fn from(vec: Vec<u8>) -> Self {
        Self(vec)
    }
}

impl From<Binary> for Vec<u8> {
    fn from(original: Binary) -> Vec<u8> {
        original.0
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

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
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
    use crate::errors::StdError;
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
        match res.unwrap_err() {
            StdError::InvalidBase64 { msg, .. } => assert_eq!(msg, "Invalid byte 37, offset 2."),
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn from_slice_works() {
        let original: &[u8] = &[0u8, 187, 61, 11, 250, 0];
        let binary: Binary = original.into();
        assert_eq!(binary.as_slice(), [0u8, 187, 61, 11, 250, 0]);
    }

    #[test]
    fn from_fixed_length_array_works() {
        let original = &[];
        let binary: Binary = original.into();
        assert_eq!(binary.len(), 0);

        let original = &[0u8];
        let binary: Binary = original.into();
        assert_eq!(binary.as_slice(), [0u8]);

        let original = &[0u8, 187, 61, 11, 250, 0];
        let binary: Binary = original.into();
        assert_eq!(binary.as_slice(), [0u8, 187, 61, 11, 250, 0]);

        let original = &[
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1,
        ];
        let binary: Binary = original.into();
        assert_eq!(
            binary.as_slice(),
            [
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1,
            ]
        );
    }

    #[test]
    fn from_owned_fixed_length_array_works() {
        let original = [];
        let binary: Binary = original.into();
        assert_eq!(binary.len(), 0);

        let original = [0u8];
        let binary: Binary = original.into();
        assert_eq!(binary.as_slice(), [0u8]);

        let original = [0u8, 187, 61, 11, 250, 0];
        let binary: Binary = original.into();
        assert_eq!(binary.as_slice(), [0u8, 187, 61, 11, 250, 0]);

        let original = [
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1,
        ];
        let binary: Binary = original.into();
        assert_eq!(
            binary.as_slice(),
            [
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1,
            ]
        );
    }

    #[test]
    fn from_literal_works() {
        let a: Binary = b"".into();
        assert_eq!(a.len(), 0);

        let a: Binary = b".".into();
        assert_eq!(a.len(), 1);

        let a: Binary = b"...".into();
        assert_eq!(a.len(), 3);

        let a: Binary = b"...............................".into();
        assert_eq!(a.len(), 31);

        let a: Binary = b"................................".into();
        assert_eq!(a.len(), 32);

        // for length > 32 we need to cast
        let a: Binary = (b"................................." as &[u8]).into();
        assert_eq!(a.len(), 33);
    }

    #[test]
    fn from_vec_works() {
        let original = vec![0u8, 187, 61, 11, 250, 0];
        let original_ptr = original.as_ptr();
        let binary: Binary = original.into();
        assert_eq!(binary.as_slice(), [0u8, 187, 61, 11, 250, 0]);
        assert_eq!(binary.0.as_ptr(), original_ptr, "vector must not be copied");
    }

    #[test]
    fn into_vec_works() {
        // Into<Vec<u8>> for Binary
        let original = Binary(vec![0u8, 187, 61, 11, 250, 0]);
        let original_ptr = original.0.as_ptr();
        let vec: Vec<u8> = original.into();
        assert_eq!(vec.as_slice(), [0u8, 187, 61, 11, 250, 0]);
        assert_eq!(vec.as_ptr(), original_ptr, "vector must not be copied");

        // From<Binary> for Vec<u8>
        let original = Binary(vec![7u8, 35, 49, 101, 0, 255]);
        let original_ptr = original.0.as_ptr();
        let vec = Vec::<u8>::from(original);
        assert_eq!(vec.as_slice(), [7u8, 35, 49, 101, 0, 255]);
        assert_eq!(vec.as_ptr(), original_ptr, "vector must not be copied");
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
