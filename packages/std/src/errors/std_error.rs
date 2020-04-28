use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

/// Structured error type for init, handle and query. This cannot be serialized to JSON, such that
/// it is only available within the contract and its unit tests.
///
/// The prefix "Std" means "the standard error within the standard library". This is not the only
/// result/error type in cosmwasm-std.
#[derive(Debug, Serialize, Deserialize, Snafu, JsonSchema)]
#[snafu(visibility = "pub(crate)")]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum StdError {
    /// Whenever there is no specific error type available
    #[snafu(display("Generic error: {}", msg))]
    GenericErr {
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

impl PartialEq for StdError {
    /// Two errors are considered equal if and only if their payloads (i.e. all fields other than backtrace) are equal.
    ///
    /// The origin of the error (expressed by its backtrace) is ignored, which allows equality checks on errors and
    /// results in tests. This is a property that might not always be desired depending on the use case and something
    /// you should be aware of.
    ///
    /// Note: We destruct the unused backtrace as _ to avoid the use of `..` which silently ignores newly added fields.
    #[allow(clippy::unneeded_field_pattern)]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                StdError::GenericErr { msg, backtrace: _ },
                StdError::GenericErr {
                    msg: msg2,
                    backtrace: _,
                },
            ) => msg == msg2,
            (
                StdError::InvalidBase64 { msg, backtrace: _ },
                StdError::InvalidBase64 {
                    msg: msg2,
                    backtrace: _,
                },
            ) => msg == msg2,
            (
                StdError::InvalidUtf8 { msg, backtrace: _ },
                StdError::InvalidUtf8 {
                    msg: msg2,
                    backtrace: _,
                },
            ) => msg == msg2,
            (
                StdError::NotFound { kind, backtrace: _ },
                StdError::NotFound {
                    kind: kind2,
                    backtrace: _,
                },
            ) => kind == kind2,
            (StdError::NullPointer { backtrace: _ }, StdError::NullPointer { backtrace: _ }) => {
                true
            }
            (
                StdError::ParseErr {
                    target,
                    msg,
                    backtrace: _,
                },
                StdError::ParseErr {
                    target: target2,
                    msg: msg2,
                    backtrace: _,
                },
            ) => target == target2 && msg == msg2,
            (
                StdError::SerializeErr {
                    source,
                    msg,
                    backtrace: _,
                },
                StdError::SerializeErr {
                    source: source2,
                    msg: msg2,
                    backtrace: _,
                },
            ) => source == source2 && msg == msg2,
            (StdError::Unauthorized { backtrace: _ }, StdError::Unauthorized { backtrace: _ }) => {
                true
            }
            (
                StdError::Underflow {
                    minuend,
                    subtrahend,
                    backtrace: _,
                },
                StdError::Underflow {
                    minuend: minued2,
                    subtrahend: subtrahend2,
                    backtrace: _,
                },
            ) => minuend == minued2 && subtrahend == subtrahend2,
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
        let error1 = StdError::InvalidBase64 {
            msg: "invalid length".to_string(),
            backtrace: None,
        };
        let error2 = StdError::InvalidBase64 {
            msg: "invalid length".to_string(),
            backtrace: None,
        };
        assert_eq!(error1, error2);
    }

    #[test]
    fn ne_works() {
        let error1 = StdError::InvalidBase64 {
            msg: "invalid length".to_string(),
            backtrace: None,
        };
        let error2 = StdError::InvalidBase64 {
            msg: "other bla".to_string(),
            backtrace: None,
        };
        assert_ne!(error1, error2);
    }

    fn assert_conversion(original: StdError) {
        let seralized = to_vec(&original).unwrap();
        let restored: StdError = from_slice(&seralized).unwrap();
        assert_eq!(restored, original);
    }

    #[test]
    fn generic_err_conversion() {
        assert_conversion(GenericErr { msg: "something" }.build());
    }

    #[test]
    fn invalid_base64_conversion() {
        assert_conversion(
            InvalidBase64 {
                msg: "invalid length".to_string(),
            }
            .build(),
        );
    }

    #[test]
    fn unauthorized_conversion() {
        assert_conversion(Unauthorized {}.build());
    }

    #[test]
    fn null_pointer_conversion() {
        assert_conversion(NullPointer {}.build());
    }

    #[test]
    fn not_found_conversion() {
        assert_conversion(NotFound { kind: "State" }.build());
    }

    #[test]
    fn parse_err_conversion() {
        let err = from_slice::<String>(b"123").unwrap_err();
        assert_conversion(err);
    }

    #[test]
    fn serialize_err_conversion() {
        assert_conversion(
            SerializeErr {
                source: "Person",
                msg: "buffer is full",
            }
            .build(),
        );
    }
}
