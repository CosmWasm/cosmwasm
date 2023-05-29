
use sha3::{Keccak256};
use digest::{Digest}; // trait
use crate::errors::{CryptoError};

pub fn keccak256(
    data: &[u8],
) -> Result<Vec<u8>, CryptoError>  {
    let hash = Keccak256::digest(data);
    Ok((&hash).to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    // For generic signature verification
    const KECCAK256_MSG: &str = "1ff5c235b3c317d054b80b4bf0a8038bd727d180872d2491a7edef4f949c4135";
    const KECCAK256_RESULT :& str = "374c6f18084ec509581669659c7bce243284f85ddaa164c77bde9e9abd65fc0d";
    #[test]
    fn test_keccak256() {
        let s_bytes: &[u8] = KECCAK256_MSG.as_bytes();
        let message_digest = keccak256(s_bytes);
        assert!(hex::encode(message_digest.unwrap()).to_owned().as_str() == KECCAK256_RESULT);
    }
}
