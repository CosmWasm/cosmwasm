use serde::{Deserialize, Serialize};
use snafu::Snafu;

/// Structured error type for init, handle and query. This cannot be serialized to JSON, such that
/// it is only available within the contract and its unit tests.
///
/// The prefix "Std" means "the standard error within the standard library". This is not the only
/// result/error type in cosmwasm-std.
#[derive(Debug, Serialize, Deserialize, Snafu)]
#[snafu(visibility = "pub")]
#[serde(rename_all = "snake_case")]
pub enum StdError {
    #[snafu(display("Contract error: {}", msg))]
    DynContractErr {
        msg: String,
        #[serde(skip)]
        backtrace: Option<snafu::Backtrace>,
    },
    #[snafu(display("Invalid Base64 string: {}", msg))]
    InvalidBase64 {
        msg: String,
        #[serde(skip)]
        backtrace: Option<snafu::Backtrace>,
    },
    /// Whenever UTF-8 bytes cannot be decoded into a unicode string, e.g. in String::from_utf8 or str::from_utf8.
    #[snafu(display("Cannot decode UTF8 bytes into string: {}", msg))]
    InvalidUtf8 {
        msg: String,
        #[serde(skip)]
        backtrace: Option<snafu::Backtrace>,
    },
    #[snafu(display("{} not found", kind))]
    NotFound {
        kind: String,
        #[serde(skip)]
        backtrace: Option<snafu::Backtrace>,
    },
    #[snafu(display("Received null pointer, refuse to use"))]
    NullPointer {
        #[serde(skip)]
        backtrace: Option<snafu::Backtrace>,
    },
    #[snafu(display("Error parsing into type {}: {}", target, msg))]
    ParseErr {
        /// the target type that was attempted
        target: String,
        msg: String,
        #[serde(skip)]
        backtrace: Option<snafu::Backtrace>,
    },
    #[snafu(display("Error serializing type {}: {}", source, msg))]
    SerializeErr {
        /// the source type that was attempted
        #[snafu(source(false))]
        source: String,
        msg: String,
        #[serde(skip)]
        backtrace: Option<snafu::Backtrace>,
    },
    #[snafu(display("Unauthorized"))]
    Unauthorized {
        #[serde(skip)]
        backtrace: Option<snafu::Backtrace>,
    },
    #[snafu(display("Cannot subtract {} from {}", subtrahend, minuend))]
    Underflow {
        minuend: String,
        subtrahend: String,
        #[serde(skip)]
        backtrace: Option<snafu::Backtrace>,
    },
}

/// No equeality is defined on backtraces. For our purposes
/// in this file we say that for two erros to be equal, their backtraces
/// must both be unset.
/// This works because we don't need a reflexive property for StdError,
/// i.e. error `x.eq(x) == true`.
fn backtraces_eq(a: &Option<snafu::Backtrace>, b: &Option<snafu::Backtrace>) -> bool {
    a.is_none() && b.is_none()
}

impl PartialEq for StdError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                StdError::DynContractErr { msg, backtrace },
                StdError::DynContractErr {
                    msg: msg2,
                    backtrace: backtrace2,
                },
            ) => msg == msg2 && backtraces_eq(backtrace, backtrace2),
            (
                StdError::InvalidBase64 { msg, backtrace },
                StdError::InvalidBase64 {
                    msg: msg2,
                    backtrace: backtrace2,
                },
            ) => msg == msg2 && backtraces_eq(backtrace, backtrace2),
            (
                StdError::InvalidUtf8 { msg, backtrace },
                StdError::InvalidUtf8 {
                    msg: msg2,
                    backtrace: backtrace2,
                },
            ) => msg == msg2 && backtraces_eq(backtrace, backtrace2),
            (
                StdError::NotFound { kind, backtrace },
                StdError::NotFound {
                    kind: kind2,
                    backtrace: backtrace2,
                },
            ) => kind == kind2 && backtraces_eq(backtrace, backtrace2),
            (
                StdError::NullPointer { backtrace },
                StdError::NullPointer {
                    backtrace: backtrace2,
                },
            ) => backtraces_eq(backtrace, backtrace2),
            (
                StdError::ParseErr {
                    target,
                    msg,
                    backtrace,
                },
                StdError::ParseErr {
                    target: target2,
                    msg: msg2,
                    backtrace: backtrace2,
                },
            ) => target == target2 && msg == msg2 && backtraces_eq(backtrace, backtrace2),
            (
                StdError::SerializeErr {
                    source,
                    msg,
                    backtrace,
                },
                StdError::SerializeErr {
                    source: source2,
                    msg: msg2,
                    backtrace: backtrace2,
                },
            ) => source == source2 && msg == msg2 && backtraces_eq(backtrace, backtrace2),
            (
                StdError::Unauthorized { backtrace },
                StdError::Unauthorized {
                    backtrace: backtrace2,
                },
            ) => backtraces_eq(backtrace, backtrace2),
            (
                StdError::Underflow {
                    minuend,
                    subtrahend,
                    backtrace,
                },
                StdError::Underflow {
                    minuend: minued2,
                    subtrahend: subtrahend2,
                    backtrace: backtrace2,
                },
            ) => {
                minuend == minued2
                    && subtrahend == subtrahend2
                    && backtraces_eq(backtrace, backtrace2)
            }
            _ => false,
        }
    }
}

