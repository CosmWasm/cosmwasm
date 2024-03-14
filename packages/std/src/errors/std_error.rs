use super::{impl_from_err, BT};
use cosmwasm_core::{ConversionOverflowError, CoreError, DivideByZeroError, OverflowError};
use thiserror::Error;

use crate::errors::{RecoverPubkeyError, VerificationError};
use crate::prelude::*;

/// Structured error type for init, execute and query.
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
#[derive(Error, Debug)]
pub enum StdError {
    #[error("Core error: {source}")]
    CoreErr { source: CoreError, backtrace: BT },
    #[error("Verification error: {source}")]
    VerificationErr {
        source: VerificationError,
        backtrace: BT,
    },
    #[error("Recover pubkey error: {source}")]
    RecoverPubkeyErr {
        source: RecoverPubkeyError,
        backtrace: BT,
    },
    /// Whenever there is no specific error type available
    #[error("Generic error: {msg}")]
    GenericErr { msg: String, backtrace: BT },
    #[error("Invalid Base64 string: {msg}")]
    InvalidBase64 { msg: String, backtrace: BT },
    #[error("Invalid data size: expected={expected} actual={actual}")]
    InvalidDataSize {
        expected: u64,
        actual: u64,
        backtrace: BT,
    },
    #[error("Invalid hex string: {msg}")]
    InvalidHex { msg: String, backtrace: BT },
    /// Whenever UTF-8 bytes cannot be decoded into a unicode string, e.g. in String::from_utf8 or str::from_utf8.
    #[error("Cannot decode UTF8 bytes into string: {msg}")]
    InvalidUtf8 { msg: String, backtrace: BT },
    #[error("{kind} not found")]
    NotFound { kind: String, backtrace: BT },
    #[error("Error parsing into type {target_type}: {msg}")]
    ParseErr {
        /// the target type that was attempted
        target_type: String,
        msg: String,
        backtrace: BT,
    },
    #[error("Error serializing type {source_type}: {msg}")]
    SerializeErr {
        /// the source type that was attempted
        source_type: String,
        msg: String,
        backtrace: BT,
    },
    #[error("Overflow: {source}")]
    Overflow {
        source: OverflowError,
        backtrace: BT,
    },
    #[error("Divide by zero: {source}")]
    DivideByZero {
        source: DivideByZeroError,
        backtrace: BT,
    },
    #[error("Conversion error: ")]
    ConversionOverflow {
        source: ConversionOverflowError,
        backtrace: BT,
    },
}

impl From<CoreError> for StdError {
    fn from(value: CoreError) -> Self {
        match value {
            CoreError::ConversionOverflow { source, backtrace } => {
                Self::ConversionOverflow { source, backtrace }
            }
            CoreError::GenericErr { msg, backtrace } => Self::GenericErr { msg, backtrace },
            CoreError::InvalidBase64 { msg, backtrace } => Self::InvalidBase64 { msg, backtrace },
            CoreError::InvalidDataSize {
                expected,
                actual,
                backtrace,
            } => Self::InvalidDataSize {
                expected,
                actual,
                backtrace,
            },
            CoreError::InvalidHex { msg, backtrace } => Self::InvalidHex { msg, backtrace },
            CoreError::Overflow { source, backtrace } => Self::Overflow { source, backtrace },
            CoreError::DivideByZero { source, backtrace } => {
                Self::DivideByZero { source, backtrace }
            }
            source => Self::CoreErr {
                source,
                backtrace: BT::capture(),
            },
        }
    }
}

impl_from_err!(
    ConversionOverflowError,
    StdError,
    StdError::ConversionOverflow
);

impl StdError {
    pub fn verification_err(source: VerificationError) -> Self {
        StdError::VerificationErr {
            source,
            backtrace: BT::capture(),
        }
    }

    pub fn recover_pubkey_err(source: RecoverPubkeyError) -> Self {
        StdError::RecoverPubkeyErr {
            source,
            backtrace: BT::capture(),
        }
    }

    pub fn generic_err(msg: impl Into<String>) -> Self {
        StdError::GenericErr {
            msg: msg.into(),
            backtrace: BT::capture(),
        }
    }

    pub fn invalid_base64(msg: impl ToString) -> Self {
        StdError::InvalidBase64 {
            msg: msg.to_string(),
            backtrace: BT::capture(),
        }
    }

    pub fn invalid_data_size(expected: usize, actual: usize) -> Self {
        StdError::InvalidDataSize {
            // Cast is safe because usize is 32 or 64 bit large in all environments we support
            expected: expected as u64,
            actual: actual as u64,
            backtrace: BT::capture(),
        }
    }

