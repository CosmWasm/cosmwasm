use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::iter::Sum;
use std::ops::{self, Shr};

use crate::errors::{
    ConversionOverflowError, DivideByZeroError, OverflowError, OverflowOperation, StdError,
};
use crate::{Uint128, Uint256, Uint64};

/// This module is purely a workaround that lets us ignore lints for all the code
/// the `construct_uint!` macro generates.
#[allow(clippy::all)]
mod uints {
    uint::construct_uint! {
        pub struct U512(8);
    }
}

/// Used internally - we don't want to leak this type since we might change
/// the implementation in the future.
use uints::U512;

/// An implementation of u512 that is using strings for JSON encoding/decoding,
/// such that the full u512 range can be used for clients that convert JSON numbers to floats,
/// like JavaScript and jq.
///
/// # Examples
///
/// Use `from` to create instances out of primitive uint types or `new` to provide big
/// endian bytes:
///
/// ```
/// # use cosmwasm_std::Uint512;
/// let a = Uint512::from(258u128);
/// let b = Uint512::new([
///     0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
///     0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
///     0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
///     0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
///     0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
///     0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
///     0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
///     0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 2u8,
/// ]);
/// assert_eq!(a, b);
/// ```
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct Uint512(#[schemars(with = "String")] U512);

impl Uint512 {
    pub const MAX: Uint512 = Uint512(U512::MAX);

    /// Creates a Uint512(value) from a big endian representation. It's just an alias for
    /// `from_big_endian`.
    pub fn new(value: [u8; 64]) -> Self {
        Self::from_be_bytes(value)
    }

    /// Creates a Uint512(0)
    pub const fn zero() -> Self {
        Uint512(U512::zero())
    }

    pub fn from_be_bytes(value: [u8; 64]) -> Self {
        Uint512(U512::from_big_endian(&value))
    }

    pub fn from_le_bytes(value: [u8; 64]) -> Self {
        Uint512(U512::from_little_endian(&value))
    }

    /// Returns a copy of the number as big endian bytes.
    pub fn to_be_bytes(self) -> [u8; 64] {
        let mut result = [0u8; 64];
        self.0.to_big_endian(&mut result);
        result
    }

    /// Returns a copy of the number as little endian bytes.
    pub fn to_le_bytes(self) -> [u8; 64] {
        let mut result = [0u8; 64];
        self.0.to_little_endian(&mut result);
        result
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
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

    pub fn checked_rem(self, other: Self) -> Result<Self, DivideByZeroError> {
        self.0
            .checked_rem(other.0)
            .map(Self)
            .ok_or_else(|| DivideByZeroError::new(self))
    }

    pub fn checked_shr(self, other: u32) -> Result<Self, OverflowError> {
        if other >= 512 {
            return Err(OverflowError::new(OverflowOperation::Shr, self, other));
        }

        Ok(Self(self.0.shr(other)))
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
}

impl From<Uint256> for Uint512 {
    fn from(val: Uint256) -> Self {
        let bytes = [[0u8; 32], val.to_be_bytes()].concat();

        Self::from_be_bytes(bytes.try_into().unwrap())
    }
}

impl From<Uint128> for Uint512 {
    fn from(val: Uint128) -> Self {
        val.u128().into()
    }
}

impl From<Uint64> for Uint512 {
    fn from(val: Uint64) -> Self {
        val.u64().into()
    }
}

impl From<u128> for Uint512 {
    fn from(val: u128) -> Self {
        Uint512(val.into())
    }
}

impl From<u64> for Uint512 {
    fn from(val: u64) -> Self {
        Uint512(val.into())
    }
}

impl From<u32> for Uint512 {
    fn from(val: u32) -> Self {
        Uint512(val.into())
    }
}

impl From<u16> for Uint512 {
    fn from(val: u16) -> Self {
        Uint512(val.into())
    }
}

impl From<u8> for Uint512 {
    fn from(val: u8) -> Self {
        Uint512(val.into())
    }
}

impl TryFrom<Uint512> for Uint128 {
    type Error = ConversionOverflowError;

    fn try_from(value: Uint512) -> Result<Self, Self::Error> {
        Ok(Uint128::new(value.0.try_into().map_err(|_| {
            ConversionOverflowError::new("Uint512", "Uint128", value.to_string())
        })?))
    }
}

impl TryFrom<&str> for Uint512 {
    type Error = StdError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match U512::from_dec_str(val) {
            Ok(u) => Ok(Uint512(u)),
            Err(e) => Err(StdError::generic_err(format!("Parsing u512: {}", e))),
        }
    }
}

impl From<Uint512> for String {
    fn from(original: Uint512) -> Self {
        original.to_string()
    }
}

impl fmt::Display for Uint512 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ops::Add<Uint512> for Uint512 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Uint512(self.0.checked_add(rhs.0).unwrap())
    }
}

