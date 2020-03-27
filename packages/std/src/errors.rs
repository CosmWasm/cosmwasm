use serde::{Deserialize, Serialize};
use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Invalid Base64 string: {}", source))]
    Base64Err {
        source: base64::DecodeError,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Contract error: {}", msg))]
    ContractErr {
        msg: &'static str,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Contract error: {}", msg))]
    DynContractErr {
        msg: String,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("{} not found", kind))]
    NotFound {
        kind: &'static str,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Received null pointer, refuse to use"))]
    NullPointer { backtrace: snafu::Backtrace },
    #[snafu(display("Error parsing {}: {}", kind, source))]
    ParseErr {
        kind: &'static str,
        source: serde_json_wasm::de::Error,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Error serializing {}: {}", kind, source))]
    SerializeErr {
        kind: &'static str,
        source: serde_json_wasm::ser::Error,
        backtrace: snafu::Backtrace,
    },
    // This is used for std::str::from_utf8, which we may well deprecate
    #[snafu(display("UTF8 encoding error: {}", source))]
    Utf8Err {
        source: std::str::Utf8Error,
        backtrace: snafu::Backtrace,
    },
    // This is used for String::from_utf8, which does zero-copy from Vec<u8>, moving towards this
    #[snafu(display("UTF8 encoding error: {}", source))]
    Utf8StringErr {
        source: std::string::FromUtf8Error,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Unauthorized"))]
    Unauthorized { backtrace: snafu::Backtrace },
    #[snafu(display("Invalid {}: {}", field, msg))]
    ValidationErr {
        field: &'static str,
        msg: &'static str,
        backtrace: snafu::Backtrace,
    },
}

pub type Result<T, E = Error> = core::result::Result<T, E>;

pub fn invalid<T>(field: &'static str, msg: &'static str) -> Result<T> {
    ValidationErr { field, msg }.fail()
}

pub fn contract_err<T>(msg: &'static str) -> Result<T> {
    ContractErr { msg }.fail()
}

pub fn dyn_contract_err<T>(msg: String) -> Result<T> {
    DynContractErr { msg }.fail()
}

pub fn unauthorized<T>() -> Result<T> {
    Unauthorized {}.fail()
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
    use super::*;
    use crate::serde::{from_slice, to_vec};
    use snafu::ResultExt;

    #[test]
    fn use_invalid() {
        let e: Result<()> = invalid("demo", "not implemented");
        match e {
            Err(Error::ValidationErr { field, msg, .. }) => {
                assert_eq!(field, "demo");
                assert_eq!(msg, "not implemented");
            }
            Err(e) => panic!("unexpected error, {:?}", e),
            Ok(_) => panic!("invalid must return error"),
        }
    }

    #[test]
    // example of reporting static contract errors
    fn contract_helper() {
        let e: Result<()> = contract_err("not implemented");
        match e {
            Err(Error::ContractErr { msg, .. }) => {
                assert_eq!(msg, "not implemented");
            }
            Err(e) => panic!("unexpected error, {:?}", e),
            Ok(_) => panic!("contract_err must return error"),
        }
    }

    #[test]
    // example of reporting contract errors with format!
    fn dyn_contract_helper() {
        let guess = 7;
        let e: Result<()> = dyn_contract_err(format!("{} is too low", guess));
        match e {
            Err(Error::DynContractErr { msg, .. }) => {
                assert_eq!(msg, String::from("7 is too low"));
            }
            Err(e) => panic!("unexpected error, {:?}", e),
            Ok(_) => panic!("dyn_contract_err must return error"),
        }
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
        let err = from_slice::<String>(b"123")
            .context(ParseErr { kind: "String" })
            .map(|_| ());
        assert_conversion(err);
    }

    #[test]
    fn serialize_err_conversion() {
        let source = Err(serde_json_wasm::ser::Error::BufferFull);
        assert_conversion(source.context(SerializeErr { kind: "faker" }));
    }
}
