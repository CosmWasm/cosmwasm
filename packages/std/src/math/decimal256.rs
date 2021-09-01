use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use std::fmt::{self, Write};
use std::ops;
use std::str::FromStr;

use crate::errors::StdError;

use super::Fraction;
use super::Isqrt;
use super::Uint256;

/// A fixed-point decimal value with 36 fractional digits, i.e. Decimal256(1_000_000_000_000_000_000) == 1.0
///
/// The greatest possible value that can be represented is
/// 115792089237316195423570985008687907853269.984665640564039457584007913129639935
/// (which is (2^256 - 1) / 10^36)
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct Decimal256(#[schemars(with = "String")] Uint256);

impl Decimal256 {
    fn decimal_fractional() -> Uint256 {
        // 1*10**36
        Uint256::from(1_000_000_000_000_000_000_000_000_000_000_000_000u128)
    }

    fn decimal_fractional_squared() -> Uint256 {
        // 1*10**72
        Uint256::new([
            0, 0, 144, 228, 15, 190, 234, 29, 58, 74, 188, 137, 85, 233, 70, 254, 49, 205, 207,
            102, 246, 52, 225, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ])
    }

    pub const MAX: Self = Self(Uint256::MAX);

    /// Create a 1.0 Decimal256
    pub fn one() -> Self {
        Self(Self::decimal_fractional())
    }

    /// Create a 0.0 Decimal256
    pub const fn zero() -> Self {
        Self(Uint256::zero())
    }

    /// Convert x% into Decimal256
    pub fn percent(x: u64) -> Self {
        Self(Uint256::from(x) * Uint256::from(10_000_000_000_000_000_000_000_000_000_000_000u128))
    }

    /// Convert permille (x/1000) into Decimal256
    pub fn permille(x: u64) -> Self {
        Self(Uint256::from(x) * Uint256::from(1_000_000_000_000_000_000_000_000_000_000_000u128))
    }

    /// Returns the ratio (numerator / denominator) as a Decimal256
    pub fn from_ratio(numerator: impl Into<Uint256>, denominator: impl Into<Uint256>) -> Self {
        let numerator: Uint256 = numerator.into();
        let denominator: Uint256 = denominator.into();
        if denominator.is_zero() {
            panic!("Denominator must not be zero");
        }

        Self(
            // numerator * DECIMAL_FRACTIONAL / denominator
            numerator.multiply_ratio(Self::decimal_fractional(), denominator),
        )
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Returns the approximate square root as a Decimal256.
    ///
    /// This should not overflow or panic.
    pub fn sqrt(&self) -> Self {
        // Algorithm described in https://hackmd.io/@webmaster128/SJThlukj_
        // We start with the highest precision possible and lower it until
        // there's no overflow.
        (0..=18)
            .rev()
            .find_map(|i| self.sqrt_with_precision(i))
            // The last step (i = 0) is guaranteed to succeed because `isqrt(Uint256::MAX) * 10^18` does not overflow
            .unwrap()
    }

    /// Lower precision means more aggressive rounding, but less risk of overflow.
    /// Precision *must* be a number between 0 and 36 (inclusive).
    ///
    /// Returns `None` if the internal multiplication overflows.
    fn sqrt_with_precision(&self, precision: u32) -> Option<Self> {
        let inner_mul = Uint256::from(100u128).pow(precision);
        self.0.checked_mul(inner_mul).ok().map(|inner| {
            let outer_mul = Uint256::from(10u128).pow(18 - precision);
            Self(inner.isqrt().checked_mul(outer_mul).unwrap())
        })
    }
}

impl Fraction<Uint256> for Decimal256 {
    #[inline]
    fn numerator(&self) -> Uint256 {
        self.0
    }

    #[inline]
    fn denominator(&self) -> Uint256 {
        Self::decimal_fractional()
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
            Some(Self(Self::decimal_fractional_squared() / self.0))
        }
    }
}

impl FromStr for Decimal256 {
    type Err = StdError;

    /// Converts the decimal string to a Decimal256
    /// Possible inputs: "1.23", "1", "000012", "1.123000000"
    /// Disallowed: "", ".23"
    ///
    /// This never performs any kind of rounding.
    /// More than 36 fractional digits, even zeros, result in an error.
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parts_iter = input.split('.');

        let whole_part = parts_iter.next().unwrap(); // split always returns at least one element
        let whole = whole_part
            .parse::<Uint256>()
            .map_err(|_| StdError::generic_err("Error parsing whole"))?;
        let mut atomics = whole
            .checked_mul(Self::decimal_fractional())
            .map_err(|_| StdError::generic_err("Value too big"))?;

        if let Some(fractional_part) = parts_iter.next() {
            let fractional = fractional_part
                .parse::<Uint256>()
                .map_err(|_| StdError::generic_err("Error parsing fractional"))?;
            let exp = (36usize.checked_sub(fractional_part.len())).ok_or_else(|| {
                StdError::generic_err("Cannot parse more than 36 fractional digits")
            })?;
            debug_assert!(exp <= 36);
            let fractional_factor = Uint256::from(10u128).pow(exp as u32);
            atomics = atomics
                .checked_add(
                    // The inner multiplication can't overflow because
                    // fractional < 10^36 && fractional_factor <= 10^36
                    fractional.checked_mul(fractional_factor).unwrap(),
                )
                .map_err(|_| StdError::generic_err("Value too big"))?;
        }

        if parts_iter.next().is_some() {
            return Err(StdError::generic_err("Unexpected number of dots"));
        }

        Ok(Self(atomics))
    }
}

