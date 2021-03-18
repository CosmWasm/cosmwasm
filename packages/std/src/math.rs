use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use std::convert::TryFrom;
use std::fmt::{self, Write};
use std::iter::Sum;
use std::ops;
use std::str::FromStr;

use crate::errors::{StdError, StdResult};

/// A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
///
/// The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct Decimal(#[schemars(with = "String")] u128);

const DECIMAL_FRACTIONAL: u128 = 1_000_000_000_000_000_000;

impl Decimal {
    pub const MAX: Decimal = Decimal(u128::MAX);

    /// Create a 1.0 Decimal
    pub const fn one() -> Decimal {
        Decimal(DECIMAL_FRACTIONAL)
    }

    /// Create a 0.0 Decimal
    pub const fn zero() -> Decimal {
        Decimal(0)
    }

    /// Convert x% into Decimal
    pub fn percent(x: u64) -> Decimal {
        Decimal((x as u128) * 10_000_000_000_000_000)
    }

    /// Convert permille (x/1000) into Decimal
    pub fn permille(x: u64) -> Decimal {
        Decimal((x as u128) * 1_000_000_000_000_000)
    }

    /// Returns the ratio (nominator / denominator) as a Decimal
    pub fn from_ratio<A: Into<u128>, B: Into<u128>>(nominator: A, denominator: B) -> Decimal {
        let nominator: u128 = nominator.into();
        let denominator: u128 = denominator.into();
        if denominator == 0 {
            panic!("Denominator must not be zero");
        }
        // TODO: better algorithm with less rounding potential?
        Decimal(nominator * DECIMAL_FRACTIONAL / denominator)
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl FromStr for Decimal {
    type Err = StdError;

    /// Converts the decimal string to a Decimal
    /// Possible inputs: "1.23", "1", "000012", "1.123000000"
    /// Disallowed: "", ".23"
    ///
    /// This never performs any kind of rounding.
    /// More than 18 fractional digits, even zeros, result in an error.
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parts_iter = input.split('.');

        let whole_part = parts_iter.next().unwrap(); // split always returns at least one element
        let whole = whole_part
            .parse::<u128>()
            .map_err(|_| StdError::generic_err("Error parsing whole"))?;
        let mut atomics = whole
            .checked_mul(DECIMAL_FRACTIONAL)
            .ok_or_else(|| StdError::generic_err("Value too big"))?;

        if let Some(fractional_part) = parts_iter.next() {
            let fractional = fractional_part
                .parse::<u128>()
                .map_err(|_| StdError::generic_err("Error parsing fractional"))?;
            let exp = (18usize.checked_sub(fractional_part.len())).ok_or_else(|| {
                StdError::generic_err("Cannot parse more than 18 fractional digits")
            })?;
            debug_assert!(exp <= 18);
            let fractional_factor = 10u128.pow(exp as u32);
            atomics = atomics
                .checked_add(fractional * fractional_factor)
                .ok_or_else(|| StdError::generic_err("Value too big"))?;
        }

        if parts_iter.next().is_some() {
            return Err(StdError::generic_err("Unexpected number of dots"));
        }

        Ok(Decimal(atomics))
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let whole = (self.0) / DECIMAL_FRACTIONAL;
        let fractional = (self.0) % DECIMAL_FRACTIONAL;

        if fractional == 0 {
            write!(f, "{}", whole)
        } else {
            let fractional_string = format!("{:018}", fractional);
            f.write_str(&whole.to_string())?;
            f.write_char('.')?;
            f.write_str(fractional_string.trim_end_matches('0'))?;
            Ok(())
        }
    }
}

impl ops::Add for Decimal {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Decimal(self.0 + other.0)
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
    fn deserialize<D>(deserializer: D) -> Result<Decimal, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(DecimalVisitor)
    }
}

struct DecimalVisitor;

