use core::fmt;

use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{StdError, StdResult};

/// A SHA-256 checksum of a Wasm blob, used to identify a Wasm code.
/// This must remain stable since this checksum is stored in the blockchain state.
///
/// This is often referred to as "code ID" in go-cosmwasm, even if code ID
/// usually refers to an auto-incrementing number.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Checksum([u8; 32]);

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
}
