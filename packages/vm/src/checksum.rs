use crate::modules::WasmHash;
use sha2::{Digest, Sha256};

/// A SHA-256 checksum of a Wasm blob, used to identify a Wasm code.
/// This must remain stable since this checksum is stored in the blockchain state.
///
/// This is often referred to as "code ID" in go-cosmwasm, even if code ID
/// usually refers to an auto-incrementing number.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Checksum([u8; 32]);

impl Checksum {
    pub fn from(data: [u8; 32]) -> Self {
        Checksum(data)
    }

    pub fn generate(wasm: &[u8]) -> Self {
        Checksum(Sha256::digest(wasm).into())
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// This generates a module hash in the data type required by Wasmer.
    /// The existence of this method is a bit hacky, since WasmHash::generate expects
    /// the Wasm blob as an input. Here we derive Wasm -> Checksum -> WasmHash.
    pub(crate) fn derive_module_hash(&self) -> WasmHash {
        WasmHash::generate(&self.0)
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
