use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::HumanAddr;

/// SystemError is used for errors inside the VM and is API frindly (i.e. serializable).
///
/// This is used on return values for Querier as a nested result: Result<StdResult<T>, SystemError>
/// The first wrap (SystemError) will trigger if the contract address doesn't exist,
/// the QueryRequest is malformated, etc. The second wrap will be an error message from
/// the contract itself.
///
/// Such errors are only created by the VM. The error type is defined in the standard library, to ensure
/// the contract understands the error format without creating a dependency on cosmwasm-vm.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SystemError {
    InvalidRequest { error: String },
    NoSuchContract { addr: HumanAddr },
    Unknown {},
    UnsupportedRequest { kind: String },
}

impl std::error::Error for SystemError {}

impl std::fmt::Display for SystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemError::InvalidRequest { error } => write!(f, "Cannot parse request: {}", error),
            SystemError::NoSuchContract { addr } => write!(f, "No such contract: {}", addr),
            SystemError::Unknown {} => write!(f, "Unknown system error"),
            SystemError::UnsupportedRequest { kind } => write!(f, "Unsupport query type: {}", kind),
        }
    }
}

pub type SystemResult<T> = Result<T, SystemError>;
