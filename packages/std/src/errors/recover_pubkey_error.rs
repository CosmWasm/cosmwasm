#[cfg(not(target_arch = "wasm32"))]
use cosmwasm_crypto::CryptoError;
#[cfg(feature = "backtraces")]
use std::backtrace::Backtrace;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecoverPubkeyError {
    #[error("Hash error")]
    HashErr,
    #[error("Signature error")]
    SignatureErr,
    #[error("Invalid recovery parameter. Supported values: 0 and 1.")]
    InvalidRecoveryParam,
    #[error("Unknown error: {error_code}")]
    UnknownErr {
        error_code: u32,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
}

impl RecoverPubkeyError {
    pub fn unknown_err(error_code: u32) -> Self {
        RecoverPubkeyError::UnknownErr {
            error_code,
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }
}

impl PartialEq<RecoverPubkeyError> for RecoverPubkeyError {
    fn eq(&self, rhs: &RecoverPubkeyError) -> bool {
        match self {
            RecoverPubkeyError::HashErr => matches!(rhs, RecoverPubkeyError::HashErr),
            RecoverPubkeyError::SignatureErr => matches!(rhs, RecoverPubkeyError::SignatureErr),
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
            CryptoError::MessageError { .. } => panic!("Conversion not supported"),
            CryptoError::HashErr { .. } => RecoverPubkeyError::HashErr,
            CryptoError::SignatureErr { .. } => RecoverPubkeyError::SignatureErr,
            CryptoError::PublicKeyErr { .. } => panic!("Conversion not supported"),
            CryptoError::GenericErr { .. } => panic!("Conversion not supported"),
            CryptoError::InvalidRecoveryParam { .. } => RecoverPubkeyError::InvalidRecoveryParam,
        }
    }
}
