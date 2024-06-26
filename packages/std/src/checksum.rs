use core::fmt;

use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::prelude::*;
use crate::{StdError, StdResult};

/// A SHA-256 checksum of a Wasm blob, used to identify a Wasm code.
/// This must remain stable since this checksum is stored in the blockchain state.
///
/// This is often referred to as "code ID" in go-cosmwasm, even if code ID
/// usually refers to an auto-incrementing number.
#[derive(JsonSchema, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Checksum(#[schemars(with = "String")] [u8; 32]);

impl Checksum {
    pub fn generate(wasm: &[u8]) -> Self {
        Checksum(Sha256::digest(wasm).into())
    }

    /// Tries to parse the given hex string into a checksum.
    /// Errors if the string contains non-hex characters or does not contain 32 bytes.
    pub fn from_hex(input: &str) -> StdResult<Self> {
        let mut binary = [0u8; 32];
        hex::decode_to_slice(input, &mut binary).map_err(StdError::invalid_hex)?;

        Ok(Self(binary))
    }

    /// Creates a lowercase hex encoded copy of this checksum.
    ///
    /// This takes an owned `self` instead of a reference because `Checksum` is cheap to `Copy`.
    pub fn to_hex(self) -> String {
        self.to_string()
    }

    /// Returns a reference to the inner bytes of this checksum as a slice.
    /// If you need a reference to the array, use [`AsRef::as_ref`].
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for byte in self.0.iter() {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl From<[u8; 32]> for Checksum {
    fn from(data: [u8; 32]) -> Self {
        Checksum(data)
    }
}

impl AsRef<[u8; 32]> for Checksum {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Serializes as a hex string
impl Serialize for Checksum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_hex())
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

/// Deserializes as a hex string
impl<'de> Deserialize<'de> for Checksum {
    fn deserialize<D>(deserializer: D) -> Result<Checksum, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(ChecksumVisitor)
        } else {
            deserializer.deserialize_bytes(ChecksumBytesVisitor)
        }
    }
}

struct ChecksumVisitor;

impl<'de> de::Visitor<'de> for ChecksumVisitor {
    type Value = Checksum;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("valid hex encoded 32 byte checksum")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match Checksum::from_hex(v) {
            Ok(data) => Ok(data),
            Err(_) => Err(E::custom(format!("invalid checksum: {v}"))),
        }
    }
}

struct ChecksumBytesVisitor;

impl<'de> de::Visitor<'de> for ChecksumBytesVisitor {
    type Value = Checksum;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("32 byte checksum")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Checksum::try_from(v).map_err(|ChecksumError| E::invalid_length(v.len(), &"32 bytes"))
    }
}

#[derive(Error, Debug)]
#[error("Checksum not of length 32")]
pub struct ChecksumError;

impl TryFrom<&[u8]> for Checksum {
    type Error = ChecksumError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 32 {
            return Err(ChecksumError);
        }
        let mut data = [0u8; 32];
        data.copy_from_slice(value);
        Ok(Checksum(data))
    }
}