impl<'de> de::Visitor<'de> for DecimalVisitor {
    type Value = Decimal;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded decimal")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match Decimal::from_str(v) {
            Ok(d) => Ok(d),
            Err(e) => Err(E::custom(format!("Error parsing decimal '{}': {}", v, e))),
        }
    }
}

//*** Uint128 ***/
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct Uint128(#[schemars(with = "String")] pub u128);

impl Uint128 {
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
            Err(e) => Err(StdError::generic_err(format!("Parsing coin: {}", e))),
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
        Uint128(self.u128() + rhs.u128())
    }
}

impl<'a> ops::Add<&'a Uint128> for Uint128 {
    type Output = Self;

    fn add(self, rhs: &'a Uint128) -> Self {
        Uint128(self.u128() + rhs.u128())
    }
}

impl ops::AddAssign<Uint128> for Uint128 {
    fn add_assign(&mut self, rhs: Uint128) {
        self.0 += rhs.u128();
    }
}

impl<'a> ops::AddAssign<&'a Uint128> for Uint128 {
    fn add_assign(&mut self, rhs: &'a Uint128) {
        self.0 += rhs.u128();
    }
}

impl ops::Sub<Uint128> for Uint128 {
    type Output = StdResult<Self>;

    fn sub(self, other: Uint128) -> StdResult<Self> {
        self.sub(&other)
    }
}

impl<'a> ops::Sub<&'a Uint128> for Uint128 {
    type Output = StdResult<Self>;

    fn sub(self, rhs: &'a Uint128) -> StdResult<Self> {
        let (min, sub) = (self.u128(), rhs.u128());
        min.checked_sub(sub)
            .map(Uint128)
            .ok_or_else(|| StdError::underflow(min, sub))
    }
}

/// Both d*u and u*d with d: Decimal and u: Uint128 returns an Uint128. There is no
/// specific reason for this decision other than the initial use cases we have. If you
/// need a Decimal result for the same calculation, use Decimal(d*u) or Decimal(u*d).
impl ops::Mul<Decimal> for Uint128 {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, rhs: Decimal) -> Self::Output {
        // 0*a and b*0 is always 0
        if self.is_zero() || rhs.is_zero() {
            return Uint128::zero();
        }
        self.multiply_ratio(rhs.0, DECIMAL_FRACTIONAL)
    }
}

impl ops::Mul<Uint128> for Decimal {
    type Output = Uint128;

    fn mul(self, rhs: Uint128) -> Self::Output {
        rhs * self
    }
}

impl Uint128 {
    /// returns self * nom / denom
    pub fn multiply_ratio<A: Into<u128>, B: Into<u128>>(&self, nom: A, denom: B) -> Uint128 {
        let nominator: u128 = nom.into();
        let denominator: u128 = denom.into();
        if denominator == 0 {
            panic!("Denominator must not be zero");
        }
        // TODO: minimize rounding that takes place (using gcd algorithm)
        let val = self.u128() * nominator / denominator;
        Uint128::from(val)
    }
}

/// Serializes as a base64 string
impl Serialize for Uint128 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Deserializes as a base64 string
impl<'de> Deserialize<'de> for Uint128 {
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
    use crate::errors::StdError;
    use crate::{from_slice, to_vec};

    #[test]
    fn decimal_one() {
        let value = Decimal::one();
        assert_eq!(value.0, DECIMAL_FRACTIONAL);
    }

    #[test]
    fn decimal_zero() {
        let value = Decimal::zero();
        assert_eq!(value.0, 0);
    }

    #[test]
    fn decimal_percent() {
        let value = Decimal::percent(50);
        assert_eq!(value.0, DECIMAL_FRACTIONAL / 2);
    }

    #[test]
    fn decimal_permille() {
        let value = Decimal::permille(125);
        assert_eq!(value.0, DECIMAL_FRACTIONAL / 8);
    }

