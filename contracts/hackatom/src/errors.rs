use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HackError {
    #[error("{0}")]
    /// this is needed so we can use `bucket.load(...)?` and have it auto-converted to the custom error
    Std(StdError),
    // this is whatever we want
    #[error("Unauthorized")]
    Unauthorized {},
    // this is whatever we want
    #[error("Downgrade is not supported")]
    Downgrade,
}

impl From<StdError> for HackError {
    fn from(err: StdError) -> Self {
        Self::Std(err)
    }
}
