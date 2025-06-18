use alloc::string::ToString;
use core::fmt;
use std::{error::Error, str, string};

use super::BT;

use crate::errors::{RecoverPubkeyError, VerificationError};
use crate::{
    Decimal256RangeExceeded, DecimalRangeExceeded, Instantiate2AddressError,
    SignedDecimal256RangeExceeded, SignedDecimalRangeExceeded,
};

mod sealed {
    pub trait Sealed {}

    impl<T> Sealed for Result<T, super::StdError> {}
}

pub trait StdResultExt<T>: sealed::Sealed {
    fn unwrap_std_error(self) -> Result<T, Box<dyn Error + Send + Sync>>;
}

impl<T> StdResultExt<T> for Result<T, super::StdError> {
    fn unwrap_std_error(self) -> Result<T, Box<dyn Error + Send + Sync>> {
        self.map_err(|err| err.0.inner)
    }
}

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
#[derive(Debug)]
pub struct StdError(Box<InnerError>);

impl Error for StdError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.0.inner)
    }
}

#[derive(Debug)]
struct InnerError {
    backtrace: BT,
    kind: ErrorKind,
    inner: Box<dyn Error + Send + Sync>,
}

const _: () = {
    // Assert smolness (˶ᵔ ᵕ ᵔ˶)
    assert!(std::mem::size_of::<StdError>() == std::mem::size_of::<usize>());
};

impl AsRef<dyn Error + Send + Sync> for StdError {
    fn as_ref(&self) -> &(dyn Error + Send + Sync + 'static) {
        &*self.0.inner
    }
}

impl StdError {
    pub fn msg<D>(msg: D) -> Self
    where
        D: fmt::Display,
    {
        Self(Box::new(InnerError {
            backtrace: BT::capture(),
            kind: ErrorKind::Other,
            inner: msg.to_string().into(),
        }))
    }

    pub fn backtrace(&self) -> &BT {
        &self.0.backtrace
    }

    pub fn is<T>(&self) -> bool
    where
        T: Error + 'static,
    {
        self.0.inner.is::<T>()
    }

    pub fn kind(&self) -> ErrorKind {
        self.0.kind
    }

    pub fn with_kind(mut self, kind: ErrorKind) -> Self {
        self.0.kind = kind;
        self
    }
}

impl fmt::Display for StdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "kind: {:?}, error: {}", self.0.kind, self.0.inner)
    }
}

/// Implements `From` for conversions from many error types to `StdError`.
macro_rules! impl_from {
    () => {};
    ($input:ty => $kind:ident) => {
        impl From<$input> for StdError {
            fn from(value: $input) -> Self {
                Self(Box::new(InnerError {
                    backtrace: BT::capture(),
                    kind: ErrorKind::$kind,
                    inner: Box::new(value),
                }))
            }
        }
    };
    ($input:ty => $kind:ident, $($rest:tt)*) => {
        impl_from!($input => $kind);
        impl_from!($($rest)*);
    };
}

impl_from! {
    str::Utf8Error => Parsing,
    string::FromUtf8Error => Parsing,
    core::num::ParseIntError => Parsing,
    CoinFromStrError => Parsing,
    CoinsError => Other,
    ConversionOverflowError => Overflow,
    OverflowError => Overflow,
    serde_json::Error => Serialization,
    rmp_serde::encode::Error => Serialization,
    rmp_serde::decode::Error => Serialization,
    RecoverPubkeyError => Cryptography,
    VerificationError => Cryptography,
    hex::FromHexError => Encoding,
    base64::DecodeError => Encoding,
    DivideByZeroError => Other,
    CheckedFromRatioError => Other,
    CheckedMultiplyFractionError => Other,
    CheckedMultiplyRatioError => Other,
    DivisionError => Other,
    RoundUpOverflowError => Overflow,
    RoundDownOverflowError => Overflow,
    DecimalRangeExceeded => Overflow,
    Decimal256RangeExceeded => Overflow,
    SignedDecimalRangeExceeded => Overflow,
    SignedDecimal256RangeExceeded => Overflow,
    Instantiate2AddressError => Other,
}

