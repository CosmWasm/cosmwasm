#![cfg_attr(feature = "backtraces", feature(backtrace))]

mod crypto;
mod errors;
mod identity_digest;

pub use crate::crypto::secp256k1_verify;
pub use crate::crypto::{MESSAGE_HASH_MAX_LENGTH, PUBKEY_MAX_LENGTH, SIGNATURE_MAX_LENGTH};
pub use crate::errors::{CryptoError, CryptoResult};
