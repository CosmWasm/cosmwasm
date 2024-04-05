use alloc::string::{String, ToString};
use core::fmt;
use derive_more::{Display, From};

use super::{impl_from_err, BT};

use crate::errors::{RecoverPubkeyError, VerificationError};

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
#[derive(Display, Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum CoreError {
    #[display("Verification error: {source}")]
    VerificationErr {
        source: VerificationError,
        backtrace: BT,
    },
    #[display("Recover pubkey error: {source}")]
    RecoverPubkeyErr {
        source: RecoverPubkeyError,
        backtrace: BT,
    },
    /// Whenever there is no specific error type available
    #[display("Generic error: {msg}")]
    GenericErr { msg: String, backtrace: BT },
    #[display("Invalid Base64 string: {msg}")]
    InvalidBase64 { msg: String, backtrace: BT },
    #[display("Invalid data size: expected={expected} actual={actual}")]
    InvalidDataSize {
        expected: u64,
        actual: u64,
        backtrace: BT,
    },
    #[display("Invalid hex string: {msg}")]
    InvalidHex { msg: String, backtrace: BT },
    /// Whenever UTF-8 bytes cannot be decoded into a unicode string, e.g. in String::from_utf8 or str::from_utf8.
    #[display("Cannot decode UTF8 bytes into string: {msg}")]
    InvalidUtf8 { msg: String, backtrace: BT },
    #[display("{kind} not found")]
    NotFound { kind: String, backtrace: BT },
    #[display("Error parsing into type {target_type}: {msg}")]
    ParseErr {
        /// the target type that was attempted
        target_type: String,
        msg: String,
        backtrace: BT,
    },
    #[display("Error serializing type {source_type}: {msg}")]
    SerializeErr {
        /// the source type that was attempted
        source_type: String,
        msg: String,
        backtrace: BT,
    },
    #[display("Overflow: {source}")]
    Overflow {
        source: OverflowError,
        backtrace: BT,
    },
    #[display("Divide by zero: {source}")]
    DivideByZero {
        source: DivideByZeroError,
        backtrace: BT,
    },
    #[display("Conversion error: ")]
    ConversionOverflow {
        source: ConversionOverflowError,
        backtrace: BT,
    },
}

impl_from_err!(
    ConversionOverflowError,
    CoreError,
    CoreError::ConversionOverflow
);

impl CoreError {
    pub fn verification_err(source: VerificationError) -> Self {
        CoreError::VerificationErr {
            source,
            backtrace: BT::capture(),
        }
    }

    pub fn recover_pubkey_err(source: RecoverPubkeyError) -> Self {
        CoreError::RecoverPubkeyErr {
            source,
            backtrace: BT::capture(),
        }
    }

    pub fn generic_err(msg: impl Into<String>) -> Self {
        CoreError::GenericErr {
            msg: msg.into(),
            backtrace: BT::capture(),
        }
    }

    pub fn invalid_base64(msg: impl ToString) -> Self {
        CoreError::InvalidBase64 {
            msg: msg.to_string(),
            backtrace: BT::capture(),
        }
    }

    pub fn invalid_data_size(expected: usize, actual: usize) -> Self {
        CoreError::InvalidDataSize {
            // Cast is safe because usize is 32 or 64 bit large in all environments we support
            expected: expected as u64,
            actual: actual as u64,
            backtrace: BT::capture(),
        }
    }

    pub fn invalid_hex(msg: impl ToString) -> Self {
        CoreError::InvalidHex {
            msg: msg.to_string(),
            backtrace: BT::capture(),
        }
    }

    pub fn invalid_utf8(msg: impl ToString) -> Self {
        CoreError::InvalidUtf8 {
            msg: msg.to_string(),
            backtrace: BT::capture(),
        }
    }

    pub fn not_found(kind: impl Into<String>) -> Self {
        CoreError::NotFound {
            kind: kind.into(),
            backtrace: BT::capture(),
        }
    }

    pub fn parse_err(target: impl Into<String>, msg: impl ToString) -> Self {
        CoreError::ParseErr {
            target_type: target.into(),
            msg: msg.to_string(),
            backtrace: BT::capture(),
        }
    }

    pub fn serialize_err(source: impl Into<String>, msg: impl ToString) -> Self {
        CoreError::SerializeErr {
            source_type: source.into(),
            msg: msg.to_string(),
            backtrace: BT::capture(),
        }
    }

    pub fn overflow(source: OverflowError) -> Self {
        CoreError::Overflow {
            source,
            backtrace: BT::capture(),
        }
    }

