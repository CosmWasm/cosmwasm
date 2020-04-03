use crate::HumanAddr;
use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
/// Structured error type for init, handle and query. This cannot be serialized to JSON, such that
/// it is only available within the contract and its unit tests.
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

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
/// SystemError is used for errors inside the runtime.
/// This is used on return values for Querier as a nested Result -
/// Result<Result<T, Error>, SystemError>
/// The first wrap (SystemError) will trigger if the contract address doesn't exist,
/// the QueryRequest is malformated, etc. The second wrap will be an error message from
/// the contract itself.
pub enum SystemError {
    #[snafu(display("Cannot parse request: {}", error))]
    InvalidRequest {
        error: String,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("No such contract: {}", addr))]
    NoSuchContract {
        addr: HumanAddr,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Unknown system error"))]
    Unknown { backtrace: snafu::Backtrace },
}

/// Result for init, handle and query. Since the error type cannot be serialized to JSON,
/// this is only available within the contract and its unit tests.
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

#[cfg(test)]
mod test {
    use super::*;

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
}
