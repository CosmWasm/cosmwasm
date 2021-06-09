use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use std::convert::{TryFrom, TryInto};
use std::fmt::{self};
use std::iter::Sum;
use std::ops;

use crate::errors::{DivideByZeroError, OverflowError, OverflowOperation, StdError};

/// A thin wrapper around u64 that is using strings for JSON encoding/decoding,
/// such that the full u64 range can be used for clients that convert JSON numbers to floats,
/// like JavaScript and jq.
///
/// # Examples
///
/// Use `from` to create instances of this and `u64` to get the value out:
///
/// ```
/// # use cosmwasm_std::Uint64;
/// let a = Uint64::from(42u64);
/// assert_eq!(a.u64(), 42);
///
/// let b = Uint64::from(70u32);
/// assert_eq!(b.u64(), 70);
/// ```
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct Uint64(#[schemars(with = "String")] u64);

impl Uint64 {
    /// Creates a Uint64(value).
    ///
    /// This method is less flexible than `from` but can be called in a const context.
    pub const fn new(value: u64) -> Self {
        Uint64(value)
    }

    /// Creates a Uint64(0)
    pub const fn zero() -> Self {
        Uint64(0)
    }

    /// Returns a copy of the internal data
    pub const fn u64(&self) -> u64 {
        self.0
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    pub fn checked_add(self, other: Self) -> Result<Self, OverflowError> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or_else(|| OverflowError::new(OverflowOperation::Add, self, other))
    }

    pub fn checked_sub(self, other: Self) -> Result<Self, OverflowError> {
        self.0
            .checked_sub(other.0)
            .map(Self)
            .ok_or_else(|| OverflowError::new(OverflowOperation::Sub, self, other))
    }

    pub fn checked_mul(self, other: Self) -> Result<Self, OverflowError> {
        self.0
            .checked_mul(other.0)
            .map(Self)
            .ok_or_else(|| OverflowError::new(OverflowOperation::Mul, self, other))
    }

    pub fn checked_div(self, other: Self) -> Result<Self, DivideByZeroError> {
        self.0
            .checked_div(other.0)
            .map(Self)
            .ok_or_else(|| DivideByZeroError::new(self))
    }

    pub fn checked_div_euclid(self, other: Self) -> Result<Self, DivideByZeroError> {
        self.0
            .checked_div_euclid(other.0)
            .map(Self)
            .ok_or_else(|| DivideByZeroError::new(self))
    }

    pub fn checked_rem(self, other: Self) -> Result<Self, DivideByZeroError> {
        self.0
            .checked_rem(other.0)
            .map(Self)
            .ok_or_else(|| DivideByZeroError::new(self))
    }

    pub fn wrapping_add(self, other: Self) -> Self {
        Self(self.0.wrapping_add(other.0))
    }

    pub fn wrapping_sub(self, other: Self) -> Self {
        Self(self.0.wrapping_sub(other.0))
    }

    pub fn wrapping_mul(self, other: Self) -> Self {
        Self(self.0.wrapping_mul(other.0))
    }

    pub fn wrapping_pow(self, other: u32) -> Self {
        Self(self.0.wrapping_pow(other))
    }

    pub fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    pub fn saturating_sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    pub fn saturating_mul(self, other: Self) -> Self {
        Self(self.0.saturating_mul(other.0))
    }

    pub fn saturating_pow(self, other: u32) -> Self {
        Self(self.0.saturating_pow(other))
    }
}

// `From<u{128,64,32,16,8}>` is implemented manually instead of
// using `impl<T: Into<u64>> From<T> for Uint64` because
// of the conflict with `TryFrom<&str>` as described here
// https://stackoverflow.com/questions/63136970/how-do-i-work-around-the-upstream-crates-may-add-a-new-impl-of-trait-error

impl From<u64> for Uint64 {
    fn from(val: u64) -> Self {
        Uint64(val)
    }
}

impl From<u32> for Uint64 {
    fn from(val: u32) -> Self {
        Uint64(val.into())
    }
}

impl From<u16> for Uint64 {
    fn from(val: u16) -> Self {
        Uint64(val.into())
    }
}

impl From<u8> for Uint64 {
    fn from(val: u8) -> Self {
        Uint64(val.into())
    }
}

impl TryFrom<&str> for Uint64 {
    type Error = StdError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val.parse::<u64>() {
            Ok(u) => Ok(Uint64(u)),
            Err(e) => Err(StdError::generic_err(format!("Parsing u64: {}", e))),
        }
    }
}

impl From<Uint64> for String {
    fn from(original: Uint64) -> Self {
        original.to_string()
    }
}

impl From<Uint64> for u64 {
    fn from(original: Uint64) -> Self {
        original.0
    }
}

impl fmt::Display for Uint64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ops::Add<Uint64> for Uint64 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Uint64(self.u64().checked_add(rhs.u64()).unwrap())
    }
}

