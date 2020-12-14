use cosmwasm_std::{CanonicalAddr, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ReflectError {
    #[error("{0}")]
    // let thiserror implement From<StdError> for you
    Std(#[from] StdError),
    // this is whatever we want
    #[error("Permission denied: the sender is not the current owner")]
    NotCurrentOwner {
        expected: CanonicalAddr,
        actual: CanonicalAddr,
    },
    #[error("Messages empty. Must reflect at least one message")]
    MessagesEmpty,
}
