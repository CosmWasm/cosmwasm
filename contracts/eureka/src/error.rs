use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum EurekaError {
    #[error("{0}")]
    Generic(#[from] StdError),
}
