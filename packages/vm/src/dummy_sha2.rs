//! Dummy Sha-256 and Sha-224 Digest impls.
//! These digests store/accept an already hashed value of the proper length.
//! Just to comply with the Digest contract.
//!
//! Adapted from `sha2` [sha256.rs](https://github.com/RustCrypto/hashes/blob/master/sha2/src/sha256.rs)
use sha2::digest::consts::{U28, U32};
use sha2::digest::generic_array::GenericArray;
use sha2::digest::{FixedOutputDirty, Reset, Update};

use sha2::digest;

/// The SHA-256 identity container
#[derive(Clone)]
pub struct IdentitySha256 {
    array: GenericArray<u8, U32>,
}

impl Default for IdentitySha256 {
    fn default() -> Self {
        IdentitySha256 {
            array: GenericArray::default(),
        }
    }
}

impl Update for IdentitySha256 {
    fn update(&mut self, hash: impl AsRef<[u8]>) {
        assert_eq!(hash.as_ref().len(), 32);
        self.array = *GenericArray::from_slice(hash.as_ref());
    }
}

impl FixedOutputDirty for IdentitySha256 {
    type OutputSize = U32;

    fn finalize_into_dirty(&mut self, out: &mut digest::Output<Self>) {
        *out = self.array;
    }
}

impl Reset for IdentitySha256 {
    fn reset(&mut self) {
        *self = Self::default();
    }
}

/// The SHA-224 identity container.
#[derive(Clone)]
pub struct IdentitySha224 {
    array: GenericArray<u8, U28>,
}

impl Default for IdentitySha224 {
    fn default() -> Self {
        IdentitySha224 {
            array: GenericArray::default(),
        }
    }
}

impl Update for IdentitySha224 {
    fn update(&mut self, hash: impl AsRef<[u8]>) {
        assert_eq!(hash.as_ref().len(), 28);
        self.array = *GenericArray::from_slice(hash.as_ref());
    }
}

impl FixedOutputDirty for IdentitySha224 {
    type OutputSize = U28;

    fn finalize_into_dirty(&mut self, out: &mut digest::Output<Self>) {
        *out = self.array;
    }
}

impl Reset for IdentitySha224 {
    fn reset(&mut self) {
        *self = Self::default();
    }
}