impl<'a> ops::Add<&'a Uint512> for Uint512 {
    type Output = Self;

    fn add(self, rhs: &'a Uint512) -> Self {
        Uint512(self.0.checked_add(rhs.0).unwrap())
    }
}

impl ops::Sub<Uint512> for Uint512 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Uint512(self.0.checked_sub(rhs.0).unwrap())
    }
}

impl<'a> ops::Sub<&'a Uint512> for Uint512 {
    type Output = Self;

    fn sub(self, rhs: &'a Uint512) -> Self {
        Uint512(self.0.checked_sub(rhs.0).unwrap())
    }
}

impl ops::Div<Uint512> for Uint512 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0.checked_div(rhs.0).unwrap())
    }
}

impl<'a> ops::Div<&'a Uint512> for Uint512 {
    type Output = Self;

    fn div(self, rhs: &'a Uint512) -> Self::Output {
        Self(self.0.checked_div(rhs.0).unwrap())
    }
}

impl ops::Mul<Uint512> for Uint512 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0.checked_mul(rhs.0).unwrap())
    }
}

impl<'a> ops::Mul<&'a Uint512> for Uint512 {
    type Output = Self;

    fn mul(self, rhs: &'a Uint512) -> Self::Output {
        Self(self.0.checked_mul(rhs.0).unwrap())
    }
}

impl ops::Shr<u32> for Uint512 {
    type Output = Self;

    fn shr(self, rhs: u32) -> Self::Output {
        self.checked_shr(rhs).unwrap_or_else(|_| {
            panic!(
                "right shift error: {} is larger or equal than the number of bits in Uint512",
                rhs,
            )
        })
    }
}

impl<'a> ops::Shr<&'a u32> for Uint512 {
    type Output = Self;

    fn shr(self, rhs: &'a u32) -> Self::Output {
        Shr::<u32>::shr(self, *rhs)
    }
}

impl ops::AddAssign<Uint512> for Uint512 {
    fn add_assign(&mut self, rhs: Uint512) {
        self.0 = self.0.checked_add(rhs.0).unwrap();
    }
}

impl<'a> ops::AddAssign<&'a Uint512> for Uint512 {
    fn add_assign(&mut self, rhs: &'a Uint512) {
        self.0 = self.0.checked_add(rhs.0).unwrap();
    }
}

impl ops::SubAssign<Uint512> for Uint512 {
    fn sub_assign(&mut self, rhs: Uint512) {
        self.0 = self.0.checked_sub(rhs.0).unwrap();
    }
}

impl<'a> ops::SubAssign<&'a Uint512> for Uint512 {
    fn sub_assign(&mut self, rhs: &'a Uint512) {
        self.0 = self.0.checked_sub(rhs.0).unwrap();
    }
}

impl ops::DivAssign<Uint512> for Uint512 {
    fn div_assign(&mut self, rhs: Self) {
        self.0 = self.0.checked_div(rhs.0).unwrap();
    }
}

impl<'a> ops::DivAssign<&'a Uint512> for Uint512 {
    fn div_assign(&mut self, rhs: &'a Uint512) {
        self.0 = self.0.checked_div(rhs.0).unwrap();
    }
}

impl ops::MulAssign<Uint512> for Uint512 {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 = self.0.checked_mul(rhs.0).unwrap();
    }
}

impl<'a> ops::MulAssign<&'a Uint512> for Uint512 {
    fn mul_assign(&mut self, rhs: &'a Uint512) {
        self.0 = self.0.checked_mul(rhs.0).unwrap();
    }
}

impl ops::ShrAssign<u32> for Uint512 {
    fn shr_assign(&mut self, rhs: u32) {
        *self = Shr::<u32>::shr(*self, rhs);
    }
}

