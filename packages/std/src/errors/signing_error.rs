#[cfg(feature = "backtraces")]
use std::backtrace::Backtrace;
use std::fmt::Debug;
use thiserror::Error;

#[cfg(not(target_arch = "wasm32"))]
use secret_cosmwasm_crypto::CryptoError;

#[derive(Error, Debug)]
pub enum SigningError {
    #[error("Invalid private key format")]
    InvalidPrivateKeyFormat,
    #[error("Unknown error: {error_code}")]
    UnknownErr {
        error_code: u32,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
}

impl SigningError {
    pub fn unknown_err(error_code: u32) -> Self {
        SigningError::UnknownErr {
            error_code,
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }
}

impl PartialEq<SigningError> for SigningError {
    fn eq(&self, rhs: &SigningError) -> bool {
        match self {
            SigningError::InvalidPrivateKeyFormat => {
                matches!(rhs, SigningError::InvalidPrivateKeyFormat)
            }
            SigningError::UnknownErr { error_code, .. } => {
                if let SigningError::UnknownErr {
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
impl From<CryptoError> for SigningError {
    fn from(original: CryptoError) -> Self {
        match original {
            CryptoError::InvalidPrivateKeyFormat { .. } => SigningError::InvalidPrivateKeyFormat,
            _ => SigningError::UnknownErr {
                error_code: 0,
                #[cfg(feature = "backtraces")]
                backtrace: Backtrace::capture(),
            }, // should never get here
        }
    }
}
