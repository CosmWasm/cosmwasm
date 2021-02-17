//! The crypto crate is intended to be used in internal crates / utils.
//! Please don't use any of these types directly, as
//! they might change frequently, or be removed in the future.
//! This crate does not adhere to semantic versioning.
#![cfg_attr(feature = "backtraces", feature(backtrace))]

mod crypto;
mod errors;
mod identity_digest;

#[doc(hidden)]
pub use crate::crypto::secp256k1_verify;
#[doc(hidden)]
pub use crate::crypto::{MESSAGE_HASH_MAX_LENGTH, PUBKEY_MAX_LENGTH, SIGNATURE_MAX_LENGTH};
#[doc(hidden)]
pub use crate::errors::{CryptoError, CryptoResult};
