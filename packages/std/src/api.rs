/// This maintains types needed for a public API
/// In particular managing serializing and deserializing errors through API boundaries
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::errors::{Error, SystemError};
use crate::HumanAddr;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApiResult<T, E: std::error::Error = ApiError> {
    Ok(T),
    Err(E),
}

impl<T: Into<U>, U, E: std::error::Error> Into<Result<U, E>> for ApiResult<T, E> {
    fn into(self) -> Result<U, E> {
        match self {
            ApiResult::Ok(t) => Ok(t.into()),
            ApiResult::Err(e) => Err(e),
        }
    }
}

impl<T, U: Into<T>, E: std::error::Error, F: Into<E>> From<Result<U, F>> for ApiResult<T, E> {
    fn from(res: Result<U, F>) -> Self {
        match res {
            Ok(t) => ApiResult::Ok(t.into()),
            Err(e) => ApiResult::Err(e.into()),
        }
    }
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
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ApiError {
    Base64Err { source: String },
    ContractErr { msg: String },
    DynContractErr { msg: String },
    NotFound { kind: String },
    NullPointer {},
    ParseErr { kind: String, source: String },
    SerializeErr { kind: String, source: String },
    // This is used for std::str::from_utf8, which we may well deprecate
    Utf8Err { source: String },
    // This is used for String::from_utf8, which does zero-copy from Vec<u8>, moving towards this
    Utf8StringErr { source: String },
    Unauthorized {},
    ValidationErr { field: String, msg: String },
}

impl std::error::Error for ApiError {}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Base64Err { source } => write!(f, "Invalid Base64 string: {}", source),
            ApiError::ContractErr { msg } => write!(f, "Contract error: {}", msg),
            ApiError::DynContractErr { msg } => write!(f, "Contract error: {}", msg),
            ApiError::NotFound { kind } => write!(f, "{} not found", kind),
            ApiError::NullPointer {} => write!(f, "Received null pointer, refuse to use"),
            ApiError::ParseErr { kind, source } => write!(f, "Error parsing {}: {}", kind, source),
            ApiError::SerializeErr { kind, source } => {
                write!(f, "Error serializing {}: {}", kind, source)
            }
            ApiError::Utf8Err { source } => write!(f, "UTF8 encoding error: {}", source),
            ApiError::Utf8StringErr { source } => write!(f, "UTF8 encoding error: {}", source),
            ApiError::Unauthorized {} => write!(f, "Unauthorized"),
            ApiError::ValidationErr { field, msg } => write!(f, "Invalid {}: {}", field, msg),
        }
    }
}

impl From<Error> for ApiError {
    fn from(value: Error) -> Self {
        match value {
            Error::Base64Err { source, .. } => ApiError::Base64Err {
                source: format!("{}", source),
            },
            Error::ContractErr { msg, .. } => ApiError::ContractErr {
                msg: msg.to_string(),
            },
            Error::DynContractErr { msg, .. } => ApiError::DynContractErr { msg },
            Error::NotFound { kind, .. } => ApiError::NotFound {
                kind: kind.to_string(),
            },
            Error::NullPointer { .. } => ApiError::NullPointer {},
            Error::ParseErr { kind, source, .. } => ApiError::ParseErr {
                kind: kind.to_string(),
                source: format!("{}", source),
            },
            Error::SerializeErr { kind, source, .. } => ApiError::SerializeErr {
                kind: kind.to_string(),
                source: format!("{}", source),
            },
            Error::Utf8Err { source, .. } => ApiError::Utf8Err {
                source: format!("{}", source),
            },
            Error::Utf8StringErr { source, .. } => ApiError::Utf8StringErr {
                source: format!("{}", source),
            },
            Error::Unauthorized { .. } => ApiError::Unauthorized {},
            Error::ValidationErr { field, msg, .. } => ApiError::ValidationErr {
                field: field.to_string(),
                msg: msg.to_string(),
            },
        }
    }
}

/// ApiSystemError is an "api friendly" version of SystemError, just as ApiError
/// is an "api friendly" version of Error
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ApiSystemError {
    InvalidRequest { source: String },
    NoSuchContract { addr: HumanAddr },
}

impl std::error::Error for ApiSystemError {}

impl std::fmt::Display for ApiSystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiSystemError::InvalidRequest { source } => {
                write!(f, "Cannot parse request: {}", source)
            }
            ApiSystemError::NoSuchContract { addr } => write!(f, "No such contract: {}", addr),
        }
    }
}

impl From<SystemError> for ApiSystemError {
    fn from(value: SystemError) -> Self {
        match value {
            SystemError::InvalidRequest { source, .. } => ApiSystemError::InvalidRequest {
                source: format!("{}", source),
            },
            SystemError::NoSuchContract { addr, .. } => ApiSystemError::NoSuchContract { addr },
        }
    }
}

#[cfg(test)]
mod test_result {
    use super::*;
    use crate::errors::{contract_err, NoSuchContract, Result};
    use crate::serde::{from_slice, to_vec};