#[cfg(not(target_arch = "wasm32"))]
impl_from! {
    bech32::EncodeError => Encoding,
    bech32::primitives::hrp::Error => Parsing,
    bech32::primitives::decode::CheckedHrpstringError => Parsing,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ErrorKind {
    Cryptography,
    Encoding,
    InvalidData,
    Overflow,
    Parsing,
    Serialization,

    Other,
}

/// The return type for init, execute and query. Since the error type cannot be serialized to JSON,
/// this is only available within the contract and its unit tests.
///
/// The prefix "Core"/"Std" means "the standard result within the core/standard library". This is not the only
/// result/error type in cosmwasm-core/cosmwasm-std.
pub type StdResult<T, E = StdError> = core::result::Result<T, E>;

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

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Cannot {operation} with given operands")]
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
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Error converting {source_type} to {target_type}")]
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

#[derive(Debug, Default, PartialEq, Eq, thiserror::Error)]
#[error("Cannot divide by zero")]
pub struct DivideByZeroError;

impl DivideByZeroError {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DivisionError {
    #[error("Divide by zero")]
    DivideByZero,

    #[error("Overflow in division")]
    Overflow,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CheckedMultiplyFractionError {
    #[error("{_0}")]
    DivideByZero(#[from] DivideByZeroError),

    #[error("{_0}")]
    ConversionOverflow(#[from] ConversionOverflowError),

    #[error("{_0}")]
    Overflow(#[from] OverflowError),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CheckedMultiplyRatioError {
    #[error("Denominator must not be zero")]
    DivideByZero,

    #[error("Multiplication overflow")]
    Overflow,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CheckedFromRatioError {
    #[error("Denominator must not be zero")]
    DivideByZero,

    #[error("Overflow")]
    Overflow,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Round up operation failed because of overflow")]
pub struct RoundUpOverflowError;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Round down operation failed because of overflow")]
pub struct RoundDownOverflowError;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CoinsError {
    #[error("Duplicate denom")]
    DuplicateDenom,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CoinFromStrError {
    #[error("Missing denominator")]
    MissingDenom,
    #[error("Missing amount or non-digit characters in amount")]
    MissingAmount,
    #[error("Invalid amount: {_0}")]
    InvalidAmount(core::num::ParseIntError),
}

impl From<core::num::ParseIntError> for CoinFromStrError {
    fn from(value: core::num::ParseIntError) -> Self {
        Self::InvalidAmount(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::str;
    use std::string;

    #[test]
    fn implements_debug() {
        let error: StdError = StdError::from(OverflowError::new(OverflowOperation::Sub));
        let embedded = format!("Debug: {error:?}");
        assert!(embedded
            .starts_with("Debug: StdError(InnerError { backtrace: <disabled>, kind: Overflow, inner: OverflowError { operation: Sub } })"), "{embedded}");
    }

    #[test]
    fn implements_display() {
        let error: StdError = StdError::from(OverflowError::new(OverflowOperation::Sub));
        let embedded = format!("Display: {error}");
        assert_eq!(
            embedded, "Display: kind: Overflow, error: Cannot Sub with given operands",
            "{embedded}"
        );
    }

    #[test]
    fn from_std_str_utf8error_works() {
        let broken = Vec::from(b"Hello \xF0\x90\x80World" as &[u8]);
        let error: StdError = str::from_utf8(&broken).unwrap_err().into();
        assert!(error.is::<str::Utf8Error>());

        assert!(error
            .to_string()
            .ends_with("invalid utf-8 sequence of 3 bytes from index 6"));
    }

    #[test]
    fn from_std_string_from_utf8error_works() {
        let error: StdError = String::from_utf8(b"Hello \xF0\x90\x80World".to_vec())
            .unwrap_err()
            .into();

        assert!(error.is::<string::FromUtf8Error>());
        assert!(error
            .to_string()
            .ends_with("invalid utf-8 sequence of 3 bytes from index 6"));
    }
}
