use cosmwasm_std::{CanonicalAddr, StdError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReflectError {
    #[error("{}", original)]
    /// this is needed so we can use `bucket.load(...)?` and have it auto-converted to the custom error
    Std {
        #[from]
        original: StdError,
    },
    // this is whatever we want
    #[error("Permission denied: the sender is not the current owner")]
    NotCurrentOwner {
        expected: CanonicalAddr,
        actual: CanonicalAddr,
    },
    #[error("Messages empty. Must reflect at least one message")]
    MessagesEmpty,
}
