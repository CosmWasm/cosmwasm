use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    /// this is needed so we can use `bucket.load(...)?` and have it auto-converted to the custom error
    Std(StdError),
}

impl From<StdError> for ContractError {
    fn from(err: StdError) -> Self {
        Self::Std(err)
    }
}
