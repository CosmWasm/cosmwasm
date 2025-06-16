use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReflectError {
    #[error("{0}")]
    // let thiserror implement From<StdError> for you
    Std(StdError),
    // this is whatever we want
    #[error("Permission denied: the sender is not the current owner")]
    NotCurrentOwner { expected: String, actual: String },
    #[error("Messages empty. Must reflect at least one message")]
    MessagesEmpty,
}

impl From<StdError> for ReflectError {
    fn from(err: StdError) -> Self {
        Self::Std(err)
    }
}