/// The return type for init, handle and query. Since the error type cannot be serialized to JSON,
/// this is only available within the contract and its unit tests.
///
/// The prefix "Std" means "the standard result within the standard library". This is not the only
/// result/error type in cosmwasm-std.
pub type StdResult<T> = core::result::Result<T, StdError>;

pub fn dyn_contract_err<T, S: Into<String>>(msg: S) -> StdResult<T> {
    DynContractErr { msg: msg.into() }.fail()
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
    use crate::serde::{from_slice, to_vec};

    #[test]
    fn can_serialize() {
        let error = InvalidBase64 {
            msg: "invalid length".to_string(),
        }
        .build();
        assert_eq!(
            to_vec(&error).unwrap(),
            br#"{"invalid_base64":{"msg":"invalid length"}}"#.to_vec()
        );
    }

    #[test]
    fn can_deserialize() {
        let error: StdError =
            from_slice(br#"{"invalid_base64":{"msg":"invalid length"}}"#).unwrap();
        match error {
            StdError::InvalidBase64 { msg, backtrace } => {
                assert_eq!(msg, "invalid length");
                assert!(backtrace.is_none());
            }
            _ => panic!("invalid type"),
        };
    }

    /// The deseralizer in from_slice can perform zero-copy deserializations (https://serde.rs/lifetimes.html).
    /// So it is possible to have `&'static str` fields as long as all source data is always static.
    /// This is an unrealistic assumption for our use case. This test case ensures we can deseralize
    /// errors from limited liefetime sources.
    #[test]
    fn can_deserialize_from_non_static_source() {
        let source = (br#"{"not_found":{"kind":"bugs"}}"#).to_vec();
        let error: StdError = from_slice(&source).unwrap();
        match error {
            StdError::NotFound { kind, backtrace } => {
                assert_eq!(kind, "bugs");
                assert!(backtrace.is_none());
            }
            _ => panic!("invalid type"),
        };
    }

    #[test]
    fn eq_works() {
        let error1 = InvalidBase64 {
            msg: "invalid length".to_string(),
        }
        .build();
        let error2 = InvalidBase64 {
            msg: "other bla".to_string(),
        }
        .build();

        // Errors are only equal when bactrace is removed
        let normalized1: StdError = from_slice(&to_vec(&error1).unwrap()).unwrap();
        let normalized2: StdError = from_slice(&to_vec(&error2).unwrap()).unwrap();
        assert_ne!(normalized1, normalized2);
    }

    #[test]
    fn ne_works() {
        let error1 = InvalidBase64 {
            msg: "invalid length".to_string(),
        }
        .build();
        let error2 = InvalidBase64 {
            msg: "other bla".to_string(),
        }
        .build();
        assert_ne!(error1, error2);
    }

    // example of reporting contract errors with format!
    #[test]
    fn dyn_contract_err_owned() {
        let guess = 7;
        let res: StdResult<()> = dyn_contract_err(format!("{} is too low", guess));
        match res.unwrap_err() {
            StdError::DynContractErr { msg, .. } => {
                assert_eq!(msg, String::from("7 is too low"));
            }
            e => panic!("unexpected error, {:?}", e),
        }
    }

    // example of reporting static contract errors
    #[test]
    fn dyn_contract_err_ref() {
        let res: StdResult<()> = dyn_contract_err("not implemented");
        match res.unwrap_err() {
            StdError::DynContractErr { msg, .. } => assert_eq!(msg, "not implemented"),
            e => panic!("unexpected error, {:?}", e),
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
