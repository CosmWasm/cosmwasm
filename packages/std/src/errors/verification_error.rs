#[cfg(feature = "backtraces")]
use std::backtrace::Backtrace;
use std::fmt::Debug;
use thiserror::Error;

#[cfg(not(target_arch = "wasm32"))]
use cosmwasm_crypto::CryptoError;

#[derive(Error, Debug)]
pub enum VerificationError {
    #[error("Batch error")]
    BatchErr,
    #[error("Generic error")]
    GenericErr,
    #[error("Message is longer than supported")]
    MessageTooLong,
    #[error("Invalid hash format")]
    InvalidHashFormat,
    #[error("Invalid signature format")]
    InvalidSignatureFormat,
    #[error("Public key error")]
    PublicKeyErr,
    #[error("Invalid recovery parameter. Supported values: 0 and 1.")]
    InvalidRecoveryParam,
    #[error("Unknown error: {error_code}")]
    UnknownErr {
        error_code: u32,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
}

impl VerificationError {
    pub fn unknown_err(error_code: u32) -> Self {
        VerificationError::UnknownErr {
            error_code,
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }
}

impl PartialEq<VerificationError> for VerificationError {
    fn eq(&self, rhs: &VerificationError) -> bool {
        match self {
            VerificationError::BatchErr => matches!(rhs, VerificationError::BatchErr),
            VerificationError::GenericErr => matches!(rhs, VerificationError::GenericErr),
            VerificationError::MessageTooLong => matches!(rhs, VerificationError::MessageTooLong),
            VerificationError::InvalidHashFormat => {
                matches!(rhs, VerificationError::InvalidHashFormat)
            }
            VerificationError::InvalidSignatureFormat => {
                matches!(rhs, VerificationError::InvalidSignatureFormat)
            }
            VerificationError::PublicKeyErr => matches!(rhs, VerificationError::PublicKeyErr),
            VerificationError::InvalidRecoveryParam => {
                matches!(rhs, VerificationError::InvalidRecoveryParam)
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
            CryptoError::MessageTooLong { .. } => VerificationError::MessageTooLong,
            CryptoError::InvalidHashFormat { .. } => VerificationError::InvalidHashFormat,
            CryptoError::InvalidSignatureFormat { .. } => VerificationError::InvalidSignatureFormat,
            CryptoError::PublicKeyErr { .. } => VerificationError::PublicKeyErr,
            CryptoError::GenericErr { .. } => VerificationError::GenericErr,
            CryptoError::InvalidRecoveryParam { .. } => VerificationError::InvalidRecoveryParam,
            CryptoError::BatchErr { .. } => VerificationError::BatchErr,
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
