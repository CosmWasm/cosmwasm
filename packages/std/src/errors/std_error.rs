use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

/// Structured error type for init, handle and query. This cannot be serialized to JSON, such that
/// it is only available within the contract and its unit tests.
///
/// The prefix "Std" means "the standard error within the standard library". This is not the only
/// result/error type in cosmwasm-std.
#[derive(Debug, Serialize, Deserialize, Snafu, JsonSchema)]
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

    fn assert_conversion(r: StdResult<()>) {
        let error = r.unwrap_err();
        let msg = format!("{}", error);
        let converted: StdError = error.into();
        assert_eq!(msg, format!("{}", converted));
        let round_trip: StdError = from_slice(&to_vec(&converted).unwrap()).unwrap();
        assert_eq!(round_trip, converted);
    }

    #[test]
    fn dyn_contract_conversion() {
        assert_conversion(DynContractErr { msg: "dynamic" }.fail());
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
        assert_conversion(Unauthorized {}.fail());
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
