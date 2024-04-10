//! The crypto crate is intended to be used in internal crates / utils.
//! Please don't use any of these types directly, as
//! they might change frequently, or be removed in the future.
//! This crate does not adhere to semantic versioning.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(test)]
extern crate std; // allow for file I/O during tests

mod backtrace;
#[cfg(feature = "std")]
mod bls12_318;
mod ecdsa;
mod ed25519;
mod errors;
mod identity_digest;
mod secp256k1;
mod secp256r1;

#[cfg(feature = "std")]
#[doc(hidden)]
pub use crate::bls12_318::{
    bls12_381_aggregate_g1, bls12_381_aggregate_g2, bls12_381_g1_generator, bls12_381_g2_generator,
    bls12_381_hash_to_g1, bls12_381_hash_to_g2, bls12_381_pairing_equality, HashFunction,
};
#[doc(hidden)]
pub use crate::ecdsa::{ECDSA_PUBKEY_MAX_LEN, ECDSA_SIGNATURE_LEN, MESSAGE_HASH_MAX_LEN};
#[doc(hidden)]
pub use crate::ed25519::EDDSA_PUBKEY_LEN;
#[doc(hidden)]
pub use crate::ed25519::{ed25519_batch_verify, ed25519_verify};
#[doc(hidden)]
pub use crate::errors::{CryptoError, CryptoResult};
#[doc(hidden)]
pub use crate::secp256k1::{secp256k1_recover_pubkey, secp256k1_verify};
#[doc(hidden)]
pub use crate::secp256r1::{secp256r1_recover_pubkey, secp256r1_verify};
pub(crate) use backtrace::BT;