    pub fn divide_by_zero(source: DivideByZeroError) -> Self {
        CoreError::DivideByZero {
            source,
            backtrace: BT::capture(),
        }
    }
}

impl PartialEq<CoreError> for CoreError {
    fn eq(&self, rhs: &CoreError) -> bool {
        match self {
            CoreError::VerificationErr {
                source,
                backtrace: _,
            } => {
                if let CoreError::VerificationErr {
                    source: rhs_source,
                    backtrace: _,
                } = rhs
                {
                    source == rhs_source
                } else {
                    false
                }
            }
            CoreError::RecoverPubkeyErr {
                source,
                backtrace: _,
            } => {
                if let CoreError::RecoverPubkeyErr {
                    source: rhs_source,
                    backtrace: _,
                } = rhs
                {
                    source == rhs_source
                } else {
                    false
                }
            }
            CoreError::GenericErr { msg, backtrace: _ } => {
                if let CoreError::GenericErr {
                    msg: rhs_msg,
                    backtrace: _,
                } = rhs
                {
                    msg == rhs_msg
                } else {
                    false
                }
            }
            CoreError::InvalidBase64 { msg, backtrace: _ } => {
                if let CoreError::InvalidBase64 {
                    msg: rhs_msg,
                    backtrace: _,
                } = rhs
                {
                    msg == rhs_msg
                } else {
                    false
                }
            }
            CoreError::InvalidDataSize {
                expected,
                actual,
                backtrace: _,
            } => {
                if let CoreError::InvalidDataSize {
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
            CoreError::InvalidHex { msg, backtrace: _ } => {
                if let CoreError::InvalidHex {
                    msg: rhs_msg,
                    backtrace: _,
                } = rhs
                {
                    msg == rhs_msg
                } else {
                    false
                }
            }
            CoreError::InvalidUtf8 { msg, backtrace: _ } => {
                if let CoreError::InvalidUtf8 {
                    msg: rhs_msg,
                    backtrace: _,
                } = rhs
                {
                    msg == rhs_msg
                } else {
                    false
                }
            }
            CoreError::NotFound { kind, backtrace: _ } => {
                if let CoreError::NotFound {
                    kind: rhs_kind,
                    backtrace: _,
                } = rhs
                {
                    kind == rhs_kind
                } else {
                    false
                }
            }
            CoreError::ParseErr {
                target_type,
                msg,
                backtrace: _,
            } => {
                if let CoreError::ParseErr {
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
            CoreError::SerializeErr {
                source_type,
                msg,
                backtrace: _,
            } => {
                if let CoreError::SerializeErr {
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
            CoreError::Overflow {
                source,
                backtrace: _,
            } => {
                if let CoreError::Overflow {
                    source: rhs_source,
                    backtrace: _,
                } = rhs
                {
                    source == rhs_source
                } else {
                    false
                }
            }
            CoreError::DivideByZero {
                source,
                backtrace: _,
            } => {
                if let CoreError::DivideByZero {
                    source: rhs_source,
                    backtrace: _,
                } = rhs
                {
                    source == rhs_source
                } else {
                    false
                }
            }
            CoreError::ConversionOverflow {
                source,
                backtrace: _,
            } => {
                if let CoreError::ConversionOverflow {
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

impl From<core::str::Utf8Error> for CoreError {
    fn from(source: core::str::Utf8Error) -> Self {
        Self::invalid_utf8(source)
    }
}

impl From<alloc::string::FromUtf8Error> for CoreError {
    fn from(source: alloc::string::FromUtf8Error) -> Self {
        Self::invalid_utf8(source)
    }
}

impl From<VerificationError> for CoreError {
    fn from(source: VerificationError) -> Self {
        Self::verification_err(source)
    }
}

impl From<RecoverPubkeyError> for CoreError {
    fn from(source: RecoverPubkeyError) -> Self {
        Self::recover_pubkey_err(source)
    }
}

impl From<OverflowError> for CoreError {
    fn from(source: OverflowError) -> Self {
        Self::overflow(source)
    }
}

impl From<DivideByZeroError> for CoreError {
    fn from(source: DivideByZeroError) -> Self {
        Self::divide_by_zero(source)
    }
}

/// The return type for init, execute and query. Since the error type cannot be serialized to JSON,
/// this is only available within the contract and its unit tests.
///
/// The prefix "Core"/"Std" means "the standard result within the core/standard library". This is not the only
/// result/error type in cosmwasm-core/cosmwasm-std.
pub type CoreResult<T> = core::result::Result<T, CoreError>;

#[derive(Debug, PartialEq, Eq)]
pub enum OverflowOperation {
    Add,
    Sub,
    Mul,
    Pow,
    Shr,
    Shl,
}

impl fmt::Display for OverflowOperation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Display, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
#[display("Cannot {operation} with given operands")]
pub struct OverflowError {
    pub operation: OverflowOperation,
}

impl OverflowError {
    pub fn new(operation: OverflowOperation) -> Self {
        Self { operation }
    }
}

/// The error returned by [`TryFrom`] conversions that overflow, for example
/// when converting from [`Uint256`] to [`Uint128`].
///
/// [`TryFrom`]: core::convert::TryFrom
/// [`Uint256`]: crate::Uint256
/// [`Uint128`]: crate::Uint128
#[derive(Display, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
#[display("Error converting {source_type} to {target_type}")]
pub struct ConversionOverflowError {
    pub source_type: &'static str,
    pub target_type: &'static str,
}

impl ConversionOverflowError {
    pub fn new(source_type: &'static str, target_type: &'static str) -> Self {
        Self {
            source_type,
            target_type,
        }
    }
}

#[derive(Display, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
#[display("Cannot divide by zero")]
pub struct DivideByZeroError;

impl DivideByZeroError {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Display, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum DivisionError {
    #[display("Divide by zero")]
    DivideByZero,

    #[display("Overflow in division")]
    Overflow,
}

#[derive(Display, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
#[cfg_attr(not(feature = "std"), derive(From))]
pub enum CheckedMultiplyFractionError {
    #[display("{_0}")]
    DivideByZero(#[from] DivideByZeroError),

    #[display("{_0}")]
    ConversionOverflow(#[from] ConversionOverflowError),

    #[display("{_0}")]
    Overflow(#[from] OverflowError),
}

#[derive(Display, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum CheckedMultiplyRatioError {
    #[display("Denominator must not be zero")]
    DivideByZero,

    #[display("Multiplication overflow")]
    Overflow,
}

#[derive(Display, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum CheckedFromRatioError {
    #[display("Denominator must not be zero")]
    DivideByZero,

    #[display("Overflow")]
    Overflow,
}

#[derive(Display, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
#[display("Round up operation failed because of overflow")]
pub struct RoundUpOverflowError;

#[derive(Display, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
#[display("Round down operation failed because of overflow")]
pub struct RoundDownOverflowError;

#[derive(Display, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum CoinsError {
    #[display("Duplicate denom")]
    DuplicateDenom,
}

impl From<CoinsError> for CoreError {
    fn from(value: CoinsError) -> Self {
        Self::generic_err(format!("Creating Coins: {value}"))
    }
}

#[derive(Display, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum CoinFromStrError {
    #[display("Missing denominator")]
    MissingDenom,
    #[display("Missing amount or non-digit characters in amount")]
    MissingAmount,
    #[display("Invalid amount: {_0}")]
    InvalidAmount(core::num::ParseIntError),
}

impl From<core::num::ParseIntError> for CoinFromStrError {
    fn from(value: core::num::ParseIntError) -> Self {
        Self::InvalidAmount(value)
    }
}

impl From<CoinFromStrError> for CoreError {
    fn from(value: CoinFromStrError) -> Self {
        Self::generic_err(format!("Parsing Coin: {value}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::str;

    // constructors

    // example of reporting contract errors with format!
    #[test]
    fn generic_err_owned() {
        let guess = 7;
        let error = CoreError::generic_err(format!("{guess} is too low"));
        match error {
            CoreError::GenericErr { msg, .. } => {
                assert_eq!(msg, String::from("7 is too low"));
            }
            e => panic!("unexpected error, {e:?}"),
        }
    }

    // example of reporting static contract errors
    #[test]
    fn generic_err_ref() {
        let error = CoreError::generic_err("not implemented");
        match error {
            CoreError::GenericErr { msg, .. } => assert_eq!(msg, "not implemented"),
            e => panic!("unexpected error, {e:?}"),
        }
    }

    #[test]
    fn invalid_base64_works_for_strings() {
        let error = CoreError::invalid_base64("my text");
        match error {
            CoreError::InvalidBase64 { msg, .. } => {
                assert_eq!(msg, "my text");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn invalid_base64_works_for_errors() {
        let original = base64::DecodeError::InvalidLength(10);
        let error = CoreError::invalid_base64(original);
        match error {
            CoreError::InvalidBase64 { msg, .. } => {
                assert_eq!(msg, "Invalid input length: 10");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn invalid_data_size_works() {
        let error = CoreError::invalid_data_size(31, 14);
        match error {
            CoreError::InvalidDataSize {
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
        let error = CoreError::invalid_hex("my text");
        match error {
            CoreError::InvalidHex { msg, .. } => {
                assert_eq!(msg, "my text");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn invalid_hex_works_for_errors() {
        let original = hex::FromHexError::OddLength;
        let error = CoreError::invalid_hex(original);
        match error {
            CoreError::InvalidHex { msg, .. } => {
                assert_eq!(msg, "Odd number of digits");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn invalid_utf8_works_for_strings() {
        let error = CoreError::invalid_utf8("my text");
        match error {
            CoreError::InvalidUtf8 { msg, .. } => {
                assert_eq!(msg, "my text");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn invalid_utf8_works_for_errors() {
        let original = String::from_utf8(vec![0x80]).unwrap_err();
        let error = CoreError::invalid_utf8(original);
        match error {
            CoreError::InvalidUtf8 { msg, .. } => {
                assert_eq!(msg, "invalid utf-8 sequence of 1 bytes from index 0");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn not_found_works() {
        let error = CoreError::not_found("gold");
        match error {
            CoreError::NotFound { kind, .. } => assert_eq!(kind, "gold"),
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn parse_err_works() {
        let error = CoreError::parse_err("Book", "Missing field: title");
        match error {
            CoreError::ParseErr {
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
        let error = CoreError::serialize_err("Book", "Content too long");
        match error {
            CoreError::SerializeErr {
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
        let error = CoreError::overflow(OverflowError::new(OverflowOperation::Sub));
        assert!(matches!(
            error,
            CoreError::Overflow {
                source: OverflowError {
                    operation: OverflowOperation::Sub
                },
                ..
            }
        ));
    }

    #[test]
    fn overflow_works_for_i64() {
        let error = CoreError::overflow(OverflowError::new(OverflowOperation::Sub));
        assert!(matches!(
            error,
            CoreError::Overflow {
                source: OverflowError {
                    operation: OverflowOperation::Sub
                },
                ..
            }
        ));
    }

    #[test]
    fn divide_by_zero_works() {
        let error = CoreError::divide_by_zero(DivideByZeroError);
        assert!(matches!(
            error,
            CoreError::DivideByZero {
                source: DivideByZeroError,
                ..
            }
        ));
    }

    #[test]
    fn implements_debug() {
        let error: CoreError = CoreError::from(OverflowError::new(OverflowOperation::Sub));
        let embedded = format!("Debug: {error:?}");
        let expected = r#"Debug: Overflow { source: OverflowError { operation: Sub }, backtrace: <disabled> }"#;
        assert_eq!(embedded, expected);
    }

    #[test]
    fn implements_display() {
        let error: CoreError = CoreError::from(OverflowError::new(OverflowOperation::Sub));
        let embedded = format!("Display: {error}");
        assert_eq!(
            embedded,
            "Display: Overflow: Cannot Sub with given operands"
        );
    }

    #[test]
    fn implements_partial_eq() {
        let u1 = CoreError::from(OverflowError::new(OverflowOperation::Sub));
        let u2 = CoreError::from(OverflowError::new(OverflowOperation::Sub));
        let s1 = CoreError::serialize_err("Book", "Content too long");
        let s2 = CoreError::serialize_err("Book", "Content too long");
        let s3 = CoreError::serialize_err("Book", "Title too long");
        assert_eq!(u1, u2);
        assert_ne!(u1, s1);
        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    #[test]
    fn from_std_str_utf8error_works() {
        let broken = Vec::from(b"Hello \xF0\x90\x80World" as &[u8]);
        let error: CoreError = str::from_utf8(&broken).unwrap_err().into();
        match error {
            CoreError::InvalidUtf8 { msg, .. } => {
                assert_eq!(msg, "invalid utf-8 sequence of 3 bytes from index 6")
            }
            err => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn from_std_string_from_utf8error_works() {
        let error: CoreError = String::from_utf8(b"Hello \xF0\x90\x80World".to_vec())
            .unwrap_err()
            .into();
        match error {
            CoreError::InvalidUtf8 { msg, .. } => {
                assert_eq!(msg, "invalid utf-8 sequence of 3 bytes from index 6")
            }
            err => panic!("Unexpected error: {err:?}"),
        }
    }
}
