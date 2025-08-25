//! CosmWasm is a smart contract platform for the Cosmos ecosystem.
//! This crate implements cryptography-related functions for CosmWasm contracts and internal crates.
//!
//! **Note:** This crate is intended to be used in internal crates / utils only.
//! Please don't use any of these types directly, as they might change frequently,
//! or be removed in the future. This crate does not adhere to semantic versioning.
//!
//! For more information, see: <https://cosmwasm.cosmos.network>

extern crate alloc;

mod backtrace;
mod bls12_381;
mod ecdsa;
mod ed25519;
mod errors;
mod identity_digest;
mod secp256k1;
mod secp256r1;

#[doc(hidden)]
pub use crate::bls12_381::{
    bls12_381_aggregate_g1, bls12_381_aggregate_g2, bls12_381_g1_is_identity,
    bls12_381_g2_is_identity, bls12_381_hash_to_g1, bls12_381_hash_to_g2,
    bls12_381_pairing_equality, HashFunction,
};
#[doc(hidden)]
pub use crate::ecdsa::{ECDSA_PUBKEY_MAX_LEN, ECDSA_SIGNATURE_LEN, MESSAGE_HASH_MAX_LEN};
#[doc(hidden)]
pub use crate::ed25519::EDDSA_PUBKEY_LEN;
#[doc(hidden)]
pub use crate::ed25519::{ed25519_batch_verify, ed25519_verify};
#[doc(hidden)]
pub use crate::errors::{
    Aggregation as AggregationError, CryptoError, CryptoResult,
    PairingEquality as PairingEqualityError,
};
#[doc(hidden)]
pub use crate::secp256k1::{secp256k1_recover_pubkey, secp256k1_verify};
#[doc(hidden)]
pub use crate::secp256r1::{secp256r1_recover_pubkey, secp256r1_verify};
pub(crate) use backtrace::BT;
