use alloc::string::ToString;
use core::cmp::Ordering;
use core::fmt::{self, Write};
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};
use core::str::FromStr;
use serde::{de, ser, Deserialize, Deserializer, Serialize};

use crate::errors::{
    CheckedFromRatioError, CheckedMultiplyRatioError, DivideByZeroError, ErrorKind, OverflowError,
    OverflowOperation, RoundUpOverflowError, StdError,
};
use crate::forward_ref::{forward_ref_binop, forward_ref_op_assign};
use crate::{Decimal256, SignedDecimal, SignedDecimal256, __internal::forward_ref_partial_eq};

use super::Fraction;
use super::Isqrt;
use super::{Uint128, Uint256};

/// A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
///
/// The greatest possible value that can be represented is
/// 340282366920938463463.374607431768211455
/// which is (2^128 - 1) / 10^18
#[derive(
    Copy,
    Clone,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    schemars::JsonSchema,
    cw_schema::Schemaifier,
)]
#[schemaifier(type = cw_schema::NodeType::Decimal { precision: 128, signed: false })]
pub struct Decimal(#[schemars(with = "String")] Uint128);

forward_ref_partial_eq!(Decimal, Decimal);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Decimal range exceeded")]
pub struct DecimalRangeExceeded;

impl Decimal {
    const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000_000_000_000u128); // 1*10**18
    const DECIMAL_FRACTIONAL_SQUARED: Uint128 =
        Uint128::new(1_000_000_000_000_000_000_000_000_000_000_000_000u128); // (1*10**18)**2 = 1*10**36

    /// The number of decimal places. Since decimal types are fixed-point rather than
    /// floating-point, this is a constant.
    pub const DECIMAL_PLACES: u32 = 18; // This needs to be an even number.
    /// The largest value that can be represented by this decimal type.
    pub const MAX: Self = Self(Uint128::MAX);
    /// The smallest value that can be represented by this decimal type.
    pub const MIN: Self = Self(Uint128::MIN);

    /// Creates a Decimal(value)
    /// This is equivalent to `Decimal::from_atomics(value, 18)` but usable in a const context.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use cosmwasm_std::{Uint128, Decimal};
    /// let atoms = Uint128::new(141_183_460_469_231_731_687_303_715_884_105_727_125);
    /// let value = Decimal::new(atoms);
    /// assert_eq!(value.to_string(), "141183460469231731687.303715884105727125");
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(value: Uint128) -> Self {
        Self(value)
    }

    /// Creates a Decimal(Uint128(value))
    /// This is equivalent to `Decimal::from_atomics(value, 18)` but usable in a const context.
    #[deprecated(
        since = "3.0.0",
        note = "Use Decimal::new(Uint128::new(value)) instead"
    )]
    pub const fn raw(value: u128) -> Self {
        Self(Uint128::new(value))
    }

    /// Create a 1.0 Decimal
    #[inline]
    pub const fn one() -> Self {
        Self(Self::DECIMAL_FRACTIONAL)
    }

    /// Create a 0.0 Decimal
    #[inline]
    pub const fn zero() -> Self {
        Self(Uint128::zero())
    }

    /// Convert x% into Decimal
    ///
    /// ## Examples
    ///
    /// ```
    /// # use std::str::FromStr;
    /// # use cosmwasm_std::Decimal;
    /// const HALF: Decimal = Decimal::percent(50);
    ///
    /// assert_eq!(HALF, Decimal::from_str("0.5").unwrap());
    /// ```
    pub const fn percent(x: u64) -> Self {
        // multiplication does not overflow since `u64::MAX` * 10**16 is well in u128 range
        let atomics = (x as u128) * 10_000_000_000_000_000;
        Self(Uint128::new(atomics))
    }

    /// Convert permille (x/1000) into Decimal
    ///
    /// ## Examples
    ///
    /// ```
    /// # use std::str::FromStr;
    /// # use cosmwasm_std::Decimal;
    /// const HALF: Decimal = Decimal::permille(500);
    ///
    /// assert_eq!(HALF, Decimal::from_str("0.5").unwrap());
    /// ```
    pub const fn permille(x: u64) -> Self {
        // multiplication does not overflow since `u64::MAX` * 10**15 is well in u128 range
        let atomics = (x as u128) * 1_000_000_000_000_000;
        Self(Uint128::new(atomics))
    }

    /// Convert basis points (x/10000) into Decimal
    ///
    /// ## Examples
    ///
    /// ```
    /// # use std::str::FromStr;
    /// # use cosmwasm_std::Decimal;
    /// const TWO_BPS: Decimal = Decimal::bps(2);
    /// const HALF: Decimal = Decimal::bps(5000);
    ///
    /// assert_eq!(TWO_BPS, Decimal::from_str("0.0002").unwrap());
    /// assert_eq!(HALF, Decimal::from_str("0.5").unwrap());
    /// ```
    pub const fn bps(x: u64) -> Self {
        // multiplication does not overflow since `u64::MAX` * 10**14 is well in u128 range
        let atomics = (x as u128) * 100_000_000_000_000;
        Self(Uint128::new(atomics))
    }

    /// Creates a decimal from a number of atomic units and the number
    /// of decimal places. The inputs will be converted internally to form
    /// a decimal with 18 decimal places. So the input 123 and 2 will create
    /// the decimal 1.23.
    ///
    /// Using 18 decimal places is slightly more efficient than other values
    /// as no internal conversion is necessary.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use cosmwasm_std::{Decimal, Uint128};
    /// let a = Decimal::from_atomics(Uint128::new(1234), 3).unwrap();
    /// assert_eq!(a.to_string(), "1.234");
    ///
    /// let a = Decimal::from_atomics(1234u128, 0).unwrap();
    /// assert_eq!(a.to_string(), "1234");
    ///
    /// let a = Decimal::from_atomics(1u64, 18).unwrap();
    /// assert_eq!(a.to_string(), "0.000000000000000001");
    /// ```
    pub fn from_atomics(
        atomics: impl Into<Uint128>,
        decimal_places: u32,
    ) -> Result<Self, DecimalRangeExceeded> {
        let atomics = atomics.into();
        const TEN: Uint128 = Uint128::new(10);
        Ok(match decimal_places.cmp(&Self::DECIMAL_PLACES) {
            Ordering::Less => {
                let digits = (Self::DECIMAL_PLACES) - decimal_places; // No overflow because decimal_places < DECIMAL_PLACES
                let factor = TEN.checked_pow(digits).unwrap(); // Safe because digits <= 17
                Self(
                    atomics
                        .checked_mul(factor)
                        .map_err(|_| DecimalRangeExceeded)?,
                )
            }
            Ordering::Equal => Self(atomics),
            Ordering::Greater => {
                let digits = decimal_places - (Self::DECIMAL_PLACES); // No overflow because decimal_places > DECIMAL_PLACES
                if let Ok(factor) = TEN.checked_pow(digits) {
                    Self(atomics.checked_div(factor).unwrap()) // Safe because factor cannot be zero
                } else {
                    // In this case `factor` exceeds the Uint128 range.
                    // Any Uint128 `x` divided by `factor` with `factor > Uint128::MAX` is 0.
                    // Try e.g. Python3: `(2**128-1) // 2**128`
                    Self(Uint128::zero())
                }
            }
        })
    }

    /// Returns the ratio (numerator / denominator) as a Decimal
    pub fn from_ratio(numerator: impl Into<Uint128>, denominator: impl Into<Uint128>) -> Self {
        match Decimal::checked_from_ratio(numerator, denominator) {
            Ok(value) => value,
            Err(CheckedFromRatioError::DivideByZero) => {
                panic!("Denominator must not be zero")
            }
            Err(CheckedFromRatioError::Overflow) => panic!("Multiplication overflow"),
        }
    }

    /// Returns the ratio (numerator / denominator) as a Decimal
    pub fn checked_from_ratio(
        numerator: impl Into<Uint128>,
        denominator: impl Into<Uint128>,
    ) -> Result<Self, CheckedFromRatioError> {
        let numerator: Uint128 = numerator.into();
        let denominator: Uint128 = denominator.into();
        match numerator.checked_multiply_ratio(Self::DECIMAL_FRACTIONAL, denominator) {
            Ok(ratio) => {
                // numerator * DECIMAL_FRACTIONAL / denominator
                Ok(Decimal(ratio))
            }
            Err(CheckedMultiplyRatioError::Overflow) => Err(CheckedFromRatioError::Overflow),
            Err(CheckedMultiplyRatioError::DivideByZero) => {
                Err(CheckedFromRatioError::DivideByZero)
            }
        }
    }

    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// A decimal is an integer of atomic units plus a number that specifies the
    /// position of the decimal dot. So any decimal can be expressed as two numbers.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use cosmwasm_std::{Decimal, Uint128};
    /// # use core::str::FromStr;
    /// // Value with whole and fractional part
    /// let a = Decimal::from_str("1.234").unwrap();
    /// assert_eq!(a.decimal_places(), 18);
    /// assert_eq!(a.atomics(), Uint128::new(1234000000000000000));
    ///
    /// // Smallest possible value
    /// let b = Decimal::from_str("0.000000000000000001").unwrap();
    /// assert_eq!(b.decimal_places(), 18);
    /// assert_eq!(b.atomics(), Uint128::new(1));
    /// ```
    #[must_use]
    #[inline]
    pub const fn atomics(&self) -> Uint128 {
        self.0
    }

    /// The number of decimal places. This is a constant value for now
    /// but this could potentially change as the type evolves.
    ///
    /// See also [`Decimal::atomics()`].
    #[must_use]
    #[inline]
    pub const fn decimal_places(&self) -> u32 {
        Self::DECIMAL_PLACES
    }

    /// Rounds value down after decimal places.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn floor(&self) -> Self {
        Self((self.0 / Self::DECIMAL_FRACTIONAL) * Self::DECIMAL_FRACTIONAL)
    }

    /// Rounds value up after decimal places. Panics on overflow.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn ceil(&self) -> Self {
        match self.checked_ceil() {
            Ok(value) => value,
            Err(_) => panic!("attempt to ceil with overflow"),
        }
    }

    /// Rounds value up after decimal places. Returns OverflowError on overflow.
    pub fn checked_ceil(&self) -> Result<Self, RoundUpOverflowError> {
        let floor = self.floor();
        if floor == self {
            Ok(floor)
        } else {
            floor
                .checked_add(Decimal::one())
                .map_err(|_| RoundUpOverflowError)
        }
    }

    pub fn checked_add(self, other: Self) -> Result<Self, OverflowError> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .map_err(|_| OverflowError::new(OverflowOperation::Add))
    }

    pub fn checked_sub(self, other: Self) -> Result<Self, OverflowError> {
        self.0
            .checked_sub(other.0)
            .map(Self)
            .map_err(|_| OverflowError::new(OverflowOperation::Sub))
    }

    /// Multiplies one `Decimal` by another, returning an `OverflowError` if an overflow occurred.
    pub fn checked_mul(self, other: Self) -> Result<Self, OverflowError> {
        let result_as_uint256 = self.numerator().full_mul(other.numerator())
            / Uint256::from_uint128(Self::DECIMAL_FRACTIONAL); // from_uint128 is a const method and should be "free"
        result_as_uint256
            .try_into()
            .map(Self)
            .map_err(|_| OverflowError::new(OverflowOperation::Mul))
    }

    /// Raises a value to the power of `exp`, panics if an overflow occurred.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn pow(self, exp: u32) -> Self {
        match self.checked_pow(exp) {
            Ok(value) => value,
            Err(_) => panic!("Multiplication overflow"),
        }
    }

    /// Raises a value to the power of `exp`, returning an `OverflowError` if an overflow occurred.
    pub fn checked_pow(self, exp: u32) -> Result<Self, OverflowError> {
        // This uses the exponentiation by squaring algorithm:
        // https://en.wikipedia.org/wiki/Exponentiation_by_squaring#Basic_method

        fn inner(mut x: Decimal, mut n: u32) -> Result<Decimal, OverflowError> {
            if n == 0 {
                return Ok(Decimal::one());
            }

            let mut y = Decimal::one();

            while n > 1 {
                if n % 2 == 0 {
                    x = x.checked_mul(x)?;
                    n /= 2;
                } else {
                    y = x.checked_mul(y)?;
                    x = x.checked_mul(x)?;
                    n = (n - 1) / 2;
                }
            }

            Ok(x * y)
        }

        inner(self, exp).map_err(|_| OverflowError::new(OverflowOperation::Pow))
    }

    pub fn checked_div(self, other: Self) -> Result<Self, CheckedFromRatioError> {
        Decimal::checked_from_ratio(self.numerator(), other.numerator())
    }

    pub fn checked_rem(self, other: Self) -> Result<Self, DivideByZeroError> {
        self.0
            .checked_rem(other.0)
            .map(Self)
            .map_err(|_| DivideByZeroError)
    }

    /// Returns the approximate square root as a Decimal.
    ///
    /// This should not overflow or panic.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn sqrt(&self) -> Self {
        // The max precision is `9 - log10(self.0) / 2`.
        // We can optimize the previous loop by using `ilog10` to directly calculate the precision.
        if self.0.is_zero() {
            // value is 0, so we can use any precision, let's use the max one
            return self.sqrt_with_precision(Self::DECIMAL_PLACES / 2).unwrap();
        }

        // 38 is the max number of digits for u128
        // 9 is the max precision (DECIMAL_PLACES / 2)
        let precision_guess = (38 - self.0.ilog10()) / 2;
        let precision = core::cmp::min(precision_guess, Self::DECIMAL_PLACES / 2);

        // The estimate using ilog10 might determine a precision that causes overflow for
        // high mantissas (e.g. 4e36). In that case, we need to lower the precision by 1.
        // We know that precision-1 is always safe because it reduces the exponent by 2.
        self.sqrt_with_precision(precision)
            .or_else(|| self.sqrt_with_precision(precision - 1))
            .unwrap()
    }

    /// Lower precision means more aggressive rounding, but less risk of overflow.
    /// Precision *must* be a number between 0 and 9 (inclusive).
    ///
    /// Returns `None` if the internal multiplication overflows.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn sqrt_with_precision(&self, precision: u32) -> Option<Self> {
        let inner_mul = 100u128.pow(precision);
        self.0.checked_mul(inner_mul.into()).ok().map(|inner| {
            let outer_mul = Uint128::from(10u128).pow(Self::DECIMAL_PLACES / 2 - precision);
            Decimal(inner.isqrt().checked_mul(outer_mul).unwrap())
        })
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn abs_diff(self, other: Self) -> Self {
        Self(self.0.abs_diff(other.0))
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn saturating_add(self, other: Self) -> Self {
        self.checked_add(other).unwrap_or(Self::MAX)
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn saturating_sub(self, other: Self) -> Self {
        self.checked_sub(other).unwrap_or_else(|_| Self::zero())
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn saturating_mul(self, other: Self) -> Self {
        self.checked_mul(other).unwrap_or(Self::MAX)
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn saturating_pow(self, exp: u32) -> Self {
        self.checked_pow(exp).unwrap_or(Self::MAX)
    }

    /// Converts this decimal to an unsigned integer by truncating
    /// the fractional part, e.g. 22.5 becomes 22.
    ///
    /// ## Examples
    ///
    /// ```
    /// use core::str::FromStr;
    /// use cosmwasm_std::{Decimal, Uint128};
    ///
    /// let d = Decimal::from_str("12.345").unwrap();
    /// assert_eq!(d.to_uint_floor(), Uint128::new(12));
    ///
    /// let d = Decimal::from_str("12.999").unwrap();
    /// assert_eq!(d.to_uint_floor(), Uint128::new(12));
    ///
    /// let d = Decimal::from_str("75.0").unwrap();
    /// assert_eq!(d.to_uint_floor(), Uint128::new(75));
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn to_uint_floor(self) -> Uint128 {
        self.0 / Self::DECIMAL_FRACTIONAL
    }

    /// Converts this decimal to an unsigned integer by rounding up
    /// to the next integer, e.g. 22.3 becomes 23.
    ///
    /// ## Examples
    ///
    /// ```
    /// use core::str::FromStr;
    /// use cosmwasm_std::{Decimal, Uint128};
    ///
    /// let d = Decimal::from_str("12.345").unwrap();
    /// assert_eq!(d.to_uint_ceil(), Uint128::new(13));
    ///
    /// let d = Decimal::from_str("12.999").unwrap();
    /// assert_eq!(d.to_uint_ceil(), Uint128::new(13));
    ///
    /// let d = Decimal::from_str("75.0").unwrap();
    /// assert_eq!(d.to_uint_ceil(), Uint128::new(75));
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn to_uint_ceil(self) -> Uint128 {
        // Using `q = 1 + ((x - 1) / y); // if x != 0` with unsigned integers x, y, q
        // from https://stackoverflow.com/a/2745086/2013738. We know `x + y` CAN overflow.
        let x = self.0;
        let y = Self::DECIMAL_FRACTIONAL;
        if x.is_zero() {
            Uint128::zero()
        } else {
            Uint128::one() + ((x - Uint128::one()) / y)
        }
    }
}

impl Fraction<Uint128> for Decimal {
    #[inline]
    fn numerator(&self) -> Uint128 {
        self.0
    }

    #[inline]
    fn denominator(&self) -> Uint128 {
        Self::DECIMAL_FRACTIONAL
    }

    /// Returns the multiplicative inverse `1/d` for decimal `d`.
    ///
    /// If `d` is zero, none is returned.
    fn inv(&self) -> Option<Self> {
        if self.is_zero() {
            None
        } else {
            // Let self be p/q with p = self.0 and q = DECIMAL_FRACTIONAL.
            // Now we calculate the inverse a/b = q/p such that b = DECIMAL_FRACTIONAL. Then
            // `a = DECIMAL_FRACTIONAL*DECIMAL_FRACTIONAL / self.0`.
            Some(Decimal(Self::DECIMAL_FRACTIONAL_SQUARED / self.0))
        }
    }
}

impl TryFrom<Decimal256> for Decimal {
    type Error = DecimalRangeExceeded;

    fn try_from(value: Decimal256) -> Result<Self, Self::Error> {
        value
            .atomics()
            .try_into()
            .map(Decimal)
            .map_err(|_| DecimalRangeExceeded)
    }
}

impl TryFrom<SignedDecimal> for Decimal {
    type Error = DecimalRangeExceeded;

    fn try_from(value: SignedDecimal) -> Result<Self, Self::Error> {
        value
            .atomics()
            .try_into()
            .map(Decimal)
            .map_err(|_| DecimalRangeExceeded)
    }
}

impl TryFrom<SignedDecimal256> for Decimal {
    type Error = DecimalRangeExceeded;

    fn try_from(value: SignedDecimal256) -> Result<Self, Self::Error> {
        value
            .atomics()
            .try_into()
            .map(Decimal)
            .map_err(|_| DecimalRangeExceeded)
    }
}

impl TryFrom<Uint128> for Decimal {
    type Error = DecimalRangeExceeded;

    #[inline]
    fn try_from(value: Uint128) -> Result<Self, Self::Error> {
        Self::from_atomics(value, 0)
    }
}

impl FromStr for Decimal {
    type Err = StdError;

    /// Converts the decimal string to a Decimal
    /// Possible inputs: "1.23", "1", "000012", "1.123000000"
    /// Disallowed: "", ".23"
    ///
    /// This never performs any kind of rounding.
    /// More than DECIMAL_PLACES fractional digits, even zeros, result in an error.
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parts_iter = input.split('.');

        let whole_part = parts_iter.next().unwrap(); // split always returns at least one element
        let whole = whole_part.parse::<Uint128>()?;
        let mut atomics = whole.checked_mul(Self::DECIMAL_FRACTIONAL)?;

        if let Some(fractional_part) = parts_iter.next() {
            let fractional = fractional_part.parse::<Uint128>()?;
            let exp = Self::DECIMAL_PLACES
                .checked_sub(fractional_part.len() as u32)
                .ok_or_else(|| {
                    StdError::msg(format_args!(
                        "Cannot parse more than {} fractional digits",
                        Self::DECIMAL_PLACES
                    ))
                })?;
            debug_assert!(exp <= Self::DECIMAL_PLACES);
            let fractional_factor = Uint128::from(10u128.pow(exp));
            atomics = atomics.checked_add(
                // The inner multiplication can't overflow because
                // fractional < 10^DECIMAL_PLACES && fractional_factor <= 10^DECIMAL_PLACES
                fractional.checked_mul(fractional_factor).unwrap(),
            )?;
        }

        if parts_iter.next().is_some() {
            return Err(StdError::msg("Unexpected number of dots").with_kind(ErrorKind::Parsing));
        }

        Ok(Decimal(atomics))
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let whole = (self.0) / Self::DECIMAL_FRACTIONAL;
        let fractional = self.0.checked_rem(Self::DECIMAL_FRACTIONAL).unwrap();

        if fractional.is_zero() {
            write!(f, "{whole}")
        } else {
            let fractional_string = format!(
                "{:0>padding$}",
                fractional,
                padding = Self::DECIMAL_PLACES as usize
            );
            f.write_str(&whole.to_string())?;
            f.write_char('.')?;
            f.write_str(fractional_string.trim_end_matches('0'))?;
            Ok(())
        }
    }
}

impl fmt::Debug for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Decimal({self})")
    }
}

