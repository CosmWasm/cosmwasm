/// This maintains types needed for a public API
/// In particular managing serializing and deserializing errors through API boundaries
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::errors::StdError;
use crate::HumanAddr;

pub type ApiError = StdError;
pub type ApiResult<T> = Result<T, ApiError>;

/// SystemError is used for errors inside the VM and is API frindly (i.e. serializable).
///
/// This is used on return values for Querier as a nested result: Result<ApiResult<T>, SystemError>
/// The first wrap (SystemError) will trigger if the contract address doesn't exist,
/// the QueryRequest is malformated, etc. The second wrap will be an error message from
/// the contract itself.
///
/// Such errors are only created by the VM. The error type is defined in the standard library, to ensure
/// the contract understands the error format without creating a dependency on cosmwasm-vm.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::errors::{
        dyn_contract_err, unauthorized, InvalidBase64, NotFound, NullPointer, SerializeErr,
        StdResult,
    };
    use crate::serde::{from_slice, to_vec};

    fn assert_conversion(r: StdResult<()>) {
        let error = r.unwrap_err();
        let msg = format!("{}", error);
        let converted: ApiError = error.into();
        assert_eq!(msg, format!("{}", converted));
        let round_trip: ApiError = from_slice(&to_vec(&converted).unwrap()).unwrap();
        assert_eq!(round_trip, converted);
    }

    #[test]
    fn dyn_contract_conversion() {
        assert_conversion(dyn_contract_err("dynamic"));
    }

    #[test]
    fn invalid_base64_conversion() {
        assert_conversion(
            InvalidBase64 {
                msg: "invalid length".to_string(),
            }
            .fail(),
        );
    }

    #[test]
    fn unauthorized_conversion() {
        assert_conversion(unauthorized());
    }

    #[test]
    fn null_pointer_conversion() {
        assert_conversion(NullPointer {}.fail());
    }

    #[test]
    fn not_found_conversion() {
        assert_conversion(NotFound { kind: "State" }.fail());
    }

    #[test]
    fn parse_err_conversion() {
        let err = from_slice::<String>(b"123").map(|_| ());
        assert_conversion(err);
    }

    #[test]
    fn serialize_err_conversion() {
        assert_conversion(
            SerializeErr {
                source: "Person",
                msg: "buffer is full",
            }
            .fail(),
        );
    }
}