    pub fn invalid_hex(msg: impl ToString) -> Self {
        StdError::InvalidHex {
            msg: msg.to_string(),
            backtrace: BT::capture(),
        }
    }

    pub fn invalid_utf8(msg: impl ToString) -> Self {
        StdError::InvalidUtf8 {
            msg: msg.to_string(),
            backtrace: BT::capture(),
        }
    }

    pub fn not_found(kind: impl Into<String>) -> Self {
        StdError::NotFound {
            kind: kind.into(),
            backtrace: BT::capture(),
        }
    }

    pub fn parse_err(target: impl Into<String>, msg: impl ToString) -> Self {
        StdError::ParseErr {
            target_type: target.into(),
            msg: msg.to_string(),
            backtrace: BT::capture(),
        }
    }

    pub fn serialize_err(source: impl Into<String>, msg: impl ToString) -> Self {
        StdError::SerializeErr {
            source_type: source.into(),
            msg: msg.to_string(),
            backtrace: BT::capture(),
        }
    }

    pub fn overflow(source: OverflowError) -> Self {
        StdError::Overflow {
            source,
            backtrace: BT::capture(),
        }
    }

    pub fn divide_by_zero(source: DivideByZeroError) -> Self {
        StdError::DivideByZero {
            source,
            backtrace: BT::capture(),
        }
    }
}

impl PartialEq<StdError> for StdError {
    fn eq(&self, rhs: &StdError) -> bool {
        match self {
            StdError::CoreErr {
                source,
                backtrace: _,
            } => {
                if let StdError::CoreErr {
                    source: rhs_source,
                    backtrace: _,
                } = rhs
                {
                    source == rhs_source
                } else {
                    false
                }
            }
            StdError::VerificationErr {
                source,
                backtrace: _,
            } => {
                if let StdError::VerificationErr {
                    source: rhs_source,
                    backtrace: _,
                } = rhs
                {
                    source == rhs_source
                } else {
                    false
                }
            }
            StdError::RecoverPubkeyErr {
                source,
                backtrace: _,
            } => {
                if let StdError::RecoverPubkeyErr {
                    source: rhs_source,
                    backtrace: _,
                } = rhs
                {
                    source == rhs_source
                } else {
                    false
                }
            }
            StdError::GenericErr { msg, backtrace: _ } => {
                if let StdError::GenericErr {
                    msg: rhs_msg,
                    backtrace: _,
                } = rhs
                {
                    msg == rhs_msg
                } else {
                    false
                }
            }
            StdError::InvalidBase64 { msg, backtrace: _ } => {
                if let StdError::InvalidBase64 {
                    msg: rhs_msg,
                    backtrace: _,
                } = rhs
                {
                    msg == rhs_msg
                } else {
                    false
                }
            }
            StdError::InvalidDataSize {
                expected,
                actual,
                backtrace: _,
            } => {
                if let StdError::InvalidDataSize {
                    expected: rhs_expected,
                    actual: rhs_actual,
                    backtrace: _,
                } = rhs
                {
                    expected == rhs_expected && actual == rhs_actual
                } else {
                    false
                }
            }
            StdError::InvalidHex { msg, backtrace: _ } => {
                if let StdError::InvalidHex {
                    msg: rhs_msg,
                    backtrace: _,
                } = rhs
                {
                    msg == rhs_msg
                } else {
                    false
                }
            }
            StdError::InvalidUtf8 { msg, backtrace: _ } => {
                if let StdError::InvalidUtf8 {
                    msg: rhs_msg,
                    backtrace: _,
                } = rhs
                {
                    msg == rhs_msg
                } else {
                    false
                }
            }
            StdError::NotFound { kind, backtrace: _ } => {
                if let StdError::NotFound {
                    kind: rhs_kind,
                    backtrace: _,
                } = rhs
                {
                    kind == rhs_kind
                } else {
                    false
                }
            }
            StdError::ParseErr {
                target_type,
                msg,
                backtrace: _,
            } => {
                if let StdError::ParseErr {
                    target_type: rhs_target_type,
                    msg: rhs_msg,
                    backtrace: _,
                } = rhs
                {
                    target_type == rhs_target_type && msg == rhs_msg
                } else {
                    false
                }
            }
            StdError::SerializeErr {
                source_type,
                msg,
                backtrace: _,
            } => {
                if let StdError::SerializeErr {
                    source_type: rhs_source_type,
                    msg: rhs_msg,
                    backtrace: _,
                } = rhs
                {
                    source_type == rhs_source_type && msg == rhs_msg
                } else {
                    false
                }
            }
            StdError::Overflow {
                source,
                backtrace: _,
            } => {
                if let StdError::Overflow {
                    source: rhs_source,
                    backtrace: _,
                } = rhs
                {
                    source == rhs_source
                } else {
                    false
                }
            }
            StdError::DivideByZero {
                source,
                backtrace: _,
            } => {
                if let StdError::DivideByZero {
                    source: rhs_source,
                    backtrace: _,
                } = rhs
                {
                    source == rhs_source
                } else {
                    false
                }
            }
            StdError::ConversionOverflow {
                source,
                backtrace: _,
            } => {
                if let StdError::ConversionOverflow {
                    source: rhs_source,
                    backtrace: _,
                } = rhs
                {
                    source == rhs_source
                } else {
                    false
                }
            }
        }
    }
}