    #[test]
    fn decimal_from_ratio_works() {
        // 1.0
        assert_eq!(Decimal::from_ratio(1u128, 1u128), Decimal::one());
        assert_eq!(Decimal::from_ratio(53u128, 53u128), Decimal::one());
        assert_eq!(Decimal::from_ratio(125u128, 125u128), Decimal::one());

        // 1.5
        assert_eq!(Decimal::from_ratio(3u128, 2u128), Decimal::percent(150));
        assert_eq!(Decimal::from_ratio(150u128, 100u128), Decimal::percent(150));
        assert_eq!(Decimal::from_ratio(333u128, 222u128), Decimal::percent(150));

        // 0.125
        assert_eq!(Decimal::from_ratio(1u64, 8u64), Decimal::permille(125));
        assert_eq!(Decimal::from_ratio(125u64, 1000u64), Decimal::permille(125));

        // 1/3 (result floored)
        assert_eq!(
            Decimal::from_ratio(1u64, 3u64),
            Decimal(333_333_333_333_333_333)
        );

        // 2/3 (result floored)
        assert_eq!(
            Decimal::from_ratio(2u64, 3u64),
            Decimal(666_666_666_666_666_666)
        );
    }

    #[test]
    #[should_panic(expected = "Denominator must not be zero")]
    fn decimal_from_ratio_panics_for_zero_denominator() {
        Decimal::from_ratio(1u128, 0u128);
    }

    #[test]
    fn decimal_from_str_works() {
        // Integers
        assert_eq!(Decimal::from_str("0").unwrap(), Decimal::percent(0));
        assert_eq!(Decimal::from_str("1").unwrap(), Decimal::percent(100));
        assert_eq!(Decimal::from_str("5").unwrap(), Decimal::percent(500));
        assert_eq!(Decimal::from_str("42").unwrap(), Decimal::percent(4200));
        assert_eq!(Decimal::from_str("000").unwrap(), Decimal::percent(0));
        assert_eq!(Decimal::from_str("001").unwrap(), Decimal::percent(100));
        assert_eq!(Decimal::from_str("005").unwrap(), Decimal::percent(500));
        assert_eq!(Decimal::from_str("0042").unwrap(), Decimal::percent(4200));

        // Decimals
        assert_eq!(Decimal::from_str("1.0").unwrap(), Decimal::percent(100));
        assert_eq!(Decimal::from_str("1.5").unwrap(), Decimal::percent(150));
        assert_eq!(Decimal::from_str("0.5").unwrap(), Decimal::percent(50));
        assert_eq!(Decimal::from_str("0.123").unwrap(), Decimal::permille(123));

        assert_eq!(Decimal::from_str("40.00").unwrap(), Decimal::percent(4000));
        assert_eq!(Decimal::from_str("04.00").unwrap(), Decimal::percent(400));
        assert_eq!(Decimal::from_str("00.40").unwrap(), Decimal::percent(40));
        assert_eq!(Decimal::from_str("00.04").unwrap(), Decimal::percent(4));

        // Can handle 18 fractional digits
        assert_eq!(
            Decimal::from_str("7.123456789012345678").unwrap(),
            Decimal(7123456789012345678)
        );
        assert_eq!(
            Decimal::from_str("7.999999999999999999").unwrap(),
            Decimal(7999999999999999999)
        );

        // Works for documented max value
        assert_eq!(
            Decimal::from_str("340282366920938463463.374607431768211455").unwrap(),
            Decimal::MAX
        );
    }

    #[test]
    fn decimal_from_str_errors_for_broken_whole_part() {
        match Decimal::from_str("").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing whole"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal::from_str(" ").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing whole"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal::from_str("-1").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing whole"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_from_str_errors_for_broken_fractinal_part() {
        match Decimal::from_str("1.").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal::from_str("1. ").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal::from_str("1.e").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal::from_str("1.2e3").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_from_str_errors_for_more_than_18_fractional_digits() {
        match Decimal::from_str("7.1234567890123456789").unwrap_err() {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "Cannot parse more than 18 fractional digits")
            }
            e => panic!("Unexpected error: {:?}", e),
        }

