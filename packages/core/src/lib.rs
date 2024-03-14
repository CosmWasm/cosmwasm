//! cosmwasm-core contains components of cosmwasm-std that can be used in a no_std environment.
//! All symbols are re-exported by cosmwasm-std, such that contract developers don't need to
//! add this dependency directly. It is recommended to only use cosmwasm-std whenever possible.

#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

mod binary;
mod errors;
mod forward_ref;
mod hex_binary;
mod math;

#[doc(hidden)]
pub mod __internal;

pub use crate::binary::Binary;
pub use crate::errors::{
    CheckedFromRatioError, CheckedMultiplyFractionError, CheckedMultiplyRatioError,
    ConversionOverflowError, CoreError, CoreResult, DivideByZeroError, DivisionError,
    OverflowError, OverflowOperation, RoundDownOverflowError, RoundUpOverflowError,
};
pub use crate::hex_binary::HexBinary;
pub use crate::math::{
    Decimal, Decimal256, Decimal256RangeExceeded, DecimalRangeExceeded, Fraction, Int128, Int256,
    Int512, Int64, Isqrt, SignedDecimal, SignedDecimal256, SignedDecimal256RangeExceeded,
    SignedDecimalRangeExceeded, Uint128, Uint256, Uint512, Uint64,
};

/// Exposed for testing only
/// Both unit tests and integration tests are compiled to native code, so everything in here does not need to compile to Wasm.
#[cfg(not(target_arch = "wasm32"))]
pub mod testing;
