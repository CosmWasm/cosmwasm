use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use std::convert::{TryFrom, TryInto};
use std::{fmt, ops};

use crate::errors::{generic_err, underflow, StdError, StdResult};

/// A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, JsonSchema)]
pub struct Decimal(Uint128);

const DECIMAL_FRACTIONAL: Uint128 = Uint128(1_000_000_000_000_000_000);

impl Decimal {
    /// Create a 1.0 Decimal
    pub const fn one() -> Decimal {
        Decimal(DECIMAL_FRACTIONAL)
    }

    /// Create a 0.0 Decimal
    pub const fn zero() -> Decimal {
        Decimal(Uint128(0))
    }

    /// Convert x% into Decimal
    pub fn percent(x: u64) -> Decimal {
        Decimal(Uint128((x as u128) * 10_000_000_000_000_000))
    }

    /// Convert permille (x/1000) into Decimal
    pub fn permille(x: u64) -> Decimal {
        Decimal(Uint128((x as u128) * 1_000_000_000_000_000))
    }

    pub fn is_zero(&self) -> bool {
        self.0.u128() == 0
    }
}

impl ops::Add for Decimal {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Decimal(self.0 + other.0)
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

    pub fn u128(&self) -> u128 {
        self.0
    }
}

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

impl TryFrom<&str> for Uint128 {
    type Error = StdError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val.parse::<u128>() {
            Ok(u) => Ok(Uint128(u)),
            Err(e) => Err(generic_err(format!("Parsing coin: {}", e))),
        }
    }
}

impl Into<String> for Uint128 {
    fn into(self) -> String {
        self.0.to_string()
    }
}

impl Into<u128> for Uint128 {
    fn into(self) -> u128 {
        self.u128()
    }
}

impl fmt::Display for Uint128 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ops::Add for Uint128 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Uint128(self.u128() + other.u128())
    }
}

impl ops::AddAssign for Uint128 {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.u128();
    }
}

impl ops::Sub for Uint128 {
    type Output = StdResult<Self>;

    fn sub(self, other: Self) -> StdResult<Self> {
        let (min, sub) = (self.u128(), other.u128());
        if sub > min {
            Err(underflow(min, sub))
        } else {
            Ok(Uint128(min - sub))
        }
    }
}

/// Both d*u and u*d with d: Decimal and u: Uint128 returns an Uint128. There is no
/// specific reason for this decision other than the initial use cases we have. If you
/// need a Decimal result for the same calculation, use Decimal(d*u) or Decimal(u*d).
impl ops::Mul<Decimal> for Uint128 {
    type Output = Self;

    fn mul(self, rhs: Decimal) -> Self::Output {
        // 0*a and b*0 is always 0
        if self.0 == 0 || rhs.is_zero() {
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
    /// returns self * num / denom
    pub fn multiply_ratio(&self, num: Uint128, denom: Uint128) -> Uint128 {
        if denom.0 == 0 {
            panic!("Denominator must not be zero");
        }
        // TODO: minimize rounding that takes place (using gcd algorithm)
        let val = self.u128() * num.u128() / denom.u128();
        Uint128::from(val)
    }

    /// Returns the ratio (self / denom) as Decimal fixed-point
    pub fn calc_ratio(&self, denom: Uint128) -> Decimal {
        // special case: 0/0 = 1.0
        if self.0 == 0 && denom.0 == 0 {
            return Decimal::one();
        }
        // otherwise, panic on 0 (or how to handle 1/0)?

        let places: u128 = DECIMAL_FRACTIONAL.into();
        // TODO: better algorithm with less rounding potential
        let val: u128 = self.u128() * places / denom.u128();
        // TODO: better error handling
        Decimal(val.try_into().unwrap())
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::errors::{StdError, StdResult};
    use crate::{from_slice, to_vec};
    use std::convert::TryInto;

    #[test]
    fn decimal_one() {
        let value = Decimal::one();
        assert_eq!(value.0, DECIMAL_FRACTIONAL);
    }

    #[test]
    fn decimal_zero() {
        let value = Decimal::zero();
        assert_eq!(value.0, Uint128::zero());
    }

    #[test]
    fn decimal_percent() {
        let value = Decimal::percent(50);
        assert_eq!(value.0.u128(), DECIMAL_FRACTIONAL.u128() / 2);
    }

    #[test]
    fn decimal_permille() {
        let value = Decimal::permille(125);
        assert_eq!(value.0.u128(), DECIMAL_FRACTIONAL.u128() / 8);
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
        assert_eq!(value.0.u128(), DECIMAL_FRACTIONAL.u128() * 3 / 2);
    }

    #[test]
    fn to_and_from_uint128() {
        let a: Uint128 = 12345u64.into();
        assert_eq!(12345, a.u128());
        assert_eq!("12345", a.to_string());

        let a: Uint128 = "34567".try_into().unwrap();
        assert_eq!(34567, a.u128());
        assert_eq!("34567", a.to_string());

        let a: StdResult<Uint128> = "1.23".try_into();
        assert!(a.is_err());
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
    fn uint128_math() {
        let a = Uint128(12345);
        let b = Uint128(23456);

        // test + and - for valid values
        assert_eq!(a + b, Uint128(35801));
        assert_eq!((b - a).unwrap(), Uint128(11111));

        // test +=
        let mut c = Uint128(300000);
        c += b;
        assert_eq!(c, Uint128(323456));

        // error result on underflow (- would produce negative result)
        let underflow = a - b;
        match underflow {
            Ok(_) => panic!("should error"),
            Err(StdError::Underflow {
                minuend,
                subtrahend,
                ..
            }) => assert_eq!((minuend, subtrahend), (a.to_string(), b.to_string())),
            _ => panic!("expected underflow error"),
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
}
