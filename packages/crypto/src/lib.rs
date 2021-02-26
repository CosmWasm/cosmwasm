//! The crypto crate is intended to be used in internal crates / utils.
//! Please don't use any of these types directly, as
//! they might change frequently, or be removed in the future.
//! This crate does not adhere to semantic versioning.
#![cfg_attr(feature = "backtraces", feature(backtrace))]

mod ed25519;
mod errors;
mod identity_digest;
mod secp256k1;

#[doc(hidden)]
pub use crate::ed25519::{ed25519_batch_verify, ed25519_verify};
#[doc(hidden)]
pub use crate::ed25519::{BATCH_MAX_LEN, EDDSA_PUBKEY_LEN, MESSAGE_MAX_LEN};
#[doc(hidden)]
pub use crate::errors::{CryptoError, CryptoResult};
#[doc(hidden)]
pub use crate::secp256k1::{secp256k1_recover_pubkey, secp256k1_verify};
#[doc(hidden)]
pub use crate::secp256k1::{ECDSA_PUBKEY_MAX_LEN, ECDSA_SIGNATURE_LEN, MESSAGE_HASH_MAX_LEN};
