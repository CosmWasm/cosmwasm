use alloc::string::String;
use core::fmt::Debug;
use derive_more::Display;

use crate::BT;

pub type CryptoResult<T> = core::result::Result<T, CryptoError>;

#[derive(Debug, Display)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum Aggregation {
    #[display("List of points is empty")]
    Empty,
    #[display("List is not a multiple of {expected_multiple}. Remainder: {remainder}")]
    NotMultiple {
        expected_multiple: usize,
        remainder: usize,
    },
}

#[derive(Debug, Display)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum PairingEquality {
    #[display("List is not a multiple of 48. Remainder: {remainder}")]
    NotMultipleG1 { remainder: usize },
    #[display("List is not a multiple of 96. Remainder: {remainder}")]
    NotMultipleG2 { remainder: usize },
    #[display("Not the same amount of points passed. Left: {left}, Right: {right}")]
    UnequalPointAmount { left: usize, right: usize },
}

#[derive(Debug, Display)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum InvalidPoint {
    #[display("Invalid input length for point (must be in compressed format): Expected {expected}, actual: {actual}")]
    InvalidLength { expected: usize, actual: usize },
    #[display("Invalid point")]
    DecodingError {},
}

#[derive(Display, Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum CryptoError {
    #[display("Point aggregation error: {source}")]
    Aggregation { source: Aggregation, backtrace: BT },
    #[display("Batch verify error: {msg}")]
    BatchErr { msg: String, backtrace: BT },
    #[display("Crypto error: {msg}")]
    GenericErr { msg: String, backtrace: BT },
    #[display("Invalid hash format")]
    InvalidHashFormat { backtrace: BT },
    #[display("Invalid public key format")]
    InvalidPubkeyFormat { backtrace: BT },
    #[display("Invalid signature format")]
    InvalidSignatureFormat { backtrace: BT },
    #[display("Invalid recovery parameter. Supported values: 0 and 1.")]
    InvalidRecoveryParam { backtrace: BT },
    #[display("Invalid point: {source}")]
    InvalidPoint { source: InvalidPoint, backtrace: BT },
    #[display("Pairing equality error: {source}")]
    PairingEquality {
        source: PairingEquality,
        backtrace: BT,
    },
    #[display("Unknown hash function")]
    UnknownHashFunction { backtrace: BT },
}

impl CryptoError {
    pub fn batch_err(msg: impl Into<String>) -> Self {
        CryptoError::BatchErr {
            msg: msg.into(),
            backtrace: BT::capture(),
        }
    }

    pub fn generic_err(msg: impl Into<String>) -> Self {
        CryptoError::GenericErr {
            msg: msg.into(),
            backtrace: BT::capture(),
        }
    }

    pub fn invalid_hash_format() -> Self {
        CryptoError::InvalidHashFormat {
            backtrace: BT::capture(),
        }
    }

    pub fn invalid_pubkey_format() -> Self {
        CryptoError::InvalidPubkeyFormat {
            backtrace: BT::capture(),
        }
    }

    pub fn invalid_signature_format() -> Self {
        CryptoError::InvalidSignatureFormat {
            backtrace: BT::capture(),
        }
    }

    pub fn invalid_recovery_param() -> Self {
        CryptoError::InvalidRecoveryParam {
            backtrace: BT::capture(),
        }
    }

    pub fn unknown_hash_function() -> Self {
        CryptoError::UnknownHashFunction {
            backtrace: BT::capture(),
        }
    }

    /// Numeric error code that can easily be passed over the
    /// contract VM boundary.
    pub fn code(&self) -> u32 {
        match self {
            CryptoError::InvalidHashFormat { .. } => 3,
            CryptoError::InvalidSignatureFormat { .. } => 4,
            CryptoError::InvalidPubkeyFormat { .. } => 5,
            CryptoError::InvalidRecoveryParam { .. } => 6,
            CryptoError::BatchErr { .. } => 7,
            CryptoError::InvalidPoint { .. } => 8,
            CryptoError::UnknownHashFunction { .. } => 9,
            CryptoError::GenericErr { .. } => 10,
            CryptoError::PairingEquality {
                source: PairingEquality::NotMultipleG1 { .. },
                ..
            } => 11,
            CryptoError::PairingEquality {
                source: PairingEquality::NotMultipleG2 { .. },
                ..
            } => 12,
            CryptoError::PairingEquality {
                source: PairingEquality::UnequalPointAmount { .. },
                ..
            } => 13,
            CryptoError::Aggregation {
                source: Aggregation::Empty,
                ..
            } => 14,
            CryptoError::Aggregation {
                source: Aggregation::NotMultiple { .. },
                ..
            } => 15,
        }
    }
}

impl From<Aggregation> for CryptoError {
    #[track_caller]
    fn from(value: Aggregation) -> Self {
        Self::Aggregation {
            source: value,
            backtrace: BT::capture(),
        }
    }
}

impl From<PairingEquality> for CryptoError {
    #[track_caller]
    fn from(value: PairingEquality) -> Self {
        Self::PairingEquality {
            source: value,
            backtrace: BT::capture(),
        }
    }
}

impl From<InvalidPoint> for CryptoError {
    #[track_caller]
    fn from(value: InvalidPoint) -> Self {
        Self::InvalidPoint {
            source: value,
            backtrace: BT::capture(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // constructors
    #[test]
    fn batch_err_works() {
        let error = CryptoError::batch_err("something went wrong in a batch way");
        match error {
            CryptoError::BatchErr { msg, .. } => {
                assert_eq!(msg, "something went wrong in a batch way")
            }
            _ => panic!("wrong error type!"),
        }
    }

    #[test]
    fn generic_err_works() {
        let error = CryptoError::generic_err("something went wrong in a general way");
        match error {
            CryptoError::GenericErr { msg, .. } => {
                assert_eq!(msg, "something went wrong in a general way")
            }
            _ => panic!("wrong error type!"),
        }
    }

    #[test]
    fn invalid_hash_format_works() {
        let error = CryptoError::invalid_hash_format();
        match error {
            CryptoError::InvalidHashFormat { .. } => {}
            _ => panic!("wrong error type!"),
        }
    }

    #[test]
    fn invalid_signature_format_works() {
        let error = CryptoError::invalid_signature_format();
        match error {
            CryptoError::InvalidSignatureFormat { .. } => {}
            _ => panic!("wrong error type!"),
        }
    }

    #[test]
    fn invalid_pubkey_format_works() {
        let error = CryptoError::invalid_pubkey_format();
        match error {
            CryptoError::InvalidPubkeyFormat { .. } => {}
            _ => panic!("wrong error type!"),
        }
    }
}
