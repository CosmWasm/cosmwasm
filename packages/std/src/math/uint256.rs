use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::iter::Sum;
use std::ops::{self, Shl, Shr};
use std::str::FromStr;

use crate::errors::{
    ConversionOverflowError, DivideByZeroError, OverflowError, OverflowOperation, StdError,
};
use crate::{Uint128, Uint512, Uint64};

/// This module is purely a workaround that lets us ignore lints for all the code
/// the `construct_uint!` macro generates.
#[allow(clippy::all)]
mod uints {
    uint::construct_uint! {
        pub struct U256(4);
    }
}

/// Used internally - we don't want to leak this type since we might change
/// the implementation in the future.
use uints::U256;

/// An implementation of u256 that is using strings for JSON encoding/decoding,
/// such that the full u256 range can be used for clients that convert JSON numbers to floats,
/// like JavaScript and jq.
///
/// # Examples
///
/// Use `from` to create instances out of primitive uint types or `new` to provide big
/// endian bytes:
///
/// ```
/// # use cosmwasm_std::Uint256;
/// let a = Uint256::from(258u128);
/// let b = Uint256::new([
///     0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
///     0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
///     0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
///     0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 2u8,
/// ]);
/// assert_eq!(a, b);
/// ```
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct Uint256(#[schemars(with = "String")] U256);

impl Uint256 {
    pub const MAX: Uint256 = Uint256(U256::MAX);

    /// Creates a Uint256(value) from a big endian representation. It's just an alias for
    /// [`Uint256::from_be_bytes`].
    ///
    /// This method is less flexible than `from` but can be called in a const context.
    pub const fn new(value: [u8; 32]) -> Self {
        Self::from_be_bytes(value)
    }

    /// Creates a Uint256(0)
    pub const fn zero() -> Self {
        Uint256(U256::zero())
    }

    pub const fn from_be_bytes(data: [u8; 32]) -> Self {
        let words: [u64; 4] = [
            u64::from_le_bytes([
                data[31], data[30], data[29], data[28], data[27], data[26], data[25], data[24],
            ]),
            u64::from_le_bytes([
                data[23], data[22], data[21], data[20], data[19], data[18], data[17], data[16],
            ]),
            u64::from_le_bytes([
                data[15], data[14], data[13], data[12], data[11], data[10], data[9], data[8],
            ]),
            u64::from_le_bytes([
                data[7], data[6], data[5], data[4], data[3], data[2], data[1], data[0],
            ]),
        ];
        Self(U256(words))
    }

    pub const fn from_le_bytes(data: [u8; 32]) -> Self {
        let words: [u64; 4] = [
            u64::from_le_bytes([
                data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
            ]),
            u64::from_le_bytes([
                data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15],
            ]),
            u64::from_le_bytes([
                data[16], data[17], data[18], data[19], data[20], data[21], data[22], data[23],
            ]),
            u64::from_le_bytes([
                data[24], data[25], data[26], data[27], data[28], data[29], data[30], data[31],
            ]),
        ];
        Uint256(U256(words))
    }

    /// Returns a copy of the number as big endian bytes.
    pub fn to_be_bytes(self) -> [u8; 32] {
        let mut result = [0u8; 32];
        self.0.to_big_endian(&mut result);
        result
    }

    /// Returns a copy of the number as little endian bytes.
    pub fn to_le_bytes(self) -> [u8; 32] {
        let mut result = [0u8; 32];
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

    pub fn checked_pow(self, exp: u32) -> Result<Self, OverflowError> {
        self.0
            .checked_pow(exp.into())
            .map(Self)
            .ok_or_else(|| OverflowError::new(OverflowOperation::Pow, self, exp))
    }

    pub fn pow(self, exp: u32) -> Self {
        self.checked_pow(exp)
            .expect("attempt to raise to a power with overflow")
    }

    pub fn checked_shr(self, other: u32) -> Result<Self, OverflowError> {
        if other >= 256 {
            return Err(OverflowError::new(OverflowOperation::Shr, self, other));
        }

        Ok(Self(self.0.shr(other)))
    }

    pub fn checked_shl(self, other: u32) -> Result<Self, OverflowError> {
        if other >= 256 {
            return Err(OverflowError::new(OverflowOperation::Shl, self, other));
        }

        Ok(Self(self.0.shl(other)))
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

impl From<Uint128> for Uint256 {
    fn from(val: Uint128) -> Self {
        val.u128().into()
    }
}

impl From<Uint64> for Uint256 {
    fn from(val: Uint64) -> Self {
        val.u64().into()
    }
}

impl From<u128> for Uint256 {
    fn from(val: u128) -> Self {
        Uint256(val.into())
    }
}

impl From<u64> for Uint256 {
    fn from(val: u64) -> Self {
        Uint256(val.into())
    }
}

impl From<u32> for Uint256 {
    fn from(val: u32) -> Self {
        Uint256(val.into())
    }
}

impl From<u16> for Uint256 {
    fn from(val: u16) -> Self {
        Uint256(val.into())
    }
}

impl From<u8> for Uint256 {
    fn from(val: u8) -> Self {
        Uint256(val.into())
    }
}

impl TryFrom<Uint256> for Uint128 {
    type Error = ConversionOverflowError;

    fn try_from(value: Uint256) -> Result<Self, Self::Error> {
        Ok(Uint128::new(value.0.try_into().map_err(|_| {
            ConversionOverflowError::new("Uint256", "Uint128", value.to_string())
        })?))
    }
}

impl TryFrom<&str> for Uint256 {
    type Error = StdError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        Self::from_str(val)
    }
}

impl FromStr for Uint256 {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(StdError::generic_err("Parsing u256: received empty string"));
        }

        match U256::from_dec_str(s) {
            Ok(u) => Ok(Uint256(u)),
            Err(e) => Err(StdError::generic_err(format!("Parsing u256: {}", e))),
        }
    }
}

