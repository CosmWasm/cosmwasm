use std::convert::TryFrom;

use sha2::{Digest, Sha256};

use crate::errors::{make_cache_err, VmError};

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

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl From<[u8; 32]> for Checksum {
    fn from(data: [u8; 32]) -> Self {
        Checksum(data)
    }
}

impl TryFrom<&[u8]> for Checksum {
    type Error = VmError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 32 {
            return Err(make_cache_err("Checksum not of length 32"));
        }
        let mut data = [0u8; 32];
        data.copy_from_slice(value);
        Ok(Checksum(data))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_works() {
        let wasm = vec![12u8; 17];
        let id = Checksum::generate(&wasm);
        assert_eq!(id.0.len(), 32);
    }
}