        // No special rules for trailing zeros. This could be changed but adds gas cost for the happy path.
        match Decimal::from_str("7.1230000000000000000").unwrap_err() {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "Cannot parse more than 18 fractional digits")
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_from_str_errors_for_invalid_number_of_dots() {
        match Decimal::from_str("1.2.3").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Unexpected number of dots"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal::from_str("1.2.3.4").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Unexpected number of dots"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_from_str_errors_for_more_than_max_value() {
        // Integer
        match Decimal::from_str("340282366920938463464").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Value too big"),
            e => panic!("Unexpected error: {:?}", e),
        }

        // Decimal
        match Decimal::from_str("340282366920938463464.0").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Value too big"),
            e => panic!("Unexpected error: {:?}", e),
        }
        match Decimal::from_str("340282366920938463463.374607431768211456").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Value too big"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_is_zero_works() {
        assert_eq!(Decimal::zero().is_zero(), true);
        assert_eq!(Decimal::percent(0).is_zero(), true);
        assert_eq!(Decimal::permille(0).is_zero(), true);

        assert_eq!(Decimal::one().is_zero(), false);
        assert_eq!(Decimal::percent(123).is_zero(), false);
        assert_eq!(Decimal::permille(1234).is_zero(), false);
    }

    #[test]
    fn decimal_add() {
        let value = Decimal::one() + Decimal::percent(50); // 1.5
        assert_eq!(value.0, DECIMAL_FRACTIONAL * 3 / 2);
    }

    #[test]
    fn decimal_to_string() {
        // Integers
        assert_eq!(Decimal::zero().to_string(), "0");
        assert_eq!(Decimal::one().to_string(), "1");
        assert_eq!(Decimal::percent(500).to_string(), "5");

        // Decimals
        assert_eq!(Decimal::percent(125).to_string(), "1.25");
        assert_eq!(Decimal::percent(42638).to_string(), "426.38");
        assert_eq!(Decimal::percent(1).to_string(), "0.01");
        assert_eq!(Decimal::permille(987).to_string(), "0.987");

        assert_eq!(Decimal(1).to_string(), "0.000000000000000001");
        assert_eq!(Decimal(10).to_string(), "0.00000000000000001");
        assert_eq!(Decimal(100).to_string(), "0.0000000000000001");
        assert_eq!(Decimal(1000).to_string(), "0.000000000000001");
        assert_eq!(Decimal(10000).to_string(), "0.00000000000001");
        assert_eq!(Decimal(100000).to_string(), "0.0000000000001");
        assert_eq!(Decimal(1000000).to_string(), "0.000000000001");
        assert_eq!(Decimal(10000000).to_string(), "0.00000000001");
        assert_eq!(Decimal(100000000).to_string(), "0.0000000001");
        assert_eq!(Decimal(1000000000).to_string(), "0.000000001");
        assert_eq!(Decimal(10000000000).to_string(), "0.00000001");
        assert_eq!(Decimal(100000000000).to_string(), "0.0000001");
        assert_eq!(Decimal(10000000000000).to_string(), "0.00001");
        assert_eq!(Decimal(100000000000000).to_string(), "0.0001");
        assert_eq!(Decimal(1000000000000000).to_string(), "0.001");
        assert_eq!(Decimal(10000000000000000).to_string(), "0.01");
        assert_eq!(Decimal(100000000000000000).to_string(), "0.1");
    }

    #[test]
    fn decimal_serialize() {
        assert_eq!(to_vec(&Decimal::zero()).unwrap(), br#""0""#);
        assert_eq!(to_vec(&Decimal::one()).unwrap(), br#""1""#);
        assert_eq!(to_vec(&Decimal::percent(8)).unwrap(), br#""0.08""#);
        assert_eq!(to_vec(&Decimal::percent(87)).unwrap(), br#""0.87""#);
        assert_eq!(to_vec(&Decimal::percent(876)).unwrap(), br#""8.76""#);
        assert_eq!(to_vec(&Decimal::percent(8765)).unwrap(), br#""87.65""#);
    }

    #[test]
    fn decimal_deserialize() {
        assert_eq!(from_slice::<Decimal>(br#""0""#).unwrap(), Decimal::zero());
        assert_eq!(from_slice::<Decimal>(br#""1""#).unwrap(), Decimal::one());
        assert_eq!(from_slice::<Decimal>(br#""000""#).unwrap(), Decimal::zero());
        assert_eq!(from_slice::<Decimal>(br#""001""#).unwrap(), Decimal::one());

        assert_eq!(
            from_slice::<Decimal>(br#""0.08""#).unwrap(),
            Decimal::percent(8)
        );
        assert_eq!(
            from_slice::<Decimal>(br#""0.87""#).unwrap(),
            Decimal::percent(87)
        );
        assert_eq!(
            from_slice::<Decimal>(br#""8.76""#).unwrap(),
            Decimal::percent(876)
        );
        assert_eq!(
            from_slice::<Decimal>(br#""87.65""#).unwrap(),
            Decimal::percent(8765)
        );
    }

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
        assert_eq!(Uint128::zero().is_zero(), true);
        assert_eq!(Uint128(0).is_zero(), true);

        assert_eq!(Uint128(1).is_zero(), false);
        assert_eq!(Uint128(123).is_zero(), false);
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
        assert_eq!((b - a).unwrap(), Uint128(11111));
        assert_eq!((b - &a).unwrap(), Uint128(11111));

        // test += with owned and reference right hand side
        let mut c = Uint128(300000);
        c += b;
        assert_eq!(c, Uint128(323456));
        let mut d = Uint128(300000);
        d += &b;
        assert_eq!(d, Uint128(323456));

        // error result on underflow (- would produce negative result)
        let underflow_result = a - b;
        match underflow_result.unwrap_err() {
            StdError::Underflow {
                minuend,
                subtrahend,
                ..
            } => assert_eq!((minuend, subtrahend), (a.to_string(), b.to_string())),
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    #[should_panic]
    fn uint128_math_overflow_panics() {
        // almost_max is 2^128 - 10
        let almost_max = Uint128(340282366920938463463374607431768211446);
        let _ = almost_max + Uint128(12);
    }

    #[test]
    // in this test the Decimal is on the right
    fn uint128_decimal_multiply() {
        // a*b
        let left = Uint128(300);
        let right = Decimal::one() + Decimal::percent(50); // 1.5
        assert_eq!(left * right, Uint128(450));

        // a*0
        let left = Uint128(300);
        let right = Decimal::zero();
        assert_eq!(left * right, Uint128(0));

        // 0*a
        let left = Uint128(0);
        let right = Decimal::one() + Decimal::percent(50); // 1.5
        assert_eq!(left * right, Uint128(0));
    }

    #[test]
    fn u128_multiply_ratio_works() {
        let base = Uint128(500);

        // factor 1/1
        assert_eq!(base.multiply_ratio(1u128, 1u128), Uint128(500));
        assert_eq!(base.multiply_ratio(3u128, 3u128), Uint128(500));
        assert_eq!(base.multiply_ratio(654321u128, 654321u128), Uint128(500));

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
    #[should_panic(expected = "Denominator must not be zero")]
    fn u128_multiply_ratio_panics_for_zero_denominator() {
        Uint128(500).multiply_ratio(1u128, 0u128);
    }

    #[test]
    // in this test the Decimal is on the left
    fn decimal_uint128_multiply() {
        // a*b
        let left = Decimal::one() + Decimal::percent(50); // 1.5
        let right = Uint128(300);
        assert_eq!(left * right, Uint128(450));

        // 0*a
        let left = Decimal::zero();
        let right = Uint128(300);
        assert_eq!(left * right, Uint128(0));

        // a*0
        let left = Decimal::one() + Decimal::percent(50); // 1.5
        let right = Uint128(0);
        assert_eq!(left * right, Uint128(0));
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
}
