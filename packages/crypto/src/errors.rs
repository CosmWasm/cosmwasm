#[cfg(feature = "backtraces")]
use std::backtrace::Backtrace;
use std::fmt::Debug;
use thiserror::Error;

pub type CryptoResult<T> = core::result::Result<T, CryptoError>;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum CryptoError {
    #[error("Crypto error: {msg}")]
    GenericErr {
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
}

impl CryptoError {
    pub fn generic_err<S: Into<String>>(msg: S) -> Self {
        CryptoError::GenericErr {
            msg: msg.into(),
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
        let error = CryptoError::generic_err("something went wrong");
        match error {
            CryptoError::GenericErr { msg, .. } => assert_eq!(msg, "something went wrong"),
        }
    }
}