impl<'a> ops::Add<&'a Uint64> for Uint64 {
    type Output = Self;

    fn add(self, rhs: &'a Uint64) -> Self {
        Uint64(self.u64().checked_add(rhs.u64()).unwrap())
    }
}

impl ops::AddAssign<Uint64> for Uint64 {
    fn add_assign(&mut self, rhs: Uint64) {
        self.0 = self.0.checked_add(rhs.u64()).unwrap();
    }
}

impl<'a> ops::AddAssign<&'a Uint64> for Uint64 {
    fn add_assign(&mut self, rhs: &'a Uint64) {
        self.0 = self.0.checked_add(rhs.u64()).unwrap();
    }
}

impl Uint64 {
    /// Returns `self * numerator / denominator`
    pub fn multiply_ratio<A: Into<u64>, B: Into<u64>>(
        &self,
        numerator: A,
        denominator: B,
    ) -> Uint64 {
        let numerator = numerator.into();
        let denominator = denominator.into();
        if denominator == 0 {
            panic!("Denominator must not be zero");
        }

        let val: u64 = (self.full_mul(numerator) / denominator as u128)
            .try_into()
            .expect("multiplication overflow");
        Uint64::from(val)
    }

    /// Multiplies two u64 values without overflow.
    fn full_mul(self, rhs: impl Into<u64>) -> u128 {
        self.u64() as u128 * rhs.into() as u128
    }
}

impl Serialize for Uint64 {
    /// Serializes as an integer string using base 10
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Uint64 {
    /// Deserialized from an integer string using base 10
    fn deserialize<D>(deserializer: D) -> Result<Uint64, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(Uint64Visitor)
    }
}

struct Uint64Visitor;

impl<'de> de::Visitor<'de> for Uint64Visitor {
    type Value = Uint64;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded integer")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v.parse::<u64>() {
            Ok(u) => Ok(Uint64(u)),
            Err(e) => Err(E::custom(format!("invalid Uint64 '{}' - {}", v, e))),
        }
    }
}

impl Sum<Uint64> for Uint64 {
    fn sum<I: Iterator<Item = Uint64>>(iter: I) -> Self {
        iter.fold(Uint64::zero(), ops::Add::add)
    }
}

