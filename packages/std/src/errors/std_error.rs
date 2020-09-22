#![allow(deprecated)]

use snafu::Snafu;

/// Structured error type for init, handle and query.
///
/// This can be serialized and passed over the Wasm/VM boundary, which allows us to use structured
/// error types in e.g. integration tests. In that process backtraces are stripped off.
///
/// The prefix "Std" means "the standard error within the standard library". This is not the only
/// result/error type in cosmwasm-std.
///
/// When new cases are added, they should describe the problem rather than what was attempted (e.g.
/// InvalidBase64 is preferred over Base64DecodingErr). In the long run this allows us to get rid of
/// the duplication in "StdError::FooErr".
///
/// Checklist for adding a new error:
/// - Add enum case
/// - Add creator function in std_error_helpers.rs
#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum StdError {
    /// Whenever there is no specific error type available
    #[snafu(display("Generic error: {}", msg))]
    GenericErr {
        msg: String,
        backtrace: Option<snafu::Backtrace>,
    },
    #[snafu(display("Invalid Base64 string: {}", msg))]
    InvalidBase64 {
        msg: String,
        backtrace: Option<snafu::Backtrace>,
    },
    /// Whenever UTF-8 bytes cannot be decoded into a unicode string, e.g. in String::from_utf8 or str::from_utf8.
    #[snafu(display("Cannot decode UTF8 bytes into string: {}", msg))]
    InvalidUtf8 {
        msg: String,
        backtrace: Option<snafu::Backtrace>,
    },
    #[snafu(display("{} not found", kind))]
    NotFound {
        kind: String,
        backtrace: Option<snafu::Backtrace>,
    },
    #[snafu(display("Error parsing into type {}: {}", target, msg))]
    ParseErr {
        /// the target type that was attempted
        target: String,
        msg: String,
        backtrace: Option<snafu::Backtrace>,
    },
    #[snafu(display("Error serializing type {}: {}", source, msg))]
    SerializeErr {
        /// the source type that was attempted
        #[snafu(source(false))]
        source: String,
        msg: String,
        backtrace: Option<snafu::Backtrace>,
    },
    #[snafu(display("Unauthorized"))]
    #[deprecated(
        since = "0.11.0",
        note = "All StdError cases not required by the standard library will be removed in cosmwasm-std 0.12. Please migrate to custom errors instead of using StdError."
    )]
    Unauthorized { backtrace: Option<snafu::Backtrace> },
    #[snafu(display("Cannot subtract {} from {}", subtrahend, minuend))]
    Underflow {
        minuend: String,
        subtrahend: String,
        backtrace: Option<snafu::Backtrace>,
    },
}

impl StdError {
    pub fn generic_err<S: Into<String>>(msg: S) -> Self {
        GenericErr { msg: msg.into() }.build()
    }

    pub fn invalid_base64<S: ToString>(msg: S) -> Self {
        InvalidBase64 {
            msg: msg.to_string(),
        }
        .build()
    }

    pub fn invalid_utf8<S: ToString>(msg: S) -> Self {
        InvalidUtf8 {
            msg: msg.to_string(),
        }
        .build()
    }

    pub fn not_found<S: Into<String>>(kind: S) -> Self {
        NotFound { kind: kind.into() }.build()
    }

    pub fn parse_err<T: Into<String>, M: ToString>(target: T, msg: M) -> Self {
        ParseErr {
            target: target.into(),
            msg: msg.to_string(),
        }
        .build()
    }

    pub fn serialize_err<S: Into<String>, M: ToString>(source: S, msg: M) -> Self {
        SerializeErr {
            source: source.into(),
            msg: msg.to_string(),
        }
        .build()
    }

    pub fn underflow<U: ToString>(minuend: U, subtrahend: U) -> Self {
        Underflow {
            minuend: minuend.to_string(),
            subtrahend: subtrahend.to_string(),
        }
        .build()
    }

    #[deprecated(
        since = "0.11.0",
        note = "All StdError cases not required by the standard library will be removed in cosmwasm-std 0.12. Please migrate to custom errors instead of using StdError."
    )]
    pub fn unauthorized() -> Self {
        Unauthorized {}.build()
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

    // constructors

    // example of reporting contract errors with format!
    #[test]
    fn generic_err_owned() {
        let guess = 7;
        let error = StdError::generic_err(format!("{} is too low", guess));
        match error {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, String::from("7 is too low"));
            }
            e => panic!("unexpected error, {:?}", e),
        }
    }

    // example of reporting static contract errors
    #[test]
    fn generic_err_ref() {
        let error = StdError::generic_err("not implemented");
        match error {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "not implemented"),
            e => panic!("unexpected error, {:?}", e),
        }
    }

    #[test]
    fn invalid_base64_works_for_strings() {
        let error = StdError::invalid_base64("my text");
        match error {
            StdError::InvalidBase64 { msg, .. } => {
                assert_eq!(msg, "my text");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn invalid_base64_works_for_errors() {
        let original = base64::DecodeError::InvalidLength;
        let error = StdError::invalid_base64(original);
        match error {
            StdError::InvalidBase64 { msg, .. } => {
                assert_eq!(msg, "Encoded text cannot have a 6-bit remainder.");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn invalid_utf8_works_for_strings() {
        let error = StdError::invalid_utf8("my text");
        match error {
            StdError::InvalidUtf8 { msg, .. } => {
                assert_eq!(msg, "my text");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn invalid_utf8_works_for_errors() {
        let original = String::from_utf8(vec![0x80]).unwrap_err();
        let error = StdError::invalid_utf8(original);
        match error {
            StdError::InvalidUtf8 { msg, .. } => {
                assert_eq!(msg, "invalid utf-8 sequence of 1 bytes from index 0");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn not_found_works() {
        let error = StdError::not_found("gold");
        match error {
            StdError::NotFound { kind, .. } => assert_eq!(kind, "gold"),
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn parse_err_works() {
        let error = StdError::parse_err("Book", "Missing field: title");
        match error {
            StdError::ParseErr { target, msg, .. } => {
                assert_eq!(target, "Book");
                assert_eq!(msg, "Missing field: title");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn serialize_err_works() {
        let error = StdError::serialize_err("Book", "Content too long");
        match error {
            StdError::SerializeErr { source, msg, .. } => {
                assert_eq!(source, "Book");
                assert_eq!(msg, "Content too long");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn underflow_works_for_u128() {
        let error = StdError::underflow(123u128, 456u128);
        match error {
            StdError::Underflow {
                minuend,
                subtrahend,
                ..
            } => {
                assert_eq!(minuend, "123");
                assert_eq!(subtrahend, "456");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn underflow_works_for_i64() {
        let error = StdError::underflow(777i64, 1234i64);
        match error {
            StdError::Underflow {
                minuend,
                subtrahend,
                ..
            } => {
                assert_eq!(minuend, "777");
                assert_eq!(subtrahend, "1234");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn unauthorized_works() {
        let error = StdError::unauthorized();
        match error {
            StdError::Unauthorized { .. } => {}
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn implements_debug() {
        let error: StdError = StdError::underflow(3, 5);
        let embedded = format!("Debug message: {:?}", error);
        assert_eq!(
            embedded,
            r#"Debug message: Underflow { minuend: "3", subtrahend: "5", backtrace: None }"#
        );
    }

    #[test]
    fn implements_display() {
        let error: StdError = StdError::underflow(3, 5);
        let embedded = format!("Display message: {}", error);
        assert_eq!(embedded, "Display message: Cannot subtract 5 from 3");
    }
}
