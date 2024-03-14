use alloc::string::{String, ToString};
use derive_more::{Display, From};

use super::backtrace::{impl_from_err, BT};

#[derive(Display, Debug)]
#[non_exhaustive]
pub enum CoreError {
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

#[cfg(feature = "std")]
impl std::error::Error for CoreError {}

impl_from_err!(
    ConversionOverflowError,
    CoreError,
    CoreError::ConversionOverflow
);

impl CoreError {
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

#[derive(Display, Debug, PartialEq, Eq)]
pub enum OverflowOperation {
    Add,
    Sub,
    Mul,
    Pow,
    Shr,
    Shl,
}

#[derive(Display, Debug, PartialEq, Eq)]
#[display("Cannot {operation} with given operands")]
pub struct OverflowError {
    pub operation: OverflowOperation,
}

impl OverflowError {
    pub fn new(operation: OverflowOperation) -> Self {
        Self { operation }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for OverflowError {}

/// The error returned by [`TryFrom`] conversions that overflow, for example
/// when converting from [`Uint256`] to [`Uint128`].
///
/// [`TryFrom`]: core::convert::TryFrom
/// [`Uint256`]: crate::Uint256
/// [`Uint128`]: crate::Uint128
#[derive(Display, Debug, PartialEq, Eq)]
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

#[cfg(feature = "std")]
impl std::error::Error for ConversionOverflowError {}

#[derive(Display, Debug, Default, PartialEq, Eq)]
#[display("Cannot divide by zero")]
pub struct DivideByZeroError;

impl DivideByZeroError {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DivideByZeroError {}

#[derive(Display, Debug, PartialEq, Eq)]
pub enum DivisionError {
    #[display("Divide by zero")]
    DivideByZero,

    #[display("Overflow in division")]
    Overflow,
}

#[cfg(feature = "std")]
impl std::error::Error for DivisionError {}

#[derive(Display, Debug, From, PartialEq, Eq)]
pub enum CheckedMultiplyFractionError {
    #[display("{_0}")]
    DivideByZero(#[from] DivideByZeroError),

    #[display("{_0}")]
    ConversionOverflow(#[from] ConversionOverflowError),

    #[display("{_0}")]
    Overflow(#[from] OverflowError),
}

#[cfg(feature = "std")]
impl std::error::Error for CheckedMultiplyFractionError {}

#[derive(Display, Debug, PartialEq, Eq)]
pub enum CheckedMultiplyRatioError {
    #[display("Denominator must not be zero")]
    DivideByZero,

    #[display("Multiplication overflow")]
    Overflow,
}

#[cfg(feature = "std")]
impl std::error::Error for CheckedMultiplyRatioError {}

#[derive(Display, Debug, PartialEq, Eq)]
pub enum CheckedFromRatioError {
    #[display("Denominator must not be zero")]
    DivideByZero,

    #[display("Overflow")]
    Overflow,
}

#[cfg(feature = "std")]
impl std::error::Error for CheckedFromRatioError {}

#[derive(Display, Debug, PartialEq, Eq)]
#[display("Round up operation failed because of overflow")]
pub struct RoundUpOverflowError;

#[cfg(feature = "std")]
impl std::error::Error for RoundUpOverflowError {}

#[derive(Display, Debug, PartialEq, Eq)]
#[display("Round down operation failed because of overflow")]
pub struct RoundDownOverflowError;

#[cfg(feature = "std")]
impl std::error::Error for RoundDownOverflowError {}
