use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use std::convert::{TryFrom, TryInto};
use std::fmt::{self};
use std::iter::Sum;
use std::ops;

use crate::errors::{DivideByZeroError, OverflowError, OverflowOperation, StdError};
use crate::Uint256;

/// A thin wrapper around u128 that is using strings for JSON encoding/decoding,
/// such that the full u128 range can be used for clients that convert JSON numbers to floats,
/// like JavaScript and jq.
///
/// # Examples
///
/// Use `from` to create instances of this and `u128` to get the value out:
///
/// ```
/// # use cosmwasm_std::Uint128;
/// let a = Uint128::from(123u128);
/// assert_eq!(a.u128(), 123);
///
/// let b = Uint128::from(42u64);
/// assert_eq!(b.u128(), 42);
///
/// let c = Uint128::from(70u32);
/// assert_eq!(c.u128(), 70);
/// ```
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct Uint128(#[schemars(with = "String")] u128);

impl Uint128 {
    /// Creates a Uint128(value).
    ///
    /// This method is less flexible than `from` but can be called in a const context.
    pub const fn new(value: u128) -> Self {
        Uint128(value)
    }

    /// Creates a Uint128(0)
    pub const fn zero() -> Self {
        Uint128(0)
    }

    /// Returns a copy of the internal data
    pub fn u128(&self) -> u128 {
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
// using `impl<T: Into<u128>> From<T> for Uint128` because
// of the conflict with `TryFrom<&str>` as described here
// https://stackoverflow.com/questions/63136970/how-do-i-work-around-the-upstream-crates-may-add-a-new-impl-of-trait-error

impl From<u128> for Uint128 {
    fn from(val: u128) -> Self {
        Uint128(val)
    }
}

impl From<u64> for Uint128 {
    fn from(val: u64) -> Self {
        Uint128(val.into())
    }
}

impl From<u32> for Uint128 {
    fn from(val: u32) -> Self {
        Uint128(val.into())
    }
}

impl From<u16> for Uint128 {
    fn from(val: u16) -> Self {
        Uint128(val.into())
    }
}

impl From<u8> for Uint128 {
    fn from(val: u8) -> Self {
        Uint128(val.into())
    }
}

impl TryFrom<&str> for Uint128 {
    type Error = StdError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val.parse::<u128>() {
            Ok(u) => Ok(Uint128(u)),
            Err(e) => Err(StdError::generic_err(format!("Parsing u128: {}", e))),
        }
    }
}

impl From<Uint128> for String {
    fn from(original: Uint128) -> Self {
        original.to_string()
    }
}

impl From<Uint128> for u128 {
    fn from(original: Uint128) -> Self {
        original.0
    }
}

impl fmt::Display for Uint128 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ops::Add<Uint128> for Uint128 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Uint128(self.u128().checked_add(rhs.u128()).unwrap())
    }
}

impl<'a> ops::Add<&'a Uint128> for Uint128 {
    type Output = Self;

    fn add(self, rhs: &'a Uint128) -> Self {
        Uint128(self.u128().checked_add(rhs.u128()).unwrap())
    }
}

impl ops::Sub<Uint128> for Uint128 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Uint128(self.u128().checked_sub(rhs.u128()).unwrap())
    }
}

impl<'a> ops::Sub<&'a Uint128> for Uint128 {
    type Output = Self;

    fn sub(self, rhs: &'a Uint128) -> Self {
        Uint128(self.u128().checked_sub(rhs.u128()).unwrap())
    }
}

impl ops::Div<Uint128> for Uint128 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.u128().checked_div(rhs.u128()).unwrap())
    }
}

impl<'a> ops::Div<&'a Uint128> for Uint128 {
    type Output = Self;

    fn div(self, rhs: &'a Uint128) -> Self::Output {
        Self(self.u128().checked_div(rhs.u128()).unwrap())
    }
}

impl ops::Shr<u32> for Uint128 {
    type Output = Self;

