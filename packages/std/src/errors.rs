use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
/// Structured error type for init, handle and query. This cannot be serialized to JSON, such that
/// it is only available within the contract and its unit tests.
///
/// The prefix "Std" means "the standard error within the standard library". This is not the only
/// result/error type in cosmwasm-std.
pub enum StdError {
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
    #[snafu(display("Invalid Base64 string: {}", msg))]
    InvalidBase64 {
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
    #[snafu(display("Unauthorized"))]
    Unauthorized { backtrace: snafu::Backtrace },
    #[snafu(display("Cannot subtract {} from {}", subtrahend, minuend))]
    Underflow {
        minuend: String,
        subtrahend: String,
        backtrace: snafu::Backtrace,
    },
    // This is used for String::from_utf8, which does zero-copy from Vec<u8>, moving towards this
    #[snafu(display("UTF8 encoding error: {}", source))]
    Utf8StringErr {
        source: std::string::FromUtf8Error,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Invalid {}: {}", field, msg))]
    ValidationErr {
        field: &'static str,
        msg: &'static str,
        backtrace: snafu::Backtrace,
    },
}

/// The return type for init, handle and query. Since the error type cannot be serialized to JSON,
/// this is only available within the contract and its unit tests.
///
/// The prefix "Std" means "the standard result within the standard library". This is not the only
/// result/error type in cosmwasm-std.
pub type StdResult<T> = core::result::Result<T, StdError>;

pub fn invalid<T>(field: &'static str, msg: &'static str) -> StdResult<T> {
    ValidationErr { field, msg }.fail()
}

pub fn contract_err<T>(msg: &'static str) -> StdResult<T> {
    ContractErr { msg }.fail()
}

pub fn dyn_contract_err<T>(msg: String) -> StdResult<T> {
    DynContractErr { msg }.fail()
}

pub fn underflow<T, U: ToString>(minuend: U, subtrahend: U) -> StdResult<T> {
    Underflow {
        minuend: minuend.to_string(),
        subtrahend: subtrahend.to_string(),
    }
    .fail()
}

pub fn unauthorized<T>() -> StdResult<T> {
    Unauthorized {}.fail()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn use_invalid() {
        let e: StdResult<()> = invalid("demo", "not implemented");
        match e {
            Err(StdError::ValidationErr { field, msg, .. }) => {
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
        let e: StdResult<()> = contract_err("not implemented");
        match e {
            Err(StdError::ContractErr { msg, .. }) => {
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
        let e: StdResult<()> = dyn_contract_err(format!("{} is too low", guess));
        match e {
            Err(StdError::DynContractErr { msg, .. }) => {
                assert_eq!(msg, String::from("7 is too low"));
            }
            Err(e) => panic!("unexpected error, {:?}", e),
            Ok(_) => panic!("dyn_contract_err must return error"),
        }
    }

    #[test]
    fn use_underflow() {
        let e: StdResult<()> = underflow(123u128, 456u128);
        match e.unwrap_err() {
            StdError::Underflow {
                minuend,
                subtrahend,
                ..
            } => {
                assert_eq!(minuend, "123");
                assert_eq!(subtrahend, "456");
            }
            _ => panic!("expect underflow error"),
        }

        let e: StdResult<()> = underflow(777i64, 1234i64);
        match e.unwrap_err() {
            StdError::Underflow {
                minuend,
                subtrahend,
                ..
            } => {
                assert_eq!(minuend, "777");
                assert_eq!(subtrahend, "1234");
            }
            _ => panic!("expect underflow error"),
        }
    }
}
