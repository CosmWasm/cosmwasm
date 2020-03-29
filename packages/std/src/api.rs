/// This maintains types needed for a public API
/// In particular managing serializing and deserializing errors through API boundaries
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::errors::Error;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ApiResult<T, E: std::error::Error = ApiError> {
    Ok(T),
    Err(E),
}

impl<T, E: std::error::Error> ApiResult<T, E> {
    pub fn result<U: From<T>>(self) -> Result<U, E> {
        match self {
            ApiResult::Ok(t) => Ok(t.into()),
            ApiResult::Err(e) => Err(e),
        }
    }
}

impl<T, U: From<T>, E: std::error::Error> Into<Result<U, E>> for ApiResult<T, E> {
    fn into(self) -> Result<U, E> {
        self.result()
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

#[cfg(test)]
mod test {
    use snafu::ResultExt;

    use super::*;
    use crate::errors::{
        contract_err, dyn_contract_err, invalid, unauthorized, Base64Err, NotFound, NullPointer,
        Result, SerializeErr,
    };
    use crate::serde::{from_slice, to_vec};

    #[test]
    fn convert_ok_result() {
        let input: Result<Vec<u8>> = Ok(b"foo".to_vec());
        let convert: ApiResult<Vec<u8>> = input.into();
        assert_eq!(convert, ApiResult::Ok(b"foo".to_vec()));
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
    }

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
}
