mod core_error;

pub(crate) mod backtrace;

pub type CoreResult<T, E = CoreError> = core::result::Result<T, E>;

pub use core_error::{
    CheckedFromRatioError, CheckedMultiplyFractionError, CheckedMultiplyRatioError,
    ConversionOverflowError, CoreError, DivideByZeroError, DivisionError, OverflowError,
    OverflowOperation, RoundDownOverflowError, RoundUpOverflowError,
};