impl From<Checksum> for Vec<u8> {
    fn from(original: Checksum) -> Vec<u8> {
        original.0.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::to_json_string;

    #[test]
    fn generate_works() {
        let wasm = vec![0x68, 0x69, 0x6a];
        let checksum = Checksum::generate(&wasm);

        // echo -n "hij" | sha256sum
        let expected = [
            0x72, 0x2c, 0x8c, 0x99, 0x3f, 0xd7, 0x5a, 0x76, 0x27, 0xd6, 0x9e, 0xd9, 0x41, 0x34,
            0x4f, 0xe2, 0xa1, 0x42, 0x3a, 0x3e, 0x75, 0xef, 0xd3, 0xe6, 0x77, 0x8a, 0x14, 0x28,
            0x84, 0x22, 0x71, 0x04,
        ];
        assert_eq!(checksum.0, expected);
    }

    #[test]
    fn implemented_display() {
        let wasm = vec![0x68, 0x69, 0x6a];
        let checksum = Checksum::generate(&wasm);
        // echo -n "hij" | sha256sum
        let embedded = format!("Check: {checksum}");
        assert_eq!(
            embedded,
            "Check: 722c8c993fd75a7627d69ed941344fe2a1423a3e75efd3e6778a142884227104"
        );
        assert_eq!(
            checksum.to_string(),
            "722c8c993fd75a7627d69ed941344fe2a1423a3e75efd3e6778a142884227104"
        );
    }

    #[test]
    fn from_hex_works() {
        // echo -n "hij" | sha256sum
        let checksum = "722c8c993fd75a7627d69ed941344fe2a1423a3e75efd3e6778a142884227104";
        let parsed = Checksum::from_hex(checksum).unwrap();
        assert_eq!(parsed, Checksum::generate(b"hij"));
        // should be inverse of `to_hex`
        assert_eq!(parsed.to_hex(), checksum);

        // invalid hex
        let too_short = "722c8c993fd75a7627d69ed941344fe2a1423a3e75efd3e6778a1428842271";
        assert!(Checksum::from_hex(too_short).is_err());
        let invalid_char = "722c8c993fd75a7627d69ed941344fe2a1423a3e75efd3e6778a1428842271g4";
        assert!(Checksum::from_hex(invalid_char).is_err());
        let too_long = "722c8c993fd75a7627d69ed941344fe2a1423a3e75efd3e6778a14288422710400";
        assert!(Checksum::from_hex(too_long).is_err());
    }

    #[test]
    fn to_hex_works() {
        let wasm = vec![0x68, 0x69, 0x6a];
        let checksum = Checksum::generate(&wasm);
        // echo -n "hij" | sha256sum
        assert_eq!(
            checksum.to_hex(),
            "722c8c993fd75a7627d69ed941344fe2a1423a3e75efd3e6778a142884227104"
        );
    }

    #[test]
    fn into_vec_works() {
        let checksum = Checksum::generate(&[12u8; 17]);
        let as_vec: Vec<u8> = checksum.into();
        assert_eq!(as_vec, checksum.0);
    }

    #[test]
    fn ref_conversions_work() {
        let checksum = Checksum::generate(&[12u8; 17]);
        // as_ref
        let _: &[u8; 32] = checksum.as_ref();
        let _: &[u8] = checksum.as_ref();
        // as_slice
        let _: &[u8; 32] = checksum.as_ref();
        let _: &[u8] = checksum.as_ref();
    }

    #[test]
    fn serde_works() {
        // echo -n "hij" | sha256sum
        let checksum =
            Checksum::from_hex("722c8c993fd75a7627d69ed941344fe2a1423a3e75efd3e6778a142884227104")
                .unwrap();

        let serialized = to_json_string(&checksum).unwrap();
        assert_eq!(
            serialized,
            "\"722c8c993fd75a7627d69ed941344fe2a1423a3e75efd3e6778a142884227104\""
        );

        let deserialized: Checksum = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, checksum);
    }

    #[test]
    fn msgpack_works() {
        // echo -n "hij" | sha256sum
        let checksum =
            Checksum::from_hex("722c8c993fd75a7627d69ed941344fe2a1423a3e75efd3e6778a142884227104")
                .unwrap();

        let serialized = rmp_serde::to_vec(&checksum).unwrap();
        // see: https://github.com/msgpack/msgpack/blob/8aa09e2/spec.md#bin-format-family
        let expected = vec![
            0xc4, 0x20, 0x72, 0x2c, 0x8c, 0x99, 0x3f, 0xd7, 0x5a, 0x76, 0x27, 0xd6, 0x9e, 0xd9,
            0x41, 0x34, 0x4f, 0xe2, 0xa1, 0x42, 0x3a, 0x3e, 0x75, 0xef, 0xd3, 0xe6, 0x77, 0x8a,
            0x14, 0x28, 0x84, 0x22, 0x71, 0x04,
        ];
        assert_eq!(serialized, expected);

        let deserialized: Checksum = rmp_serde::from_slice(&serialized).unwrap();
        assert_eq!(deserialized, checksum);
    }
}