impl From<Uint256> for String {
    fn from(original: Uint256) -> Self {
        original.to_string()
    }
}

impl fmt::Display for Uint256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // The inner type doesn't work as expected with padding, so we
        // work around that.
        let unpadded = self.0.to_string();

        f.pad_integral(true, "", &unpadded)
    }
}

impl ops::Add<Uint256> for Uint256 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(
            self.0
                .checked_add(rhs.0)
                .expect("attempt to add with overflow"),
        )
    }
}

impl<'a> ops::Add<&'a Uint256> for Uint256 {
    type Output = Self;

    fn add(self, rhs: &'a Uint256) -> Self {
        self + *rhs
    }
}

impl ops::Sub<Uint256> for Uint256 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(
            self.0
                .checked_sub(rhs.0)
                .expect("attempt to subtract with overflow"),
        )
    }
}

impl<'a> ops::Sub<&'a Uint256> for Uint256 {
    type Output = Self;

    fn sub(self, rhs: &'a Uint256) -> Self {
        self - *rhs
    }
}

impl ops::Div<Uint256> for Uint256 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(
            self.0
                .checked_div(rhs.0)
                .expect("attempt to divide by zero"),
        )
    }
}

impl<'a> ops::Div<&'a Uint256> for Uint256 {
    type Output = Self;

    fn div(self, rhs: &'a Uint256) -> Self::Output {
        self / *rhs
    }
}

impl ops::Mul<Uint256> for Uint256 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(
            self.0
                .checked_mul(rhs.0)
                .expect("attempt to multiply with overflow"),
        )
    }
}

impl<'a> ops::Mul<&'a Uint256> for Uint256 {
    type Output = Self;

    fn mul(self, rhs: &'a Uint256) -> Self::Output {
        self.mul(*rhs)
    }
}

impl ops::Shr<u32> for Uint256 {
    type Output = Self;