impl From<core::str::Utf8Error> for StdError {
    fn from(source: core::str::Utf8Error) -> Self {
        Self::invalid_utf8(source)
    }
}

impl From<alloc::string::FromUtf8Error> for StdError {
    fn from(source: alloc::string::FromUtf8Error) -> Self {
        Self::invalid_utf8(source)
    }
}

impl From<VerificationError> for StdError {
    fn from(source: VerificationError) -> Self {
        Self::verification_err(source)
    }
}

impl From<RecoverPubkeyError> for StdError {
    fn from(source: RecoverPubkeyError) -> Self {
        Self::recover_pubkey_err(source)
    }
}

impl From<OverflowError> for StdError {
    fn from(source: OverflowError) -> Self {
        Self::overflow(source)
    }
}

impl From<DivideByZeroError> for StdError {
    fn from(source: DivideByZeroError) -> Self {
        Self::divide_by_zero(source)
    }
}

/// The return type for init, execute and query. Since the error type cannot be serialized to JSON,
/// this is only available within the contract and its unit tests.
///
/// The prefix "Std" means "the standard result within the standard library". This is not the only
/// result/error type in cosmwasm-std.
pub type StdResult<T> = core::result::Result<T, StdError>;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum CoinsError {
    #[error("Duplicate denom")]
    DuplicateDenom,
}

