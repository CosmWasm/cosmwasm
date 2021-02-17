#[cfg(feature = "backtraces")]
use std::backtrace::Backtrace;
use std::fmt::Debug;
use thiserror::Error;

#[cfg(not(target_arch = "wasm32"))]
use cosmwasm_crypto::CryptoError;

#[derive(Error, Debug)]
pub enum VerificationError {
    #[error("Generic error")]
    GenericErr,
    #[error("Message error")]
    MessageErr,
    #[error("Hash error")]
    HashErr,
    #[error("Signature error")]
    SignatureErr,
    #[error("Public key error")]
    PublicKeyErr,
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
            VerificationError::GenericErr => matches!(rhs, VerificationError::GenericErr),
            VerificationError::MessageErr => matches!(rhs, VerificationError::MessageErr),
            VerificationError::HashErr => matches!(rhs, VerificationError::HashErr),
            VerificationError::SignatureErr => matches!(rhs, VerificationError::SignatureErr),
            VerificationError::PublicKeyErr => matches!(rhs, VerificationError::PublicKeyErr),
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
            CryptoError::MessageError { .. } => VerificationError::MessageErr,
            CryptoError::HashErr { .. } => VerificationError::HashErr,
            CryptoError::SignatureErr { .. } => VerificationError::SignatureErr,
            CryptoError::PublicKeyErr { .. } => VerificationError::PublicKeyErr,
            CryptoError::GenericErr { .. } => VerificationError::GenericErr,
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