    fn shr(self, rhs: u32) -> Self::Output {
        self.checked_shr(rhs).unwrap_or_else(|_| {
            panic!(
                "right shift error: {} is larger or equal than the number of bits in Uint256",
                rhs,
            )
        })
    }
}

impl<'a> ops::Shr<&'a u32> for Uint256 {
    type Output = Self;

    fn shr(self, rhs: &'a u32) -> Self::Output {
        self.shr(*rhs)
    }
}

impl ops::Shl<u32> for Uint256 {
    type Output = Self;

    fn shl(self, rhs: u32) -> Self::Output {
        self.checked_shl(rhs).unwrap_or_else(|_| {
            panic!(
                "left shift error: {} is larger or equal than the number of bits in Uint256",
                rhs,
            )
        })
    }
}

impl<'a> ops::Shl<&'a u32> for Uint256 {
    type Output = Self;

    fn shl(self, rhs: &'a u32) -> Self::Output {
        self.shl(*rhs)
    }
}

impl ops::AddAssign<Uint256> for Uint256 {
    fn add_assign(&mut self, rhs: Uint256) {
        *self = *self + rhs;
    }
}

impl<'a> ops::AddAssign<&'a Uint256> for Uint256 {
    fn add_assign(&mut self, rhs: &'a Uint256) {
        *self = *self + rhs;
    }
}

impl ops::SubAssign<Uint256> for Uint256 {
    fn sub_assign(&mut self, rhs: Uint256) {
        *self = *self - rhs;
    }
}

impl<'a> ops::SubAssign<&'a Uint256> for Uint256 {
    fn sub_assign(&mut self, rhs: &'a Uint256) {
        *self = *self - rhs;
    }
}

impl ops::MulAssign<Uint256> for Uint256 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl<'a> ops::MulAssign<&'a Uint256> for Uint256 {
    fn mul_assign(&mut self, rhs: &'a Uint256) {
        *self = *self * rhs;
    }
}

impl ops::DivAssign<Uint256> for Uint256 {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl<'a> ops::DivAssign<&'a Uint256> for Uint256 {
    fn div_assign(&mut self, rhs: &'a Uint256) {
        *self = *self / rhs;
    }
}

impl ops::ShrAssign<u32> for Uint256 {
    fn shr_assign(&mut self, rhs: u32) {
        *self = Shr::<u32>::shr(*self, rhs);
    }
}

impl<'a> ops::ShrAssign<&'a u32> for Uint256 {
    fn shr_assign(&mut self, rhs: &'a u32) {
        *self = Shr::<u32>::shr(*self, *rhs);
    }
}

impl Uint256 {
    /// Returns `self * numerator / denominator`
    pub fn multiply_ratio<A: Into<Uint256>, B: Into<Uint256>>(
        &self,
        numerator: A,
        denominator: B,
    ) -> Uint256 {
        let numerator: Uint256 = numerator.into();
        let denominator: Uint256 = denominator.into();
        if denominator.is_zero() {
            panic!("Denominator must not be zero");
        }
        (self.full_mul(numerator) / Uint512::from(denominator))
            .try_into()
            .expect("multiplication overflow")
    }

    /// Multiplies two u256 values without overflow, producing an
    /// [`Uint512`].
    ///
    /// # Examples
    ///
    /// ```
    /// use cosmwasm_std::Uint256;
    ///
    /// let a = Uint256::MAX;
    /// let result = a.full_mul(2u32);
    /// assert_eq!(
    ///     result.to_string(),
    ///     "231584178474632390847141970017375815706539969331281128078915168015826259279870",
    /// );
    /// ```
    pub fn full_mul(self, rhs: impl Into<Uint256>) -> Uint512 {
        Uint512::from(self)
            .checked_mul(Uint512::from(rhs.into()))
            .unwrap()
    }
}

impl Serialize for Uint256 {
    /// Serializes as an integer string using base 10
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Uint256 {
    /// Deserialized from an integer string using base 10
    fn deserialize<D>(deserializer: D) -> Result<Uint256, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(Uint256Visitor)
    }
}

struct Uint256Visitor;

impl<'de> de::Visitor<'de> for Uint256Visitor {
    type Value = Uint256;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded integer")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Uint256::try_from(v).map_err(|e| E::custom(format!("invalid Uint256 '{}' - {}", v, e)))
    }
}

