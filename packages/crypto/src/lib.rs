//! The crypto crate is intended to be used in internal crates / utils.
//! Please don't use any of these types directly, as
//! they might change frequently, or be removed in the future.
//! This crate does not adhere to semantic versioning.
#![cfg_attr(feature = "backtraces", feature(backtrace))]

mod crypto;
mod errors;
mod identity_digest;

#[doc(hidden)]
pub use crate::crypto::{ed25519_verify, secp256k1_verify};
#[doc(hidden)]
pub use crate::crypto::{ECDSA_PUBKEY_MAX_LEN, ECDSA_SIGNATURE_LEN, MESSAGE_HASH_MAX_LEN};
#[doc(hidden)]
pub use crate::crypto::{EDDSA_PUBKEY_LEN, EDDSA_SIGNATURE_LEN, MESSAGE_MAX_LEN};
#[doc(hidden)]
pub use crate::errors::{CryptoError, CryptoResult};
