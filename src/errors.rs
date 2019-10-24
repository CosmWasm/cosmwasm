use snafu::{Snafu};

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Received null pointer, refuse to use"))]
    NullPointer { },
    #[snafu(display("JSON error: {}", source))]
    JsonError { source: serde_json::Error },
    #[snafu(display("Contract error: {}", msg))]
    ContractErr { msg: String },
    #[snafu(display("Unauthorized"))]
    Unauthorized { },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;