impl Add for Decimal {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Decimal(self.0 + other.0)
    }
}
forward_ref_binop!(impl Add, add for Decimal, Decimal);

impl AddAssign for Decimal {
    fn add_assign(&mut self, rhs: Decimal) {
        *self = *self + rhs;
    }
}
forward_ref_op_assign!(impl AddAssign, add_assign for Decimal, Decimal);

impl Sub for Decimal {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Decimal(self.0 - other.0)
    }
}
forward_ref_binop!(impl Sub, sub for Decimal, Decimal);

impl SubAssign for Decimal {
    fn sub_assign(&mut self, rhs: Decimal) {
        *self = *self - rhs;
    }
}
forward_ref_op_assign!(impl SubAssign, sub_assign for Decimal, Decimal);

impl Mul for Decimal {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, other: Self) -> Self {
        // Decimals are fractions. We can multiply two decimals a and b
        // via
        //       (a.numerator() * b.numerator()) / (a.denominator() * b.denominator())
        //     = (a.numerator() * b.numerator()) / a.denominator() / b.denominator()

        let result_as_uint256 = self.numerator().full_mul(other.numerator())
            / Uint256::from_uint128(Self::DECIMAL_FRACTIONAL); // from_uint128 is a const method and should be "free"
        match result_as_uint256.try_into() {
            Ok(result) => Self(result),
            Err(_) => panic!("attempt to multiply with overflow"),
        }
    }
}
forward_ref_binop!(impl Mul, mul for Decimal, Decimal);