impl<'a> ops::ShrAssign<&'a u32> for Uint512 {
    fn shr_assign(&mut self, rhs: &'a u32) {
        *self = Shr::<u32>::shr(*self, *rhs);
    }
}

impl Serialize for Uint512 {
    /// Serializes as an integer string using base 10
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Uint512 {
    /// Deserialized from an integer string using base 10
    fn deserialize<D>(deserializer: D) -> Result<Uint512, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(Uint512Visitor)
    }
}

struct Uint512Visitor;

impl<'de> de::Visitor<'de> for Uint512Visitor {
    type Value = Uint512;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded integer")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Uint512::try_from(v).map_err(|e| E::custom(format!("invalid Uint512 '{}' - {}", v, e)))
    }
}

impl Sum<Uint512> for Uint512 {
    fn sum<I: Iterator<Item = Uint512>>(iter: I) -> Self {
        iter.fold(Uint512::zero(), ops::Add::add)
    }
}

impl<'a> Sum<&'a Uint512> for Uint512 {
    fn sum<I: Iterator<Item = &'a Uint512>>(iter: I) -> Self {
        iter.fold(Uint512::zero(), ops::Add::add)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_slice, to_vec};

    #[test]
    fn uint512_construct() {
        let num = Uint512::new([1; 64]);
        let a: [u8; 64] = num.to_be_bytes();
        assert_eq!(a, [1; 64]);

        let be_bytes = [
            0u8, 222u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 2u8, 3u8,
        ];
        let num = Uint512::new(be_bytes);
        let resulting_bytes: [u8; 64] = num.to_be_bytes();
        assert_eq!(be_bytes, resulting_bytes);
    }

    #[test]
    fn uint512_endianness() {
        let be_bytes = [
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 2u8, 3u8,
        ];
        let le_bytes = [
            3u8, 2u8, 1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        ];

        // These should all be the same.
        let num1 = Uint512::new(be_bytes);
        let num2 = Uint512::from_be_bytes(be_bytes);
        let num3 = Uint512::from_le_bytes(le_bytes);
        assert_eq!(num1, Uint512::from(65536u32 + 512 + 3));
        assert_eq!(num1, num2);
        assert_eq!(num1, num3);
    }

    #[test]
    fn uint512_convert_from() {
        let a = Uint512::from(5u128);
        assert_eq!(a.0, U512::from(5));

        let a = Uint512::from(5u64);
        assert_eq!(a.0, U512::from(5));

        let a = Uint512::from(5u32);
        assert_eq!(a.0, U512::from(5));

        let a = Uint512::from(5u16);
        assert_eq!(a.0, U512::from(5));

        let a = Uint512::from(5u8);
        assert_eq!(a.0, U512::from(5));

        let result = Uint512::try_from("34567");
        assert_eq!(result.unwrap().0, U512::from_dec_str("34567").unwrap());

        let result = Uint512::try_from("1.23");
        assert!(result.is_err());
    }

    #[test]
    fn uint512_convert_to_uint128() {
        let source = Uint512::from(42u128);
        let target = Uint128::try_from(source);
        assert_eq!(target, Ok(Uint128::new(42u128)));

        let source = Uint512::MAX;
        let target = Uint128::try_from(source);
        assert_eq!(
            target,
            Err(ConversionOverflowError::new(
                "Uint512",
                "Uint128",
                Uint512::MAX.to_string()
            ))
        );
    }

    #[test]
    fn uint512_implements_display() {
        let a = Uint512::from(12345u32);
        assert_eq!(format!("Embedded: {}", a), "Embedded: 12345");
        assert_eq!(a.to_string(), "12345");

        let a = Uint512::zero();
        assert_eq!(format!("Embedded: {}", a), "Embedded: 0");
        assert_eq!(a.to_string(), "0");
    }

    #[test]
    fn uint512_is_zero_works() {
        assert!(Uint512::zero().is_zero());
        assert!(Uint512(U512::from(0)).is_zero());

        assert!(!Uint512::from(1u32).is_zero());
        assert!(!Uint512::from(123u32).is_zero());
    }

    #[test]
    fn uint512_json() {
        let orig = Uint512::from(1234567890987654321u128);
        let serialized = to_vec(&orig).unwrap();
        assert_eq!(serialized.as_slice(), b"\"1234567890987654321\"");
        let parsed: Uint512 = from_slice(&serialized).unwrap();
        assert_eq!(parsed, orig);
    }

    #[test]
    fn uint512_compare() {
        let a = Uint512::from(12345u32);
        let b = Uint512::from(23456u32);

        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, Uint512::from(12345u32));
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn uint512_math() {
        let a = Uint512::from(12345u32);
        let b = Uint512::from(23456u32);

        // test + with owned and reference right hand side
        assert_eq!(a + b, Uint512::from(35801u32));
        assert_eq!(a + &b, Uint512::from(35801u32));

        // test - with owned and reference right hand side
        assert_eq!(b - a, Uint512::from(11111u32));
        assert_eq!(b - &a, Uint512::from(11111u32));

        // test += with owned and reference right hand side
        let mut c = Uint512::from(300000u32);
        c += b;
        assert_eq!(c, Uint512::from(323456u32));
        let mut d = Uint512::from(300000u32);
        d += &b;
        assert_eq!(d, Uint512::from(323456u32));

        // test -= with owned and reference right hand side
        let mut c = Uint512::from(300000u32);
        c -= b;
        assert_eq!(c, Uint512::from(276544u32));
        let mut d = Uint512::from(300000u32);
        d -= &b;
        assert_eq!(d, Uint512::from(276544u32));

        // error result on underflow (- would produce negative result)
        let underflow_result = a.checked_sub(b);
        let OverflowError {
            operand1, operand2, ..
        } = underflow_result.unwrap_err();
        assert_eq!((operand1, operand2), (a.to_string(), b.to_string()));
    }

    #[test]
    #[should_panic]
    fn uint512_add_overflow_panics() {
        let max = Uint512::new([255u8; 64]);
        let _ = max + Uint512::from(12u32);
    }

    #[test]
    #[should_panic]
    fn uint512_sub_overflow_panics() {
        let _ = Uint512::from(1u32) - Uint512::from(2u32);
    }

    #[test]
    fn uint512_shr_works() {
        let original = Uint512::new([
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 2u8, 0u8, 4u8, 2u8,
        ]);

        let shifted = Uint512::new([
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 128u8, 1u8, 0u8,
        ]);

        assert_eq!(original >> 2u32, shifted);
    }

    #[test]
    #[should_panic]
    fn uint512_shr_overflow_panics() {
        let _ = Uint512::from(1u32) >> 512u32;
    }

    #[test]
    fn sum_works() {
        let nums = vec![
            Uint512::from(17u32),
            Uint512::from(123u32),
            Uint512::from(540u32),
            Uint512::from(82u32),
        ];
        let expected = Uint512::from(762u32);

        let sum_as_ref = nums.iter().sum();
        assert_eq!(expected, sum_as_ref);

        let sum_as_owned = nums.into_iter().sum();
        assert_eq!(expected, sum_as_owned);
    }

    #[test]
    fn uint512_methods() {
        // checked_*
        assert!(matches!(
            Uint512::MAX.checked_add(Uint512::from(1u32)),
            Err(OverflowError { .. })
        ));
        assert!(matches!(
            Uint512::from(0u32).checked_sub(Uint512::from(1u32)),
            Err(OverflowError { .. })
        ));
        assert!(matches!(
            Uint512::MAX.checked_mul(Uint512::from(2u32)),
            Err(OverflowError { .. })
        ));
        assert!(matches!(
            Uint512::MAX.checked_div(Uint512::from(0u32)),
            Err(DivideByZeroError { .. })
        ));
        assert!(matches!(
            Uint512::MAX.checked_rem(Uint512::from(0u32)),
            Err(DivideByZeroError { .. })
        ));

        // saturating_*
        assert_eq!(
            Uint512::MAX.saturating_add(Uint512::from(1u32)),
            Uint512::MAX
        );
        assert_eq!(
            Uint512::from(0u32).saturating_sub(Uint512::from(1u32)),
            Uint512::from(0u32)
        );
        assert_eq!(
            Uint512::MAX.saturating_mul(Uint512::from(2u32)),
            Uint512::MAX
        );
    }
}
