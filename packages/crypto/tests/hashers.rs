// only some of the hashers are used in each test file, so some will be unused
#![allow(unused)]

use digest::Digest;
use sha2::{Sha256, Sha512};
use sha3::{Keccak256, Sha3_256, Sha3_512};

pub fn sha256(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

pub fn keccak_256(data: &[u8]) -> [u8; 32] {
    Keccak256::digest(data).into()
}

// ecdsa_secp256k1_sha512 requires truncating to 32 bytes
pub fn sha512(data: &[u8]) -> [u8; 32] {
    let hash = Sha512::digest(data).to_vec();
    hash[..32].try_into().unwrap()
}

pub fn sha3_256(data: &[u8]) -> [u8; 32] {
    Sha3_256::digest(data).into()
}

// ecdsa_secp256k1_sha3_512 requires truncating to 32 bytes
pub fn sha3_512(data: &[u8]) -> [u8; 32] {
    let hash = Sha3_512::digest(data).to_vec();
    hash[..32].try_into().unwrap()
}