impl fmt::Display for Decimal256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let whole = (self.0) / Self::decimal_fractional();
        let fractional = (self.0).checked_rem(Self::decimal_fractional()).unwrap();

        if fractional.is_zero() {
            write!(f, "{}", whole)
        } else {
            let fractional_string = format!("{:036}", fractional);
            f.write_str(&whole.to_string())?;
            f.write_char('.')?;
            f.write_str(fractional_string.trim_end_matches('0'))?;
            Ok(())
        }
    }
}

impl ops::Add for Decimal256 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl ops::Sub for Decimal256 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

/// Both d*u and u*d with d: Decimal256 and u: Uint256 returns an Uint256. There is no
/// specific reason for this decision other than the initial use cases we have. If you
/// need a Decimal256 result for the same calculation, use Decimal256(d*u) or Decimal256(u*d).
impl ops::Mul<Decimal256> for Uint256 {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, rhs: Decimal256) -> Self::Output {
        // 0*a and b*0 is always 0
        if self.is_zero() || rhs.is_zero() {
            return Uint256::zero();
        }
        self.multiply_ratio(rhs.0, Decimal256::decimal_fractional())
    }
}

impl ops::Mul<Uint256> for Decimal256 {
    type Output = Uint256;

    fn mul(self, rhs: Uint256) -> Self::Output {
        rhs * self
    }
}

impl ops::Div<Uint256> for Decimal256 {
    type Output = Self;

    fn div(self, rhs: Uint256) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl ops::DivAssign<Uint256> for Decimal256 {
    fn div_assign(&mut self, rhs: Uint256) {
        self.0 /= rhs;
    }
}

/// Serializes as a decimal string
impl Serialize for Decimal256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Deserializes as a base64 string
impl<'de> Deserialize<'de> for Decimal256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(Decimal256Visitor)
    }
}

struct Decimal256Visitor;