impl Sum<Uint256> for Uint256 {
    fn sum<I: Iterator<Item = Uint256>>(iter: I) -> Self {
        iter.fold(Uint256::zero(), ops::Add::add)
    }
}

impl<'a> Sum<&'a Uint256> for Uint256 {
    fn sum<I: Iterator<Item = &'a Uint256>>(iter: I) -> Self {
        iter.fold(Uint256::zero(), ops::Add::add)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_slice, to_vec};

    #[test]
    fn uint256_construct() {
        let num = Uint256::new([1; 32]);
        let a: [u8; 32] = num.to_be_bytes();
        assert_eq!(a, [1; 32]);

        let be_bytes = [
            0u8, 222u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 2u8, 3u8,
        ];
        let num = Uint256::new(be_bytes);
        let resulting_bytes: [u8; 32] = num.to_be_bytes();
        assert_eq!(be_bytes, resulting_bytes);
    }

    #[test]
    fn uint256_from_be_bytes() {
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(0u128));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 42,
        ]);
        assert_eq!(a, Uint256::from(42u128));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 1,
        ]);
        assert_eq!(a, Uint256::from(1u128));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 1, 0,
        ]);
        assert_eq!(a, Uint256::from(256u128));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(65536u128));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(16777216u128));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(4294967296u128));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1099511627776u128));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(281474976710656u128));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(72057594037927936u128));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(18446744073709551616u128));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(4722366482869645213696u128));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1208925819614629174706176u128));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1329227995784915872903807060280344576u128));

        // Values > u128::MAX
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 16));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 17));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 18));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 19));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 20));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 21));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 22));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 23));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 24));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 25));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 26));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 27));
        let a = Uint256::from_be_bytes([
            0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 28));
        let a = Uint256::from_be_bytes([
            0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 29));
        let a = Uint256::from_be_bytes([
            0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 30));
        let a = Uint256::from_be_bytes([
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 31));
    }

    #[test]
    fn uint256_from_le_bytes() {
        let a = Uint256::from_le_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(0u128));
        let a = Uint256::from_le_bytes([
            42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(42u128));
        let a = Uint256::from_le_bytes([
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128));
        let a = Uint256::from_le_bytes([
            0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(256u128));
        let a = Uint256::from_le_bytes([
            0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(65536u128));
        let a = Uint256::from_le_bytes([
            0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(16777216u128));
        let a = Uint256::from_le_bytes([
            0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(4294967296u128));
        let a = Uint256::from_le_bytes([
            0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(72057594037927936u128));
        let a = Uint256::from_le_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(18446744073709551616u128));
        let a = Uint256::from_le_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1329227995784915872903807060280344576u128));

        // Values > u128::MAX
        let a = Uint256::from_le_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 16));
        let a = Uint256::from_le_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 17));
        let a = Uint256::from_le_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 18));
        let a = Uint256::from_le_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 19));
        let a = Uint256::from_le_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 20));
        let a = Uint256::from_le_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 21));
        let a = Uint256::from_le_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 22));
        let a = Uint256::from_le_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 23));
        let a = Uint256::from_le_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 24));
        let a = Uint256::from_le_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 25));
        let a = Uint256::from_le_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 26));
        let a = Uint256::from_le_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 27));
        let a = Uint256::from_le_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            0, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 28));
        let a = Uint256::from_le_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 0, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 29));
        let a = Uint256::from_le_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 1, 0,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 30));
        let a = Uint256::from_le_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 1,
        ]);
        assert_eq!(a, Uint256::from(1u128) << (8 * 31));
    }

    #[test]
    fn uint256_endianness() {
        let be_bytes = [
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 2u8, 3u8,
        ];
        let le_bytes = [
            3u8, 2u8, 1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        ];

        // These should all be the same.
        let num1 = Uint256::new(be_bytes);
        let num2 = Uint256::from_be_bytes(be_bytes);
        let num3 = Uint256::from_le_bytes(le_bytes);
        assert_eq!(num1, Uint256::from(65536u32 + 512 + 3));
        assert_eq!(num1, num2);
        assert_eq!(num1, num3);
    }

    #[test]
    fn uint256_convert_from() {
        let a = Uint256::from(5u128);
        assert_eq!(a.0, U256::from(5));

        let a = Uint256::from(5u64);
        assert_eq!(a.0, U256::from(5));

        let a = Uint256::from(5u32);
        assert_eq!(a.0, U256::from(5));

        let a = Uint256::from(5u16);
        assert_eq!(a.0, U256::from(5));

        let a = Uint256::from(5u8);
        assert_eq!(a.0, U256::from(5));

        let result = Uint256::try_from("34567");
        assert_eq!(result.unwrap().0, U256::from_dec_str("34567").unwrap());

        let result = Uint256::try_from("1.23");
        assert!(result.is_err());
    }

    #[test]
    fn uint256_convert_to_uint128() {
        let source = Uint256::from(42u128);
        let target = Uint128::try_from(source);
        assert_eq!(target, Ok(Uint128::new(42u128)));

        let source = Uint256::MAX;
        let target = Uint128::try_from(source);
        assert_eq!(
            target,
            Err(ConversionOverflowError::new(
                "Uint256",
                "Uint128",
                Uint256::MAX.to_string()
            ))
        );
    }

    #[test]
    fn uint256_implements_display() {
        let a = Uint256::from(12345u32);
        assert_eq!(format!("Embedded: {}", a), "Embedded: 12345");
        assert_eq!(a.to_string(), "12345");

        let a = Uint256::zero();
        assert_eq!(format!("Embedded: {}", a), "Embedded: 0");
        assert_eq!(a.to_string(), "0");
    }

    #[test]
    fn uint256_display_padding_works() {
        let a = Uint256::from(123u64);
        assert_eq!(format!("Embedded: {:05}", a), "Embedded: 00123");
    }

    #[test]
    fn uint256_to_be_bytes_works() {
        assert_eq!(
            Uint256::zero().to_be_bytes(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ]
        );
        assert_eq!(
            Uint256::MAX.to_be_bytes(),
            [
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff,
            ]
        );
        assert_eq!(
            Uint256::from(1u128).to_be_bytes(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 1
            ]
        );
        // Python: `[b for b in (240282366920938463463374607431768124608).to_bytes(32, "big")]`
        assert_eq!(
            Uint256::from(240282366920938463463374607431768124608u128).to_be_bytes(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 180, 196, 179, 87, 165, 121, 59,
                133, 246, 117, 221, 191, 255, 254, 172, 192
            ]
        );
        assert_eq!(
            Uint256::from_be_bytes([
                233, 2, 240, 200, 115, 150, 240, 218, 88, 106, 45, 208, 134, 238, 119, 85, 22, 14,
                88, 166, 195, 154, 73, 64, 10, 44, 252, 96, 230, 187, 38, 29
            ])
            .to_be_bytes(),
            [
                233, 2, 240, 200, 115, 150, 240, 218, 88, 106, 45, 208, 134, 238, 119, 85, 22, 14,
                88, 166, 195, 154, 73, 64, 10, 44, 252, 96, 230, 187, 38, 29
            ]
        );
    }

    #[test]
    fn uint256_to_le_bytes_works() {
        assert_eq!(
            Uint256::zero().to_le_bytes(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0
            ]
        );
        assert_eq!(
            Uint256::MAX.to_le_bytes(),
            [
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff
            ]
        );
        assert_eq!(
            Uint256::from(1u128).to_le_bytes(),
            [
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0
            ]
        );
        // Python: `[b for b in (240282366920938463463374607431768124608).to_bytes(32, "little")]`
        assert_eq!(
            Uint256::from(240282366920938463463374607431768124608u128).to_le_bytes(),
            [
                192, 172, 254, 255, 191, 221, 117, 246, 133, 59, 121, 165, 87, 179, 196, 180, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
        assert_eq!(
            Uint256::from_be_bytes([
                233, 2, 240, 200, 115, 150, 240, 218, 88, 106, 45, 208, 134, 238, 119, 85, 22, 14,
                88, 166, 195, 154, 73, 64, 10, 44, 252, 96, 230, 187, 38, 29
            ])
            .to_le_bytes(),
            [
                29, 38, 187, 230, 96, 252, 44, 10, 64, 73, 154, 195, 166, 88, 14, 22, 85, 119, 238,
                134, 208, 45, 106, 88, 218, 240, 150, 115, 200, 240, 2, 233
            ]
        );
    }

    #[test]
    fn uint256_is_zero_works() {
        assert!(Uint256::zero().is_zero());
        assert!(Uint256(U256::from(0)).is_zero());

        assert!(!Uint256::from(1u32).is_zero());
        assert!(!Uint256::from(123u32).is_zero());
    }

    #[test]
    fn uint256_json() {
        let orig = Uint256::from(1234567890987654321u128);
        let serialized = to_vec(&orig).unwrap();
        assert_eq!(serialized.as_slice(), b"\"1234567890987654321\"");
        let parsed: Uint256 = from_slice(&serialized).unwrap();
        assert_eq!(parsed, orig);
    }

    #[test]
    fn uint256_compare() {
        let a = Uint256::from(12345u32);
        let b = Uint256::from(23456u32);

        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, Uint256::from(12345u32));
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn uint256_math() {
        let a = Uint256::from(12345u32);
        let b = Uint256::from(23456u32);

        // test + with owned and reference right hand side
        assert_eq!(a + b, Uint256::from(35801u32));
        assert_eq!(a + &b, Uint256::from(35801u32));

        // test - with owned and reference right hand side
        assert_eq!(b - a, Uint256::from(11111u32));
        assert_eq!(b - &a, Uint256::from(11111u32));

        // test += with owned and reference right hand side
        let mut c = Uint256::from(300000u32);
        c += b;
        assert_eq!(c, Uint256::from(323456u32));
        let mut d = Uint256::from(300000u32);
        d += &b;
        assert_eq!(d, Uint256::from(323456u32));

        // test -= with owned and reference right hand side
        let mut c = Uint256::from(300000u32);
        c -= b;
        assert_eq!(c, Uint256::from(276544u32));
        let mut d = Uint256::from(300000u32);
        d -= &b;
        assert_eq!(d, Uint256::from(276544u32));

        // error result on underflow (- would produce negative result)
        let underflow_result = a.checked_sub(b);
        let OverflowError {
            operand1, operand2, ..
        } = underflow_result.unwrap_err();
        assert_eq!((operand1, operand2), (a.to_string(), b.to_string()));
    }

    #[test]
    #[should_panic]
    fn uint256_add_overflow_panics() {
        let max = Uint256::new([255u8; 32]);
        let _ = max + Uint256::from(12u32);
    }

    #[test]
    #[should_panic]
    fn uint256_sub_overflow_panics() {
        let _ = Uint256::from(1u32) - Uint256::from(2u32);
    }

    #[test]
    fn uint256_multiply_ratio_works() {
        let base = Uint256::from(500u32);

        // factor 1/1
        assert_eq!(base.multiply_ratio(1u128, 1u128), base);
        assert_eq!(base.multiply_ratio(3u128, 3u128), base);
        assert_eq!(base.multiply_ratio(654321u128, 654321u128), base);
        assert_eq!(base.multiply_ratio(Uint256::MAX, Uint256::MAX), base);

        // factor 3/2
        assert_eq!(base.multiply_ratio(3u128, 2u128), Uint256::from(750u32));
        assert_eq!(
            base.multiply_ratio(333333u128, 222222u128),
            Uint256::from(750u32)
        );

        // factor 2/3 (integer devision always floors the result)
        assert_eq!(base.multiply_ratio(2u128, 3u128), Uint256::from(333u32));
        assert_eq!(
            base.multiply_ratio(222222u128, 333333u128),
            Uint256::from(333u32)
        );

        // factor 5/6 (integer devision always floors the result)
        assert_eq!(base.multiply_ratio(5u128, 6u128), Uint256::from(416u32));
        assert_eq!(base.multiply_ratio(100u128, 120u128), Uint256::from(416u32));
    }

    #[test]
    fn uint256_multiply_ratio_does_not_overflow_when_result_fits() {
        // Almost max value for Uint256.
        let base = Uint256::MAX - Uint256::from(9u8);

        assert_eq!(base.multiply_ratio(2u128, 2u128), base);
    }

    #[test]
    #[should_panic]
    fn uint256_multiply_ratio_panicks_on_overflow() {
        // Almost max value for Uint256.
        let base = Uint256::MAX - Uint256::from(9u8);

        assert_eq!(base.multiply_ratio(2u128, 1u128), base);
    }

    #[test]
    #[should_panic(expected = "Denominator must not be zero")]
    fn uint256_multiply_ratio_panics_for_zero_denominator() {
        Uint256::from(500u32).multiply_ratio(1u128, 0u128);
    }

    #[test]
    fn uint256_shr_works() {
        let original = Uint256::new([
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 2u8, 0u8, 4u8, 2u8,
        ]);

        let shifted = Uint256::new([
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 128u8, 1u8, 0u8,
        ]);

        assert_eq!(original >> 2u32, shifted);
    }

    #[test]
    #[should_panic]
    fn uint256_shr_overflow_panics() {
        let _ = Uint256::from(1u32) >> 256u32;
    }

    #[test]
    fn sum_works() {
        let nums = vec![
            Uint256::from(17u32),
            Uint256::from(123u32),
            Uint256::from(540u32),
            Uint256::from(82u32),
        ];
        let expected = Uint256::from(762u32);

        let sum_as_ref = nums.iter().sum();
        assert_eq!(expected, sum_as_ref);

        let sum_as_owned = nums.into_iter().sum();
        assert_eq!(expected, sum_as_owned);
    }

    #[test]
    fn uint256_methods() {
        // checked_*
        assert!(matches!(
            Uint256::MAX.checked_add(Uint256::from(1u32)),
            Err(OverflowError { .. })
        ));
        assert!(matches!(
            Uint256::from(0u32).checked_sub(Uint256::from(1u32)),
            Err(OverflowError { .. })
        ));
        assert!(matches!(
            Uint256::MAX.checked_mul(Uint256::from(2u32)),
            Err(OverflowError { .. })
        ));
        assert!(matches!(
            Uint256::MAX.checked_div(Uint256::from(0u32)),
            Err(DivideByZeroError { .. })
        ));
        assert!(matches!(
            Uint256::MAX.checked_rem(Uint256::from(0u32)),
            Err(DivideByZeroError { .. })
        ));

        // saturating_*
        assert_eq!(
            Uint256::MAX.saturating_add(Uint256::from(1u32)),
            Uint256::MAX
        );
        assert_eq!(
            Uint256::from(0u32).saturating_sub(Uint256::from(1u32)),
            Uint256::from(0u32)
        );
        assert_eq!(
            Uint256::MAX.saturating_mul(Uint256::from(2u32)),
            Uint256::MAX
        );
    }
}
