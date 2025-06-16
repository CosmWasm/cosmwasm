use cosmwasm_std::{Instantiate2AddressError, StdError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    /// this is needed so we can use `bucket.load(...)?` and have it auto-converted to the custom error
    Std(StdError),
    #[error("{0}")]
    Instantiate2Address(#[from] Instantiate2AddressError),
}

impl From<StdError> for ContractError {
    fn from(err: StdError) -> Self {
        Self::Std(err)
    }
}