impl<'de> de::Visitor<'de> for Decimal256Visitor {
    type Value = Decimal256;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded decimal")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match Self::Value::from_str(v) {
            Ok(d) => Ok(d),
            Err(e) => Err(E::custom(format!("Error parsing decimal '{}': {}", v, e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::StdError;
    use crate::{from_slice, to_vec};

    #[test]
    fn decimal_one() {
        let value = Decimal256::one();
        assert_eq!(value.0, Decimal256::decimal_fractional());
    }

    #[test]
    fn decimal_zero() {
        let value = Decimal256::zero();
        assert!(value.0.is_zero());
    }

    #[test]
    fn decimal_percent() {
        let value = Decimal256::percent(50);
        assert_eq!(
            value.0,
            Decimal256::decimal_fractional() / Uint256::from(2u8)
        );
    }

    #[test]
    fn decimal_permille() {
        let value = Decimal256::permille(125);
        assert_eq!(
            value.0,
            Decimal256::decimal_fractional() / Uint256::from(8u8)
        );
    }

    #[test]
    fn decimal_from_ratio_works() {
        // 1.0
        assert_eq!(Decimal256::from_ratio(1u128, 1u128), Decimal256::one());
        assert_eq!(Decimal256::from_ratio(53u128, 53u128), Decimal256::one());
        assert_eq!(Decimal256::from_ratio(125u128, 125u128), Decimal256::one());

        // 1.5
        assert_eq!(
            Decimal256::from_ratio(3u128, 2u128),
            Decimal256::percent(150)
        );
        assert_eq!(
            Decimal256::from_ratio(150u128, 100u128),
            Decimal256::percent(150)
        );
        assert_eq!(
            Decimal256::from_ratio(333u128, 222u128),
            Decimal256::percent(150)
        );

        // 0.125
        assert_eq!(
            Decimal256::from_ratio(1u64, 8u64),
            Decimal256::permille(125)
        );
        assert_eq!(
            Decimal256::from_ratio(125u64, 1000u64),
            Decimal256::permille(125)
        );

        // 1/3 (result floored)
        assert_eq!(
            Decimal256::from_ratio(1u64, 3u64),
            Decimal256(Uint256::from_str("333333333333333333333333333333333333").unwrap())
        );

        // 2/3 (result floored)
        assert_eq!(
            Decimal256::from_ratio(2u64, 3u64),
            Decimal256(Uint256::from_str("666666666666666666666666666666666666").unwrap())
        );

        // large inputs
        assert_eq!(Decimal256::from_ratio(0u128, u128::MAX), Decimal256::zero());
        assert_eq!(
            Decimal256::from_ratio(u128::MAX, u128::MAX),
            Decimal256::one()
        );
        // 340282366920938463463 is the largest integer <= Decimal256::MAX
        assert_eq!(
            Decimal256::from_ratio(340282366920938463463u128, 1u128),
            Decimal256::from_str("340282366920938463463").unwrap()
        );
    }

    #[test]
    #[should_panic(expected = "Denominator must not be zero")]
    fn decimal_from_ratio_panics_for_zero_denominator() {
        Decimal256::from_ratio(1u128, 0u128);
    }

    #[test]
    fn decimal_implements_fraction() {
        let fraction = Decimal256::from_str("1234.567").unwrap();
        assert_eq!(
            fraction.numerator(),
            Uint256::from_str("1234567000000000000000000000000000000000").unwrap()
        );
        assert_eq!(
            fraction.denominator(),
            Uint256::from_str("1000000000000000000000000000000000000").unwrap()
        );
    }

    #[test]
    fn decimal_from_str_works() {
        // Integers
        assert_eq!(Decimal256::from_str("0").unwrap(), Decimal256::percent(0));
        assert_eq!(Decimal256::from_str("1").unwrap(), Decimal256::percent(100));
        assert_eq!(Decimal256::from_str("5").unwrap(), Decimal256::percent(500));
        assert_eq!(
            Decimal256::from_str("42").unwrap(),
            Decimal256::percent(4200)
        );
        assert_eq!(Decimal256::from_str("000").unwrap(), Decimal256::percent(0));
        assert_eq!(
            Decimal256::from_str("001").unwrap(),
            Decimal256::percent(100)
        );
        assert_eq!(
            Decimal256::from_str("005").unwrap(),
            Decimal256::percent(500)
        );
        assert_eq!(
            Decimal256::from_str("0042").unwrap(),
            Decimal256::percent(4200)
        );

        // Decimals
        assert_eq!(
            Decimal256::from_str("1.0").unwrap(),
            Decimal256::percent(100)
        );
        assert_eq!(
            Decimal256::from_str("1.5").unwrap(),
            Decimal256::percent(150)
        );
        assert_eq!(
            Decimal256::from_str("0.5").unwrap(),
            Decimal256::percent(50)
        );
        assert_eq!(
            Decimal256::from_str("0.123").unwrap(),
            Decimal256::permille(123)
        );

        assert_eq!(
            Decimal256::from_str("40.00").unwrap(),
            Decimal256::percent(4000)
        );
        assert_eq!(
            Decimal256::from_str("04.00").unwrap(),
            Decimal256::percent(400)
        );
        assert_eq!(
            Decimal256::from_str("00.40").unwrap(),
            Decimal256::percent(40)
        );
        assert_eq!(
            Decimal256::from_str("00.04").unwrap(),
            Decimal256::percent(4)
        );

        // Can handle 36 fractional digits
        assert_eq!(
            Decimal256::from_str("7.123456789012345678123456789012345678").unwrap(),
            Decimal256(Uint256::from(7123456789012345678123456789012345678u128))
        );
        assert_eq!(
            Decimal256::from_str("7.999999999999999999999999999999999999").unwrap(),
            Decimal256(Uint256::from(7999999999999999999999999999999999999u128))
        );

        // Works for documented max value
        assert_eq!(
            Decimal256::from_str(
                "115792089237316195423570985008687907853269.984665640564039457584007913129639935"
            )
            .unwrap(),
            Decimal256::MAX
        );
    }

    #[test]
    fn decimal_from_str_errors_for_broken_whole_part() {
        match Decimal256::from_str("").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing whole"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal256::from_str(" ").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing whole"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal256::from_str("-1").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing whole"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_from_str_errors_for_broken_fractinal_part() {
        match Decimal256::from_str("1.").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal256::from_str("1. ").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal256::from_str("1.e").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal256::from_str("1.2e3").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_from_str_errors_for_more_than_36_fractional_digits() {
        match Decimal256::from_str("7.1234567890123456789012345678901234567").unwrap_err() {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "Cannot parse more than 36 fractional digits")
            }
            e => panic!("Unexpected error: {:?}", e),
        }

        // No special rules for trailing zeros. This could be changed but adds gas cost for the happy path.
        match Decimal256::from_str("7.1230000000000000000000000000000000000").unwrap_err() {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "Cannot parse more than 36 fractional digits")
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_from_str_errors_for_invalid_number_of_dots() {
        match Decimal256::from_str("1.2.3").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Unexpected number of dots"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal256::from_str("1.2.3.4").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Unexpected number of dots"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_from_str_errors_for_more_than_max_value() {
        // Integer
        match Decimal256::from_str("115792089237316195423570985008687907853270").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Value too big"),
            e => panic!("Unexpected error: {:?}", e),
        }

        // Decimal
        match Decimal256::from_str("115792089237316195423570985008687907853270.0").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Value too big"),
            e => panic!("Unexpected error: {:?}", e),
        }
        match Decimal256::from_str(
            "115792089237316195423570985008687907853269.984665640564039457584007913129639936",
        )
        .unwrap_err()
        {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Value too big"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_is_zero_works() {
        assert!(Decimal256::zero().is_zero());
        assert!(Decimal256::percent(0).is_zero());
        assert!(Decimal256::permille(0).is_zero());

        assert!(!Decimal256::one().is_zero());
        assert!(!Decimal256::percent(123).is_zero());
        assert!(!Decimal256::permille(1234).is_zero());
    }

    #[test]
    fn decimal_inv_works() {
        // d = 0
        assert_eq!(Decimal256::zero().inv(), None);

        // d == 1
        assert_eq!(Decimal256::one().inv(), Some(Decimal256::one()));

        // d > 1 exact
        assert_eq!(
            Decimal256::from_str("2").unwrap().inv(),
            Some(Decimal256::from_str("0.5").unwrap())
        );
        assert_eq!(
            Decimal256::from_str("20").unwrap().inv(),
            Some(Decimal256::from_str("0.05").unwrap())
        );
        assert_eq!(
            Decimal256::from_str("200").unwrap().inv(),
            Some(Decimal256::from_str("0.005").unwrap())
        );
        assert_eq!(
            Decimal256::from_str("2000").unwrap().inv(),
            Some(Decimal256::from_str("0.0005").unwrap())
        );

        // d > 1 rounded
        assert_eq!(
            Decimal256::from_str("3").unwrap().inv(),
            Some(Decimal256::from_str("0.333333333333333333333333333333333333").unwrap())
        );
        assert_eq!(
            Decimal256::from_str("6").unwrap().inv(),
            Some(Decimal256::from_str("0.166666666666666666666666666666666666").unwrap())
        );

        // d < 1 exact
        assert_eq!(
            Decimal256::from_str("0.5").unwrap().inv(),
            Some(Decimal256::from_str("2").unwrap())
        );
        assert_eq!(
            Decimal256::from_str("0.05").unwrap().inv(),
            Some(Decimal256::from_str("20").unwrap())
        );
        assert_eq!(
            Decimal256::from_str("0.005").unwrap().inv(),
            Some(Decimal256::from_str("200").unwrap())
        );
        assert_eq!(
            Decimal256::from_str("0.0005").unwrap().inv(),
            Some(Decimal256::from_str("2000").unwrap())
        );
    }

    #[test]
    fn decimal_add() {
        let value = Decimal256::one() + Decimal256::percent(50); // 1.5
        assert_eq!(
            value.0,
            Decimal256::decimal_fractional() * Uint256::from(3u8) / Uint256::from(2u8)
        );
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn decimal_add_overflow_panics() {
        let _value = Decimal256::MAX + Decimal256::percent(50);
    }

    #[test]
    fn decimal_sub() {
        let value = Decimal256::one() - Decimal256::percent(50); // 0.5
        assert_eq!(
            value.0,
            Decimal256::decimal_fractional() / Uint256::from(2u8)
        );
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn decimal_sub_overflow_panics() {
        let _value = Decimal256::zero() - Decimal256::percent(50);
    }

    #[test]
    // in this test the Decimal256 is on the right
    fn uint128_decimal_multiply() {
        // a*b
        let left = Uint256::from(300u128);
        let right = Decimal256::one() + Decimal256::percent(50); // 1.5
        assert_eq!(left * right, Uint256::from(450u32));

        // a*0
        let left = Uint256::from(300u128);
        let right = Decimal256::zero();
        assert_eq!(left * right, Uint256::from(0u128));

        // 0*a
        let left = Uint256::from(0u128);
        let right = Decimal256::one() + Decimal256::percent(50); // 1.5
        assert_eq!(left * right, Uint256::from(0u128));
    }

    #[test]
    // in this test the Decimal256 is on the left
    fn decimal_uint128_multiply() {
        // a*b
        let left = Decimal256::one() + Decimal256::percent(50); // 1.5
        let right = Uint256::from(300u128);
        assert_eq!(left * right, Uint256::from(450u128));

        // 0*a
        let left = Decimal256::zero();
        let right = Uint256::from(300u128);
        assert_eq!(left * right, Uint256::from(0u128));

        // a*0
        let left = Decimal256::one() + Decimal256::percent(50); // 1.5
        let right = Uint256::from(0u128);
        assert_eq!(left * right, Uint256::from(0u128));
    }

    #[test]
    fn decimal_uint128_division() {
        // a/b
        let left = Decimal256::percent(150); // 1.5
        let right = Uint256::from(3u128);
        assert_eq!(left / right, Decimal256::percent(50));

        // 0/a
        let left = Decimal256::zero();
        let right = Uint256::from(300u128);
        assert_eq!(left / right, Decimal256::zero());
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn decimal_uint128_divide_by_zero() {
        let left = Decimal256::percent(150); // 1.5
        let right = Uint256::from(0u128);
        let _result = left / right;
    }

    #[test]
    fn decimal_uint128_div_assign() {
        // a/b
        let mut dec = Decimal256::percent(150); // 1.5
        dec /= Uint256::from(3u128);
        assert_eq!(dec, Decimal256::percent(50));

        // 0/a
        let mut dec = Decimal256::zero();
        dec /= Uint256::from(300u128);
        assert_eq!(dec, Decimal256::zero());
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn decimal_uint128_div_assign_by_zero() {
        // a/0
        let mut dec = Decimal256::percent(50);
        dec /= Uint256::from(0u128);
    }

    #[test]
    fn decimal_uint128_sqrt() {
        assert_eq!(Decimal256::percent(900).sqrt(), Decimal256::percent(300));

        assert!(Decimal256::percent(316) < Decimal256::percent(1000).sqrt());
        assert!(Decimal256::percent(1000).sqrt() < Decimal256::percent(317));
    }

    /// sqrt(2) is an irrational number, i.e. all 36 decimal places should be used.
    #[test]
    fn decimal_uint128_sqrt_is_precise() {
        assert_eq!(
            Decimal256::from_str("2").unwrap().sqrt(),
            Decimal256::from_str("1.414213562373095048801688724209698078").unwrap() // https://www.wolframalpha.com/input/?i=sqrt%282%29
        );
    }

    #[test]
    fn decimal_uint128_sqrt_does_not_overflow() {
        assert_eq!(
            Decimal256::from_str("40000000000000000000000000000000000000000")
                .unwrap()
                .sqrt(),
            Decimal256::from_str("200000000000000000000").unwrap()
        );
    }

    #[test]
    fn decimal_uint128_sqrt_intermediate_precision_used() {
        assert_eq!(
            Decimal256::from_str("400000000001").unwrap().sqrt(),
            // The last four digits (8380) are truncated below due to the algorithm
            // we use. Larger numbers will cause less precision.
            // https://www.wolframalpha.com/input/?i=sqrt%28400000000001%29
            Decimal256::from_str("632455.532034466435814820309613659029430000").unwrap()
        );
    }

    #[test]
    fn decimal_to_string() {
        // Integers
        assert_eq!(Decimal256::zero().to_string(), "0");
        assert_eq!(Decimal256::one().to_string(), "1");
        assert_eq!(Decimal256::percent(500).to_string(), "5");

        // Decimals
        assert_eq!(Decimal256::percent(125).to_string(), "1.25");
        assert_eq!(Decimal256::percent(42638).to_string(), "426.38");
        assert_eq!(Decimal256::percent(3).to_string(), "0.03");
        assert_eq!(Decimal256::permille(987).to_string(), "0.987");

        assert_eq!(
            Decimal256(Uint256::from(1u128)).to_string(),
            "0.000000000000000000000000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(10u128)).to_string(),
            "0.00000000000000000000000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(100u128)).to_string(),
            "0.0000000000000000000000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(1000u128)).to_string(),
            "0.000000000000000000000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(10000u128)).to_string(),
            "0.00000000000000000000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(100000u128)).to_string(),
            "0.0000000000000000000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(1000000u128)).to_string(),
            "0.000000000000000000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(10000000u128)).to_string(),
            "0.00000000000000000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(100000000u128)).to_string(),
            "0.0000000000000000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(1000000000u128)).to_string(),
            "0.000000000000000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(10000000000u128)).to_string(),
            "0.00000000000000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(100000000000u128)).to_string(),
            "0.0000000000000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(10000000000000u128)).to_string(),
            "0.00000000000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(100000000000000u128)).to_string(),
            "0.0000000000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(1000000000000000u128)).to_string(),
            "0.000000000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(10000000000000000u128)).to_string(),
            "0.00000000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(100000000000000000u128)).to_string(),
            "0.0000000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from_str("1000000000000000000").unwrap()).to_string(),
            "0.000000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from_str("10000000000000000000").unwrap()).to_string(),
            "0.00000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from_str("100000000000000000000").unwrap()).to_string(),
            "0.0000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from_str("1000000000000000000000").unwrap()).to_string(),
            "0.000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from_str("10000000000000000000000").unwrap()).to_string(),
            "0.00000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from_str("100000000000000000000000").unwrap()).to_string(),
            "0.0000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from_str("1000000000000000000000000").unwrap()).to_string(),
            "0.000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from_str("10000000000000000000000000").unwrap()).to_string(),
            "0.00000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from_str("100000000000000000000000000").unwrap()).to_string(),
            "0.0000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from_str("1000000000000000000000000000").unwrap()).to_string(),
            "0.000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from_str("10000000000000000000000000000").unwrap()).to_string(),
            "0.00000001"
        );
        assert_eq!(
            Decimal256(Uint256::from_str("100000000000000000000000000000").unwrap()).to_string(),
            "0.0000001"
        );
        assert_eq!(
            Decimal256(Uint256::from_str("10000000000000000000000000000000").unwrap()).to_string(),
            "0.00001"
        );
        assert_eq!(
            Decimal256(Uint256::from_str("100000000000000000000000000000000").unwrap()).to_string(),
            "0.0001"
        );
        assert_eq!(
            Decimal256(Uint256::from_str("1000000000000000000000000000000000").unwrap())
                .to_string(),
            "0.001"
        );
        assert_eq!(
            Decimal256(Uint256::from_str("10000000000000000000000000000000000").unwrap())
                .to_string(),
            "0.01"
        );
        assert_eq!(
            Decimal256(Uint256::from_str("100000000000000000000000000000000000").unwrap())
                .to_string(),
            "0.1"
        );
    }

    #[test]
    fn decimal_serialize() {
        assert_eq!(to_vec(&Decimal256::zero()).unwrap(), br#""0""#);
        assert_eq!(to_vec(&Decimal256::one()).unwrap(), br#""1""#);
        assert_eq!(to_vec(&Decimal256::percent(8)).unwrap(), br#""0.08""#);
        assert_eq!(to_vec(&Decimal256::percent(87)).unwrap(), br#""0.87""#);
        assert_eq!(to_vec(&Decimal256::percent(876)).unwrap(), br#""8.76""#);
        assert_eq!(to_vec(&Decimal256::percent(8765)).unwrap(), br#""87.65""#);
    }

    #[test]
    fn decimal_deserialize() {
        assert_eq!(
            from_slice::<Decimal256>(br#""0""#).unwrap(),
            Decimal256::zero()
        );
        assert_eq!(
            from_slice::<Decimal256>(br#""1""#).unwrap(),
            Decimal256::one()
        );
        assert_eq!(
            from_slice::<Decimal256>(br#""000""#).unwrap(),
            Decimal256::zero()
        );
        assert_eq!(
            from_slice::<Decimal256>(br#""001""#).unwrap(),
            Decimal256::one()
        );

        assert_eq!(
            from_slice::<Decimal256>(br#""0.08""#).unwrap(),
            Decimal256::percent(8)
        );
        assert_eq!(
            from_slice::<Decimal256>(br#""0.87""#).unwrap(),
            Decimal256::percent(87)
        );
        assert_eq!(
            from_slice::<Decimal256>(br#""8.76""#).unwrap(),
            Decimal256::percent(876)
        );
        assert_eq!(
            from_slice::<Decimal256>(br#""87.65""#).unwrap(),
            Decimal256::percent(8765)
        );
    }
}