impl MulAssign for Decimal {
    fn mul_assign(&mut self, rhs: Decimal) {
        *self = *self * rhs;
    }
}
forward_ref_op_assign!(impl MulAssign, mul_assign for Decimal, Decimal);

impl Div for Decimal {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        match Decimal::checked_from_ratio(self.numerator(), other.numerator()) {
            Ok(ratio) => ratio,
            Err(CheckedFromRatioError::DivideByZero) => {
                panic!("Division failed - denominator must not be zero")
            }
            Err(CheckedFromRatioError::Overflow) => {
                panic!("Division failed - multiplication overflow")
            }
        }
    }
}
forward_ref_binop!(impl Div, div for Decimal, Decimal);

impl DivAssign for Decimal {
    fn div_assign(&mut self, rhs: Decimal) {
        *self = *self / rhs;
    }
}
forward_ref_op_assign!(impl DivAssign, div_assign for Decimal, Decimal);

impl Div<Uint128> for Decimal {
    type Output = Self;

    fn div(self, rhs: Uint128) -> Self::Output {
        Decimal(self.0 / rhs)
    }
}

impl DivAssign<Uint128> for Decimal {
    fn div_assign(&mut self, rhs: Uint128) {
        self.0 /= rhs;
    }
}

impl Rem for Decimal {
    type Output = Self;