    fn shr(self, rhs: u32) -> Self::Output {
        Self(self.u128().checked_shr(rhs).unwrap())
    }
}

impl<'a> ops::Shr<&'a u32> for Uint128 {
    type Output = Self;

    fn shr(self, rhs: &'a u32) -> Self::Output {
        Self(self.u128().checked_shr(*rhs).unwrap())
    }
}

impl ops::AddAssign<Uint128> for Uint128 {
    fn add_assign(&mut self, rhs: Uint128) {
        self.0 = self.0.checked_add(rhs.u128()).unwrap();
    }
}

impl<'a> ops::AddAssign<&'a Uint128> for Uint128 {
    fn add_assign(&mut self, rhs: &'a Uint128) {
        self.0 = self.0.checked_add(rhs.u128()).unwrap();
    }
}

impl ops::SubAssign<Uint128> for Uint128 {
    fn sub_assign(&mut self, rhs: Uint128) {
        self.0 = self.0.checked_sub(rhs.u128()).unwrap();
    }
}

impl<'a> ops::SubAssign<&'a Uint128> for Uint128 {
    fn sub_assign(&mut self, rhs: &'a Uint128) {
        self.0 = self.0.checked_sub(rhs.u128()).unwrap();
    }
}

impl ops::DivAssign<Uint128> for Uint128 {
    fn div_assign(&mut self, rhs: Self) {
        self.0 = self.0.checked_div(rhs.u128()).unwrap();
    }
}

impl<'a> ops::DivAssign<&'a Uint128> for Uint128 {
    fn div_assign(&mut self, rhs: &'a Uint128) {
        self.0 = self.0.checked_div(rhs.u128()).unwrap();
    }
}

impl ops::ShrAssign<u32> for Uint128 {
    fn shr_assign(&mut self, rhs: u32) {
        self.0 = self.0.checked_shr(rhs).unwrap();
    }
}

impl<'a> ops::ShrAssign<&'a u32> for Uint128 {
    fn shr_assign(&mut self, rhs: &'a u32) {
        self.0 = self.0.checked_shr(*rhs).unwrap();
    }
}

impl Uint128 {
    /// Returns `self * numerator / denominator`
    pub fn multiply_ratio<A: Into<u128>, B: Into<u128>>(
        &self,
        numerator: A,
        denominator: B,
    ) -> Uint128 {
        let numerator: u128 = numerator.into();
        let denominator: u128 = denominator.into();
        if denominator == 0 {
            panic!("Denominator must not be zero");
        }
        (self.full_mul(numerator) / Uint256::from(denominator))
            .try_into()
            .expect("multiplication overflow")
    }

    /// Multiplies two u128 values without overflow.
    pub fn full_mul(self, rhs: impl Into<u128>) -> Uint256 {
        Uint256::from(self.u128())
            .checked_mul(Uint256::from(rhs.into()))
            .unwrap()
    }
}

impl Serialize for Uint128 {
    /// Serializes as an integer string using base 10
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Uint128 {
    /// Deserialized from an integer string using base 10
    fn deserialize<D>(deserializer: D) -> Result<Uint128, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(Uint128Visitor)
    }
}

struct Uint128Visitor;

impl<'de> de::Visitor<'de> for Uint128Visitor {
    type Value = Uint128;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded integer")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v.parse::<u128>() {
            Ok(u) => Ok(Uint128(u)),
            Err(e) => Err(E::custom(format!("invalid Uint128 '{}' - {}", v, e))),
        }
    }
}

impl Sum<Uint128> for Uint128 {
    fn sum<I: Iterator<Item = Uint128>>(iter: I) -> Self {
        iter.fold(Uint128::zero(), ops::Add::add)
    }
}