    #[test]
    fn convert_ok_result() {
        let input: Result<Vec<u8>> = Ok(b"foo".to_vec());
        let convert: ApiResult<Vec<u8>> = input.into();
        assert_eq!(convert, ApiResult::Ok(b"foo".to_vec()));
    }

    #[test]
    fn check_ok_into_conversion() {
        let input: Result<bool> = Ok(true);
        let convert: ApiResult<i32> = input.into();
        assert_eq!(convert, ApiResult::Ok(1i32));
        let expanded: Result<i64, ApiError> = convert.into();
        assert!(expanded.is_ok());
        assert_eq!(expanded.unwrap(), 1i64);
    }

    #[test]
    fn convert_err_result() {
        let input: Result<()> = contract_err("sample error");
        let convert: ApiResult<()> = input.into();
        assert_eq!(
            convert,
            ApiResult::Err(ApiError::ContractErr {
                msg: "sample error".to_string()
            })
        );
        let reconvert: Result<(), ApiError> = convert.into();
        match reconvert {
            Ok(_) => panic!("must be error"),
            Err(e) => assert_eq!(
                e,
                ApiError::ContractErr {
                    msg: "sample error".to_string()
                }
            ),
        }
    }

    #[test]
    fn convert_sys_err_result() {
        let input: Result<(), SystemError> = NoSuchContract {
            addr: HumanAddr::from("bad_address"),
        }
        .fail();
        let convert: ApiResult<(), ApiSystemError> = input.into();
        assert_eq!(
            convert,
            ApiResult::Err(ApiSystemError::NoSuchContract {
                addr: HumanAddr::from("bad_address"),
            })
        );
    }

    #[test]
    // this tests Ok(Err(_)) case for SystemError, Error
    fn convert_nested_ok_err_result() {
        let input: Result<Result<()>, SystemError> = Ok(contract_err("nested error"));
        let convert: ApiResult<ApiResult<()>, ApiSystemError> = input.into();
        assert_eq!(
            convert,
            ApiResult::Ok(ApiResult::Err(ApiError::ContractErr {
                msg: "nested error".to_string()
            }))
        );
    }

    #[test]
    // this tests Ok(Ok(_)) case for SystemError, Error
    fn convert_nested_ok_ok_result() {
        let input: Result<Result<i32>, SystemError> = Ok(Ok(123));
        let convert: ApiResult<ApiResult<i32>, ApiSystemError> = input.into();
        assert_eq!(convert, ApiResult::Ok(ApiResult::Ok(123)),);
    }

    #[test]
    // make sure we can shove this all over API boundaries
    fn serialize_and_recover_nested_result() {
        let input: Result<Result<()>, SystemError> = Ok(contract_err("over ffi"));
        let convert: ApiResult<ApiResult<()>, ApiSystemError> = input.into();
        let recovered: ApiResult<ApiResult<(), ApiError>, ApiSystemError> =
            from_slice(&to_vec(&convert).unwrap()).unwrap();
        assert_eq!(
            recovered,
            ApiResult::Ok(ApiResult::Err(ApiError::ContractErr {
                msg: "over ffi".to_string()
            }))
        );
        // into handles nested errors
        let recovered_result: Result<Result<(), ApiError>, ApiSystemError> = recovered.into();
        let wrapped_err = recovered_result.unwrap().unwrap_err();
        assert_eq!(
            wrapped_err,
            ApiError::ContractErr {
                msg: "over ffi".to_string()
            }
        );
    }
}

#[cfg(test)]
mod test_errors {
    use snafu::ResultExt;

    use super::*;
    use crate::errors::{
        contract_err, dyn_contract_err, invalid, unauthorized, Base64Err, InvalidRequest,
        NoSuchContract, NotFound, NullPointer, Result, SerializeErr,
    };
    use crate::serde::{from_slice, to_vec};

    fn assert_conversion(r: Result<()>) {
        let error = r.unwrap_err();
        let msg = format!("{}", error);
        let converted: ApiError = error.into();
        assert_eq!(msg, format!("{}", converted));
        let round_trip: ApiError = from_slice(&to_vec(&converted).unwrap()).unwrap();
        assert_eq!(round_trip, converted);
    }

    #[test]
    fn base64_conversion() {
        let source = Err(base64::DecodeError::InvalidLength);
        assert_conversion(source.context(Base64Err {}));
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

    fn assert_system_conversion(r: Result<(), SystemError>) {
        let error = r.unwrap_err();
        let msg = format!("{}", error);
        let converted: ApiSystemError = error.into();
        assert_eq!(msg, format!("{}", converted));
        let round_trip: ApiSystemError = from_slice(&to_vec(&converted).unwrap()).unwrap();
        assert_eq!(round_trip, converted);
    }

    #[test]
    fn invalid_request_conversion() {
        let err = Err(serde_json_wasm::de::Error::ExpectedSomeValue).context(InvalidRequest {});
        assert_system_conversion(err);
    }

    #[test]
    fn no_such_contract_conversion() {
        let err = NoSuchContract {
            addr: HumanAddr::from("bad_address"),
        }
        .fail();
        assert_system_conversion(err);
    }
}