    /// # Panics
    ///
    /// This operation will panic if `rhs` is zero
    #[inline]
    fn rem(self, rhs: Self) -> Self {
        Self(self.0.rem(rhs.0))
    }
}
forward_ref_binop!(impl Rem, rem for Decimal, Decimal);

impl RemAssign<Decimal> for Decimal {
    fn rem_assign(&mut self, rhs: Decimal) {
        *self = *self % rhs;
    }
}
forward_ref_op_assign!(impl RemAssign, rem_assign for Decimal, Decimal);

impl<A> core::iter::Sum<A> for Decimal
where
    Self: Add<A, Output = Self>,
{
    fn sum<I: Iterator<Item = A>>(iter: I) -> Self {
        iter.fold(Self::zero(), Add::add)
    }
}

/// Serializes as a decimal string
impl Serialize for Decimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Deserializes as a base64 string
impl<'de> Deserialize<'de> for Decimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(DecimalVisitor)
    }
}

struct DecimalVisitor;

impl de::Visitor<'_> for DecimalVisitor {
    type Value = Decimal;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("expected string-encoded decimal")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match Decimal::from_str(v) {
            Ok(d) => Ok(d),
            Err(e) => Err(E::custom(format_args!("Error parsing decimal '{v}': {e}"))),
        }
    }
}