impl From<CoinsError> for StdError {
    fn from(value: CoinsError) -> Self {
        Self::generic_err(format!("Creating Coins: {value}"))
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum CoinFromStrError {
    #[error("Missing denominator")]
    MissingDenom,
    #[error("Missing amount or non-digit characters in amount")]
    MissingAmount,
    #[error("Invalid amount: {0}")]
    InvalidAmount(core::num::ParseIntError),
}

impl From<core::num::ParseIntError> for CoinFromStrError {
    fn from(value: core::num::ParseIntError) -> Self {
        Self::InvalidAmount(value)
    }
}

impl From<CoinFromStrError> for StdError {
    fn from(value: CoinFromStrError) -> Self {
        Self::generic_err(format!("Parsing Coin: {value}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::str;
    use cosmwasm_core::OverflowOperation;

    // constructors

    // example of reporting contract errors with format!
    #[test]
    fn generic_err_owned() {
        let guess = 7;
        let error = StdError::generic_err(format!("{guess} is too low"));
        match error {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, String::from("7 is too low"));
            }
            e => panic!("unexpected error, {e:?}"),
        }
    }

    // example of reporting static contract errors
    #[test]
    fn generic_err_ref() {
        let error = StdError::generic_err("not implemented");
        match error {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "not implemented"),
            e => panic!("unexpected error, {e:?}"),
        }
    }

    #[test]
    fn core_error_conversion() {
        let generic = StdError::from(CoreError::generic_err("test error"));
        let base64 = StdError::from(CoreError::invalid_base64("invalid data"));
        let data_size = StdError::from(CoreError::invalid_data_size(10, 12));
        let hex = StdError::from(CoreError::invalid_hex("invalid hex"));
        let overflow = StdError::from(CoreError::overflow(OverflowError::new(
            OverflowOperation::Pow,
        )));
        let divide = StdError::from(CoreError::divide_by_zero(DivideByZeroError::new()));

        match generic {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "test error"),
            _ => panic!("expected different error"),
        }

        match base64 {
            StdError::InvalidBase64 { msg, .. } => assert_eq!(msg, "invalid data"),
            _ => panic!("expected different error"),
        }

        match data_size {
            StdError::InvalidDataSize {
                expected, actual, ..
            } => {
                assert_eq!(expected, 10);
                assert_eq!(actual, 12);
            }
            _ => panic!("expected different error"),
        }

        match hex {
            StdError::InvalidHex { msg, .. } => assert_eq!(msg, "invalid hex"),
            _ => panic!("expected different error"),
        }

        match overflow {
            StdError::Overflow {
                source: OverflowError { operation },
                ..
            } => assert_eq!(operation, OverflowOperation::Pow),
            _ => panic!("expected different error"),
        }

        match divide {
            StdError::DivideByZero { .. } => (),
            _ => panic!("expected different error"),
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
        let original = base64::DecodeError::InvalidLength(10);
        let error = StdError::invalid_base64(original);
        match error {
            StdError::InvalidBase64 { msg, .. } => {
                assert_eq!(msg, "Invalid input length: 10");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn invalid_data_size_works() {
        let error = StdError::invalid_data_size(31, 14);
        match error {
            StdError::InvalidDataSize {
                expected, actual, ..
            } => {
                assert_eq!(expected, 31);
                assert_eq!(actual, 14);
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn invalid_hex_works_for_strings() {
        let error = StdError::invalid_hex("my text");
        match error {
            StdError::InvalidHex { msg, .. } => {
                assert_eq!(msg, "my text");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn invalid_hex_works_for_errors() {
        let original = hex::FromHexError::OddLength;
        let error = StdError::invalid_hex(original);
        match error {
            StdError::InvalidHex { msg, .. } => {
                assert_eq!(msg, "Odd number of digits");
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
            StdError::ParseErr {
                target_type, msg, ..
            } => {
                assert_eq!(target_type, "Book");
                assert_eq!(msg, "Missing field: title");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn serialize_err_works() {
        let error = StdError::serialize_err("Book", "Content too long");
        match error {
            StdError::SerializeErr {
                source_type, msg, ..
            } => {
                assert_eq!(source_type, "Book");
                assert_eq!(msg, "Content too long");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn underflow_works_for_u128() {
        let error = StdError::overflow(OverflowError::new(OverflowOperation::Sub));
        assert!(matches!(
            error,
            StdError::Overflow {
                source: OverflowError {
                    operation: OverflowOperation::Sub
                },
                ..
            }
        ));
    }

    #[test]
    fn overflow_works_for_i64() {
        let error = StdError::overflow(OverflowError::new(OverflowOperation::Sub));
        assert!(matches!(
            error,
            StdError::Overflow {
                source: OverflowError {
                    operation: OverflowOperation::Sub
                },
                ..
            }
        ));
    }

    #[test]
    fn divide_by_zero_works() {
        let error = StdError::divide_by_zero(DivideByZeroError);
        assert!(matches!(
            error,
            StdError::DivideByZero {
                source: DivideByZeroError,
                ..
            }
        ));
    }

    #[test]
    fn implements_debug() {
        let error: StdError = StdError::from(OverflowError::new(OverflowOperation::Sub));
        let embedded = format!("Debug: {error:?}");
        let expected = r#"Debug: Overflow { source: OverflowError { operation: Sub }, backtrace: <disabled> }"#;
        assert_eq!(embedded, expected);
    }

    #[test]
    fn implements_display() {
        let error: StdError = StdError::from(OverflowError::new(OverflowOperation::Sub));
        let embedded = format!("Display: {error}");
        assert_eq!(
            embedded,
            "Display: Overflow: Cannot Sub with given operands"
        );
    }

    #[test]
    fn implements_partial_eq() {
        let u1 = StdError::from(OverflowError::new(OverflowOperation::Sub));
        let u2 = StdError::from(OverflowError::new(OverflowOperation::Sub));
        let s1 = StdError::serialize_err("Book", "Content too long");
        let s2 = StdError::serialize_err("Book", "Content too long");
        let s3 = StdError::serialize_err("Book", "Title too long");
        assert_eq!(u1, u2);
        assert_ne!(u1, s1);
        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    #[test]
    fn from_std_str_utf8error_works() {
        let broken = Vec::from(b"Hello \xF0\x90\x80World" as &[u8]);
        let error: StdError = str::from_utf8(&broken).unwrap_err().into();
        match error {
            StdError::InvalidUtf8 { msg, .. } => {
                assert_eq!(msg, "invalid utf-8 sequence of 3 bytes from index 6")
            }
            err => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn from_std_string_from_utf8error_works() {
        let error: StdError = String::from_utf8(b"Hello \xF0\x90\x80World".to_vec())
            .unwrap_err()
            .into();
        match error {
            StdError::InvalidUtf8 { msg, .. } => {
                assert_eq!(msg, "invalid utf-8 sequence of 3 bytes from index 6")
            }
            err => panic!("Unexpected error: {err:?}"),
        }
    }
}
