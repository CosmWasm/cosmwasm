use sha2::{Digest, Sha256};

/// A SHA-256 checksum of a Wasm blob, used to identify a Wasm code.
/// This must remain stable since this checksum is stored in the blockchain state.
///
/// This is often referred to as "code ID" in go-cosmwasm, even if code ID
/// usually refers to an auto-incrementing number.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Checksum(pub [u8; 32]);

impl Checksum {
    pub fn generate(wasm: &[u8]) -> Self {
        Checksum(Sha256::digest(wasm).into())
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
