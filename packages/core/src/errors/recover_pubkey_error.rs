use core::fmt::Debug;
#[cfg(not(target_arch = "wasm32"))]
use cosmwasm_crypto::CryptoError;
use derive_more::Display;

use super::BT;

#[derive(Display, Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum RecoverPubkeyError {
    #[display("Invalid hash format")]
    InvalidHashFormat,
    #[display("Invalid signature format")]
    InvalidSignatureFormat,
    #[display("Invalid recovery parameter. Supported values: 0 and 1.")]
    InvalidRecoveryParam,
    #[display("Unknown error: {error_code}")]
    UnknownErr { error_code: u32, backtrace: BT },
}

impl RecoverPubkeyError {
    pub fn unknown_err(error_code: u32) -> Self {
        RecoverPubkeyError::UnknownErr {
            error_code,

            backtrace: BT::capture(),
        }
    }
}

impl PartialEq<RecoverPubkeyError> for RecoverPubkeyError {
    fn eq(&self, rhs: &RecoverPubkeyError) -> bool {
        match self {
            RecoverPubkeyError::InvalidHashFormat => {
                matches!(rhs, RecoverPubkeyError::InvalidHashFormat)
            }
            RecoverPubkeyError::InvalidSignatureFormat => {
                matches!(rhs, RecoverPubkeyError::InvalidSignatureFormat)
            }
            RecoverPubkeyError::InvalidRecoveryParam => {
                matches!(rhs, RecoverPubkeyError::InvalidRecoveryParam)
            }
            RecoverPubkeyError::UnknownErr { error_code, .. } => {
                if let RecoverPubkeyError::UnknownErr {
                    error_code: rhs_error_code,
                    ..
                } = rhs
                {
                    error_code == rhs_error_code
                } else {
                    false
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<CryptoError> for RecoverPubkeyError {
    fn from(original: CryptoError) -> Self {
        match original {
            CryptoError::InvalidHashFormat { .. } => RecoverPubkeyError::InvalidHashFormat,
            CryptoError::InvalidSignatureFormat { .. } => {
                RecoverPubkeyError::InvalidSignatureFormat
            }
            CryptoError::GenericErr { .. } => RecoverPubkeyError::unknown_err(original.code()),
            CryptoError::InvalidRecoveryParam { .. } => RecoverPubkeyError::InvalidRecoveryParam,
            CryptoError::Aggregation { .. }
            | CryptoError::AggregationPairingEquality { .. }
            | CryptoError::BatchErr { .. }
            | CryptoError::InvalidPubkeyFormat { .. }
            | CryptoError::InvalidPoint { .. }
            | CryptoError::UnknownHashFunction { .. } => panic!("Conversion not supported"),
        }
    }
}