impl<'a> Sum<&'a Uint64> for Uint64 {
    fn sum<I: Iterator<Item = &'a Uint64>>(iter: I) -> Self {
        iter.fold(Uint64::zero(), ops::Add::add)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_slice, to_vec};

    #[test]
    fn uint64_convert_into() {
        let original = Uint64(12345);
        let a = u64::from(original);
        assert_eq!(a, 12345);

        let original = Uint64(12345);
        let a = String::from(original);
        assert_eq!(a, "12345");
    }

    #[test]
    fn uint64_convert_from() {
        let a = Uint64::from(5u64);
        assert_eq!(a.0, 5);

        let a = Uint64::from(5u32);
        assert_eq!(a.0, 5);

        let a = Uint64::from(5u16);
        assert_eq!(a.0, 5);

        let a = Uint64::from(5u8);
        assert_eq!(a.0, 5);

        let result = Uint64::try_from("34567");
        assert_eq!(result.unwrap().0, 34567);

        let result = Uint64::try_from("1.23");
        assert!(result.is_err());
    }

    #[test]
    fn uint64_implements_display() {
        let a = Uint64(12345);
        assert_eq!(format!("Embedded: {}", a), "Embedded: 12345");
        assert_eq!(a.to_string(), "12345");

        let a = Uint64(0);
        assert_eq!(format!("Embedded: {}", a), "Embedded: 0");
        assert_eq!(a.to_string(), "0");
    }

    #[test]
    fn uint64_is_zero_works() {
        assert_eq!(Uint64::zero().is_zero(), true);
        assert_eq!(Uint64(0).is_zero(), true);

        assert_eq!(Uint64(1).is_zero(), false);
        assert_eq!(Uint64(123).is_zero(), false);
    }

    #[test]
    fn uint64_json() {
        let orig = Uint64(1234567890987654321);
        let serialized = to_vec(&orig).unwrap();
        assert_eq!(serialized.as_slice(), b"\"1234567890987654321\"");
        let parsed: Uint64 = from_slice(&serialized).unwrap();
        assert_eq!(parsed, orig);
    }

    #[test]
    fn uint64_compare() {
        let a = Uint64(12345);
        let b = Uint64(23456);

        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, Uint64(12345));
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn uint64_math() {
        let a = Uint64(12345);
        let b = Uint64(23456);

        // test + with owned and reference right hand side
        assert_eq!(a + b, Uint64(35801));
        assert_eq!(a + &b, Uint64(35801));

        // test - with owned and reference right hand side
        assert_eq!((b.checked_sub(a)).unwrap(), Uint64(11111));

        // test += with owned and reference right hand side
        let mut c = Uint64(300000);
        c += b;
        assert_eq!(c, Uint64(323456));
        let mut d = Uint64(300000);
        d += &b;
        assert_eq!(d, Uint64(323456));

        // error result on underflow (- would produce negative result)
        let underflow_result = a.checked_sub(b);
        let OverflowError {
            operand1, operand2, ..
        } = underflow_result.unwrap_err();
        assert_eq!((operand1, operand2), (a.to_string(), b.to_string()));
    }

    #[test]
    #[should_panic]
    fn uint64_math_overflow_panics() {
        // almost_max is 2^64 - 10
        let almost_max = Uint64(18446744073709551606);
        let _ = almost_max + Uint64(12);
    }

    #[test]
    fn uint64_multiply_ratio_works() {
        let base = Uint64(500);

        // factor 1/1
        assert_eq!(base.multiply_ratio(1u64, 1u64), Uint64(500));
        assert_eq!(base.multiply_ratio(3u64, 3u64), Uint64(500));
        assert_eq!(base.multiply_ratio(654321u64, 654321u64), Uint64(500));
        // Reactivate after https://github.com/CosmWasm/cosmwasm/issues/920
        // assert_eq!(base.multiply_ratio(u64::MAX, u64::MAX), Uint64(500));

        // factor 3/2
        assert_eq!(base.multiply_ratio(3u64, 2u64), Uint64(750));
        assert_eq!(base.multiply_ratio(333333u64, 222222u64), Uint64(750));

        // factor 2/3 (integer devision always floors the result)
        assert_eq!(base.multiply_ratio(2u64, 3u64), Uint64(333));
        assert_eq!(base.multiply_ratio(222222u64, 333333u64), Uint64(333));

        // factor 5/6 (integer devision always floors the result)
        assert_eq!(base.multiply_ratio(5u64, 6u64), Uint64(416));
        assert_eq!(base.multiply_ratio(100u64, 120u64), Uint64(416));
    }

    #[test]
    fn uint64_multiply_ratio_does_not_overflow_when_result_fits() {
        // Almost max value for Uint64.
        let base = Uint64(18446744073709551606);

        assert_eq!(base.multiply_ratio(2u64, 2u64), base);
    }

    #[test]
    #[should_panic]
    fn uint64_multiply_ratio_panicks_on_overflow() {
        // Almost max value for Uint64.
        let base = Uint64(18446744073709551606);

        assert_eq!(base.multiply_ratio(2u64, 1u64), base);
    }

    #[test]
    #[should_panic(expected = "Denominator must not be zero")]
    fn uint64_multiply_ratio_panics_for_zero_denominator() {
        Uint64(500).multiply_ratio(1u64, 0u64);
    }

    #[test]
    fn sum_works() {
        let nums = vec![Uint64(17), Uint64(123), Uint64(540), Uint64(82)];
        let expected = Uint64(762);

        let sum_as_ref = nums.iter().sum();
        assert_eq!(expected, sum_as_ref);

        let sum_as_owned = nums.into_iter().sum();
        assert_eq!(expected, sum_as_owned);
    }

    #[test]
    fn uint64_methods() {
        // checked_*
        assert!(matches!(
            Uint64(u64::MAX).checked_add(Uint64(1)),
            Err(OverflowError { .. })
        ));
        assert!(matches!(
            Uint64(0).checked_sub(Uint64(1)),
            Err(OverflowError { .. })
        ));
        assert!(matches!(
            Uint64(u64::MAX).checked_mul(Uint64(2)),
            Err(OverflowError { .. })
        ));
        assert!(matches!(
            Uint64(u64::MAX).checked_div(Uint64(0)),
            Err(DivideByZeroError { .. })
        ));
        assert!(matches!(
            Uint64(u64::MAX).checked_div_euclid(Uint64(0)),
            Err(DivideByZeroError { .. })
        ));
        assert!(matches!(
            Uint64(u64::MAX).checked_rem(Uint64(0)),
            Err(DivideByZeroError { .. })
        ));

        // saturating_*
        assert_eq!(Uint64(u64::MAX).saturating_add(Uint64(1)), Uint64(u64::MAX));
        assert_eq!(Uint64(0).saturating_sub(Uint64(1)), Uint64(0));
        assert_eq!(Uint64(u64::MAX).saturating_mul(Uint64(2)), Uint64(u64::MAX));
        assert_eq!(Uint64(u64::MAX).saturating_pow(2), Uint64(u64::MAX));

        // wrapping_*
        assert_eq!(Uint64(u64::MAX).wrapping_add(Uint64(1)), Uint64(0));
        assert_eq!(Uint64(0).wrapping_sub(Uint64(1)), Uint64(u64::MAX));
        assert_eq!(
            Uint64(u64::MAX).wrapping_mul(Uint64(2)),
            Uint64(u64::MAX - 1)
        );
        assert_eq!(Uint64(u64::MAX).wrapping_pow(2), Uint64(1));
    }
}
