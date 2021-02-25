#[cfg(feature = "backtraces")]
use std::backtrace::Backtrace;
use std::fmt::Debug;
use thiserror::Error;

pub type CryptoResult<T> = core::result::Result<T, CryptoError>;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Batch verify error: {msg}")]
    BatchErr {
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Crypto error: {msg}")]
    GenericErr {
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Invalid hash format")]
    InvalidHashFormat {
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Invalid signature format")]
    InvalidSignatureFormat {
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Message is longer than supported by this implementation (Limit: {limit}, actual length: {actual})")]
    MessageTooLong {
        limit: usize,
        actual: usize,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Public key error: {msg}")]
    PublicKeyErr {
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Invalid recovery parameter. Supported values: 0 and 1.")]
    InvalidRecoveryParam {
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
}

impl CryptoError {
    pub fn batch_err<S: Into<String>>(msg: S) -> Self {
        CryptoError::BatchErr {
            msg: msg.into(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn generic_err<S: Into<String>>(msg: S) -> Self {
        CryptoError::GenericErr {
            msg: msg.into(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn invalid_hash_format() -> Self {
        CryptoError::InvalidHashFormat {
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn invalid_signature_format() -> Self {
        CryptoError::InvalidSignatureFormat {
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn message_too_long(limit: usize, actual: usize) -> Self {
        CryptoError::MessageTooLong {
            limit,
            actual,
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn pubkey_err<S: Into<String>>(msg: S) -> Self {
        CryptoError::PublicKeyErr {
            msg: msg.into(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn invalid_recovery_param() -> Self {
        CryptoError::InvalidRecoveryParam {
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    /// Numeric error code that can easily be passed over the
    /// contract VM boundary.
    pub fn code(&self) -> u32 {
        match self {
            CryptoError::MessageTooLong { .. } => 2,
            CryptoError::InvalidHashFormat { .. } => 3,
            CryptoError::InvalidSignatureFormat { .. } => 4,
            CryptoError::PublicKeyErr { .. } => 5,
            CryptoError::InvalidRecoveryParam { .. } => 6,
            CryptoError::BatchErr { .. } => 7,
            CryptoError::GenericErr { .. } => 10,
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
    fn message_too_long_works() {
        let error = CryptoError::message_too_long(5, 7);
        match error {
            CryptoError::MessageTooLong { limit, actual, .. } => {
                assert_eq!(limit, 5);
                assert_eq!(actual, 7);
            }
            _ => panic!("wrong error type!"),
        }
    }

    #[test]
    fn pubkey_err_works() {
        let error = CryptoError::pubkey_err("something went wrong with the pubkey");
        match error {
            CryptoError::PublicKeyErr { msg, .. } => {
                assert_eq!(msg, "something went wrong with the pubkey")
            }
            _ => panic!("wrong error type!"),
        }
    }
}
