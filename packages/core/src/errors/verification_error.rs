use core::fmt::Debug;
use derive_more::Display;

use super::BT;

#[cfg(not(target_arch = "wasm32"))]
use cosmwasm_crypto::CryptoError;

#[derive(Display, Debug, PartialEq)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum AggregationError {
    #[display("List of points is empty")]
    Empty,
    #[display("List is not an expected multiple")]
    NotMultiple,
}

#[derive(Display, Debug, PartialEq)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum PairingEqualityError {
    #[display("List of G1 points is empty")]
    EmptyG1,
    #[display("List of G2 points is empty")]
    EmptyG2,
    #[display("List is not a multiple of 48")]
    NotMultipleG1,
    #[display("List is not a multiple of 96")]
    NotMultipleG2,
    #[display("Not the same amount of points passed")]
    UnequalPointAmount,
}

#[derive(Display, Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum VerificationError {
    #[display("Aggregation error: {source}")]
    Aggregation { source: AggregationError },
    #[display("Batch error")]
    BatchErr,
    #[display("Generic error")]
    GenericErr,
    #[display("Invalid hash format")]
    InvalidHashFormat,
    #[display("Invalid signature format")]
    InvalidSignatureFormat,
    #[display("Invalid public key format")]
    InvalidPubkeyFormat,
    #[display("Invalid recovery parameter. Supported values: 0 and 1.")]
    InvalidRecoveryParam,
    #[display("Invalid point")]
    InvalidPoint,
    #[display("Unknown hash function")]
    UnknownHashFunction,
    #[display("Aggregation pairing equality error: {source}")]
    PairingEquality { source: PairingEqualityError },
    #[display("Unknown error: {error_code}")]
    UnknownErr { error_code: u32, backtrace: BT },
}

impl VerificationError {
    pub fn unknown_err(error_code: u32) -> Self {
        VerificationError::UnknownErr {
            error_code,

            backtrace: BT::capture(),
        }
    }
}

impl PartialEq<VerificationError> for VerificationError {
    fn eq(&self, rhs: &VerificationError) -> bool {
        match self {
            VerificationError::Aggregation { source: lhs_source } => {
                matches!(rhs, VerificationError::Aggregation { source: rhs_source } if rhs_source == lhs_source)
            }
            VerificationError::PairingEquality { source: lhs_source } => {
                matches!(rhs, VerificationError::PairingEquality { source: rhs_source } if rhs_source == lhs_source)
            }
            VerificationError::BatchErr => matches!(rhs, VerificationError::BatchErr),
            VerificationError::GenericErr => matches!(rhs, VerificationError::GenericErr),
            VerificationError::InvalidHashFormat => {
                matches!(rhs, VerificationError::InvalidHashFormat)
            }
            VerificationError::InvalidPubkeyFormat => {
                matches!(rhs, VerificationError::InvalidPubkeyFormat)
            }
            VerificationError::InvalidSignatureFormat => {
                matches!(rhs, VerificationError::InvalidSignatureFormat)
            }
            VerificationError::InvalidRecoveryParam => {
                matches!(rhs, VerificationError::InvalidRecoveryParam)
            }
            VerificationError::InvalidPoint => matches!(rhs, VerificationError::InvalidPoint),
            VerificationError::UnknownHashFunction => {
                matches!(rhs, VerificationError::UnknownHashFunction)
            }
            VerificationError::UnknownErr { error_code, .. } => {
                if let VerificationError::UnknownErr {
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
impl From<CryptoError> for VerificationError {
    fn from(original: CryptoError) -> Self {
        match original {
            CryptoError::Aggregation {
                source: cosmwasm_crypto::AggregationError::Empty,
                ..
            } => VerificationError::Aggregation {
                source: AggregationError::Empty,
            },
            CryptoError::Aggregation {
                source: cosmwasm_crypto::AggregationError::NotMultiple { .. },
                ..
            } => VerificationError::Aggregation {
                source: AggregationError::NotMultiple,
            },
            CryptoError::PairingEquality {
                source: cosmwasm_crypto::PairingEqualityError::EmptyG1,
                ..
            } => VerificationError::PairingEquality {
                source: PairingEqualityError::EmptyG1,
            },
            CryptoError::PairingEquality {
                source: cosmwasm_crypto::PairingEqualityError::EmptyG2,
                ..
            } => VerificationError::PairingEquality {
                source: PairingEqualityError::EmptyG2,
            },
            CryptoError::PairingEquality {
                source: cosmwasm_crypto::PairingEqualityError::NotMultipleG1 { .. },
                ..
            } => VerificationError::PairingEquality {
                source: PairingEqualityError::NotMultipleG1,
            },
            CryptoError::PairingEquality {
                source: cosmwasm_crypto::PairingEqualityError::NotMultipleG2 { .. },
                ..
            } => VerificationError::PairingEquality {
                source: PairingEqualityError::NotMultipleG2,
            },
            CryptoError::PairingEquality {
                source: cosmwasm_crypto::PairingEqualityError::UnequalPointAmount { .. },
                ..
            } => VerificationError::PairingEquality {
                source: PairingEqualityError::UnequalPointAmount,
            },
            CryptoError::InvalidHashFormat { .. } => VerificationError::InvalidHashFormat,
            CryptoError::InvalidPubkeyFormat { .. } => VerificationError::InvalidPubkeyFormat,
            CryptoError::InvalidSignatureFormat { .. } => VerificationError::InvalidSignatureFormat,
            CryptoError::GenericErr { .. } => VerificationError::GenericErr,
            CryptoError::InvalidRecoveryParam { .. } => VerificationError::InvalidRecoveryParam,
            CryptoError::InvalidPoint { .. } => VerificationError::InvalidPoint,
            CryptoError::BatchErr { .. } => VerificationError::BatchErr,
            CryptoError::UnknownHashFunction { .. } => VerificationError::UnknownHashFunction,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // constructors
    #[test]
    fn unknown_err_works() {
        let error = VerificationError::unknown_err(123);
        match error {
            VerificationError::UnknownErr { error_code, .. } => assert_eq!(error_code, 123),
            _ => panic!("wrong error type!"),
        }
    }
}
