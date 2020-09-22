use cosmwasm_std::StdError;
use snafu::Snafu;

#[derive(Snafu, Debug)]
#[snafu(visibility = "pub(crate)")]
pub enum StakingError {
    /// this is needed so we can use `bucket.load(...)?` and have it auto-converted to the custom error
    #[snafu(display("StdError: {}", original))]
    Std { original: StdError },
    #[snafu(display("Unauthorized"))]
    Unauthorized { backtrace: Option<snafu::Backtrace> },
}

impl From<StdError> for StakingError {
    fn from(original: StdError) -> Self {
        Std { original }.build()
    }
}
