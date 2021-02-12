#[cfg(feature = "backtraces")]
use std::backtrace::Backtrace;
use std::fmt::Debug;
use thiserror::Error;

pub type CryptoResult<T> = core::result::Result<T, CryptoError>;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Crypto error: {msg}")]
    GenericErr {
        msg: String,
        error_code: u32,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Hash error: {msg}")]
    HashErr {
        msg: String,
        error_code: u32,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Message error: {msg}")]
    MsgErr {
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Signature error: {msg}")]
    SignatureErr {
        msg: String,
        error_code: u32,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Public key error: {msg}")]
    PublicKeyErr {
        msg: String,
        error_code: u32,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
}

impl CryptoError {
    pub fn generic_err<S: Into<String>>(msg: S) -> Self {
        CryptoError::GenericErr {
            msg: msg.into(),
            error_code: 10,
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn hash_err<S: Into<String>>(msg: S) -> Self {
        CryptoError::HashErr {
            msg: msg.into(),
            error_code: 3,
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn msg_err<S: Into<String>>(msg: S) -> Self {
        CryptoError::MsgErr {
            msg: msg.into(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn sig_err<S: Into<String>>(msg: S) -> Self {
        CryptoError::SignatureErr {
            msg: msg.into(),
            error_code: 4,
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn pubkey_err<S: Into<String>>(msg: S) -> Self {
        CryptoError::PublicKeyErr {
            msg: msg.into(),
            error_code: 5,
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // constructors
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
    fn hash_err_works() {
        let error = CryptoError::hash_err("something went wrong with the hash");
        match error {
            CryptoError::HashErr { msg, .. } => {
                assert_eq!(msg, "something went wrong with the hash")
            }
            _ => panic!("wrong error type!"),
        }
    }

    #[test]
    fn sig_err_works() {
        let error = CryptoError::sig_err("something went wrong with the sig");
        match error {
            CryptoError::SignatureErr { msg, .. } => {
                assert_eq!(msg, "something went wrong with the sig")
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
