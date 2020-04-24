/// This maintains types needed for a public API
/// In particular managing serializing and deserializing errors through API boundaries
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::errors::StdError;
use crate::HumanAddr;

pub type ApiResult<T> = Result<T, ApiError>;

/// We neither "own" StdResult nor ApiResult, since those are just aliases to the external
/// std::result::Result. For this reason, we cannot add trait implementations like Into or From.
/// But we can achive all we need from outside interfaces of StdResult and ApiResult.
pub fn to_api_result<T>(result: crate::errors::StdResult<T>) -> ApiResult<T> {
    result.map_err(|std_err| std_err.into())
}

/// ApiError is a "converted" Error that can be serialized and deserialized.
/// It can be created via `error.into()`
/// This will not contain all information of the original (source error and backtrace cannot be serialized),
/// but we ensure the following:
/// 1. An ApiError will have the same type as the original Error
/// 2. An ApiError will have the same display as the original
/// 3. Serializing and deserializing an ApiError will give you an identical struct
///
/// Rather than use Display to pass Errors over API/FFI boundaries, we can use ApiError
/// and provide much more context to the client.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApiError {
    ContractErr { msg: String },
    DynContractErr { msg: String },
    InvalidBase64 { msg: String },
    NotFound { kind: String },
    NullPointer {},
    ParseErr { kind: String, source: String },
    SerializeErr { kind: String, source: String },
    Unauthorized {},
    Underflow { minuend: String, subtrahend: String },
    // This is used for std::str::from_utf8, which we may well deprecate
    Utf8Err { source: String },
    // This is used for String::from_utf8, which does zero-copy from Vec<u8>, moving towards this
    Utf8StringErr { source: String },
    ValidationErr { field: String, msg: String },
}

impl std::error::Error for ApiError {}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::ContractErr { msg } => write!(f, "Contract error: {}", msg),
            ApiError::DynContractErr { msg } => write!(f, "Contract error: {}", msg),
            ApiError::InvalidBase64 { msg } => write!(f, "Invalid Base64 string: {}", msg),
            ApiError::NotFound { kind } => write!(f, "{} not found", kind),
            ApiError::NullPointer {} => write!(f, "Received null pointer, refuse to use"),
            ApiError::ParseErr { kind, source } => write!(f, "Error parsing {}: {}", kind, source),
            ApiError::SerializeErr { kind, source } => {
                write!(f, "Error serializing {}: {}", kind, source)
            }
            ApiError::Unauthorized {} => write!(f, "Unauthorized"),
            ApiError::Underflow {
                minuend,
                subtrahend,
            } => write!(f, "Cannot subtract {} from {}", subtrahend, minuend),
            ApiError::Utf8Err { source } => write!(f, "UTF8 encoding error: {}", source),
            ApiError::Utf8StringErr { source } => write!(f, "UTF8 encoding error: {}", source),
            ApiError::ValidationErr { field, msg } => write!(f, "Invalid {}: {}", field, msg),
        }
    }
}

impl From<StdError> for ApiError {
    fn from(value: StdError) -> Self {
        match value {
            StdError::ContractErr { msg, .. } => ApiError::ContractErr {
                msg: msg.to_string(),
            },
            StdError::DynContractErr { msg, .. } => ApiError::DynContractErr { msg },
            StdError::InvalidBase64 { msg, .. } => ApiError::InvalidBase64 { msg },
            StdError::NotFound { kind, .. } => ApiError::NotFound {
                kind: kind.to_string(),
            },
            StdError::NullPointer { .. } => ApiError::NullPointer {},
            StdError::ParseErr { kind, source, .. } => ApiError::ParseErr {
                kind: kind.to_string(),
                source: format!("{}", source),
            },
            StdError::SerializeErr { kind, source, .. } => ApiError::SerializeErr {
                kind: kind.to_string(),
                source: format!("{}", source),
            },
            StdError::Unauthorized { .. } => ApiError::Unauthorized {},
            StdError::Underflow {
                minuend,
                subtrahend,
                ..
            } => ApiError::Underflow {
                minuend,
                subtrahend,
            },
            StdError::Utf8Err { source, .. } => ApiError::Utf8Err {
                source: format!("{}", source),
            },
            StdError::Utf8StringErr { source, .. } => ApiError::Utf8StringErr {
                source: format!("{}", source),
            },
            StdError::ValidationErr { field, msg, .. } => ApiError::ValidationErr {
                field: field.to_string(),
                msg: msg.to_string(),
            },
        }
    }
}

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
    use snafu::ResultExt;

    use super::*;
    use crate::errors::{
        contract_err, dyn_contract_err, invalid, unauthorized, InvalidBase64, NotFound,
        NullPointer, SerializeErr, StdResult,
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
    fn to_api_result_works_for_ok() {
        let input: StdResult<Vec<u8>> = Ok(b"foo".to_vec());
        assert_eq!(to_api_result(input), ApiResult::Ok(b"foo".to_vec()));
    }

    #[test]
    fn to_api_result_works_for_err() {
        let input: StdResult<()> = contract_err("sample error");
        assert_eq!(
            to_api_result(input),
            ApiResult::Err(ApiError::ContractErr {
                msg: "sample error".to_string()
            })
        );
    }

    #[test]
    fn contract_conversion() {
        assert_conversion(contract_err("foobar"));
    }

    #[test]
    fn dyn_contract_conversion() {
        assert_conversion(dyn_contract_err("dynamic".to_string()));
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
    fn invalid_conversion() {
        assert_conversion(invalid("name", "too short"));
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
        let source = Err(serde_json_wasm::ser::Error::BufferFull);
        assert_conversion(source.context(SerializeErr { kind: "faker" }));
    }
}