impl<'a> Sum<&'a Uint128> for Uint128 {
    fn sum<I: Iterator<Item = &'a Uint128>>(iter: I) -> Self {
        iter.fold(Uint128::zero(), ops::Add::add)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_slice, to_vec};

    #[test]
    fn uint128_convert_into() {
        let original = Uint128(12345);
        let a = u128::from(original);
        assert_eq!(a, 12345);

        let original = Uint128(12345);
        let a = String::from(original);
        assert_eq!(a, "12345");
    }

    #[test]
    fn uint128_convert_from() {
        let a = Uint128::from(5u128);
        assert_eq!(a.0, 5);

        let a = Uint128::from(5u64);
        assert_eq!(a.0, 5);

        let a = Uint128::from(5u32);
        assert_eq!(a.0, 5);

        let a = Uint128::from(5u16);
        assert_eq!(a.0, 5);

        let a = Uint128::from(5u8);
        assert_eq!(a.0, 5);

        let result = Uint128::try_from("34567");
        assert_eq!(result.unwrap().0, 34567);

        let result = Uint128::try_from("1.23");
        assert!(result.is_err());
    }

    #[test]
    fn uint128_implements_display() {
        let a = Uint128(12345);
        assert_eq!(format!("Embedded: {}", a), "Embedded: 12345");
        assert_eq!(a.to_string(), "12345");

        let a = Uint128(0);
        assert_eq!(format!("Embedded: {}", a), "Embedded: 0");
        assert_eq!(a.to_string(), "0");
    }

    #[test]
    fn uint128_is_zero_works() {
        assert!(Uint128::zero().is_zero());
        assert!(Uint128(0).is_zero());

        assert!(!Uint128(1).is_zero());
        assert!(!Uint128(123).is_zero());
    }

    #[test]
    fn uint128_json() {
        let orig = Uint128(1234567890987654321);
        let serialized = to_vec(&orig).unwrap();
        assert_eq!(serialized.as_slice(), b"\"1234567890987654321\"");
        let parsed: Uint128 = from_slice(&serialized).unwrap();
        assert_eq!(parsed, orig);
    }

    #[test]
    fn uint128_compare() {
        let a = Uint128(12345);
        let b = Uint128(23456);

        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, Uint128(12345));
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn uint128_math() {
        let a = Uint128(12345);
        let b = Uint128(23456);

        // test + with owned and reference right hand side
        assert_eq!(a + b, Uint128(35801));
        assert_eq!(a + &b, Uint128(35801));

        // test - with owned and reference right hand side
        assert_eq!(b - a, Uint128(11111));
        assert_eq!(b - &a, Uint128(11111));

        // test += with owned and reference right hand side
        let mut c = Uint128(300000);
        c += b;
        assert_eq!(c, Uint128(323456));
        let mut d = Uint128(300000);
        d += &b;
        assert_eq!(d, Uint128(323456));

        // test -= with owned and reference right hand side
        let mut c = Uint128(300000);
        c -= b;
        assert_eq!(c, Uint128(276544));
        let mut d = Uint128(300000);
        d -= &b;
        assert_eq!(d, Uint128(276544));

        // error result on underflow (- would produce negative result)
        let underflow_result = a.checked_sub(b);
        let OverflowError {
            operand1, operand2, ..
        } = underflow_result.unwrap_err();
        assert_eq!((operand1, operand2), (a.to_string(), b.to_string()));
    }

    #[test]
    #[should_panic]
    fn uint128_add_overflow_panics() {
        // almost_max is 2^128 - 10
        let almost_max = Uint128(340282366920938463463374607431768211446);
        let _ = almost_max + Uint128(12);
    }

    #[test]
    #[should_panic]
    fn uint128_sub_overflow_panics() {
        let _ = Uint128(1) - Uint128(2);
    }

    #[test]
    fn uint128_multiply_ratio_works() {
        let base = Uint128(500);

        // factor 1/1
        assert_eq!(base.multiply_ratio(1u128, 1u128), Uint128(500));
        assert_eq!(base.multiply_ratio(3u128, 3u128), Uint128(500));
        assert_eq!(base.multiply_ratio(654321u128, 654321u128), Uint128(500));
        // Reactivate after https://github.com/CosmWasm/cosmwasm/issues/920
        // assert_eq!(base.multiply_ratio(u128::MAX, u128::MAX), Uint128(500));

        // factor 3/2
        assert_eq!(base.multiply_ratio(3u128, 2u128), Uint128(750));
        assert_eq!(base.multiply_ratio(333333u128, 222222u128), Uint128(750));

        // factor 2/3 (integer devision always floors the result)
        assert_eq!(base.multiply_ratio(2u128, 3u128), Uint128(333));
        assert_eq!(base.multiply_ratio(222222u128, 333333u128), Uint128(333));

        // factor 5/6 (integer devision always floors the result)
        assert_eq!(base.multiply_ratio(5u128, 6u128), Uint128(416));
        assert_eq!(base.multiply_ratio(100u128, 120u128), Uint128(416));
    }

    #[test]
    fn uint128_multiply_ratio_does_not_overflow_when_result_fits() {
        // Almost max value for Uint128.
        let base = Uint128(u128::MAX - 9);

        assert_eq!(base.multiply_ratio(2u128, 2u128), base);
    }

    #[test]
    #[should_panic]
    fn uint128_multiply_ratio_panicks_on_overflow() {
        // Almost max value for Uint128.
        let base = Uint128(u128::MAX - 9);

        assert_eq!(base.multiply_ratio(2u128, 1u128), base);
    }

    #[test]
    #[should_panic(expected = "Denominator must not be zero")]
    fn uint128_multiply_ratio_panics_for_zero_denominator() {
        Uint128(500).multiply_ratio(1u128, 0u128);
    }

    #[test]
    fn sum_works() {
        let nums = vec![Uint128(17), Uint128(123), Uint128(540), Uint128(82)];
        let expected = Uint128(762);

        let sum_as_ref = nums.iter().sum();
        assert_eq!(expected, sum_as_ref);

        let sum_as_owned = nums.into_iter().sum();
        assert_eq!(expected, sum_as_owned);
    }

    #[test]
    fn uint128_methods() {
        // checked_*
        assert!(matches!(
            Uint128(u128::MAX).checked_add(Uint128(1)),
            Err(OverflowError { .. })
        ));
        assert!(matches!(
            Uint128(0).checked_sub(Uint128(1)),
            Err(OverflowError { .. })
        ));
        assert!(matches!(
            Uint128(u128::MAX).checked_mul(Uint128(2)),
            Err(OverflowError { .. })
        ));
        assert!(matches!(
            Uint128(u128::MAX).checked_div(Uint128(0)),
            Err(DivideByZeroError { .. })
        ));
        assert!(matches!(
            Uint128(u128::MAX).checked_div_euclid(Uint128(0)),
            Err(DivideByZeroError { .. })
        ));
        assert!(matches!(
            Uint128(u128::MAX).checked_rem(Uint128(0)),
            Err(DivideByZeroError { .. })
        ));

        // saturating_*
        assert_eq!(
            Uint128(u128::MAX).saturating_add(Uint128(1)),
            Uint128(u128::MAX)
        );
        assert_eq!(Uint128(0).saturating_sub(Uint128(1)), Uint128(0));
        assert_eq!(
            Uint128(u128::MAX).saturating_mul(Uint128(2)),
            Uint128(u128::MAX)
        );
        assert_eq!(Uint128(u128::MAX).saturating_pow(2), Uint128(u128::MAX));

        // wrapping_*
        assert_eq!(Uint128(u128::MAX).wrapping_add(Uint128(1)), Uint128(0));
        assert_eq!(Uint128(0).wrapping_sub(Uint128(1)), Uint128(u128::MAX));
        assert_eq!(
            Uint128(u128::MAX).wrapping_mul(Uint128(2)),
            Uint128(u128::MAX - 1)
        );
        assert_eq!(Uint128(u128::MAX).wrapping_pow(2), Uint128(1));
    }
}
