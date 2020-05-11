use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use std::convert::{TryFrom, TryInto};
use std::{fmt, ops};

use crate::errors::{generic_err, underflow, StdError, StdResult};
use crate::query::DECIMAL_FRACTIONAL;
use crate::Decimal9;

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct Coin {
    pub denom: String,
    pub amount: Uint128,
}

impl Coin {
    pub fn new(amount: u128, denom: &str) -> Self {
        Coin {
            amount: Uint128(amount),
            denom: denom.to_string(),
        }
    }
}

// coins is a shortcut constructor for a set of one denomination of coins
pub fn coins(amount: u128, denom: &str) -> Vec<Coin> {
    vec![coin(amount, denom)]
}

// coin is a shorthand constructor for Coin
pub fn coin(amount: u128, denom: &str) -> Coin {
    Coin::new(amount, denom)
}

/// has_coins returns true if the list of coins has at least the required amount
pub fn has_coins(coins: &[Coin], required: &Coin) -> bool {
    coins
        .iter()
        .find(|c| c.denom == required.denom)
        .map(|m| m.amount >= required.amount)
        .unwrap_or(false)
}

#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd, JsonSchema)]
pub struct Uint128(#[schemars(with = "String")] pub u128);

impl Uint128 {
    pub fn u128(&self) -> u128 {
        self.0
    }
}

impl From<u128> for Uint128 {
    fn from(val: u128) -> Self {
        Uint128(val)
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

impl Uint128 {
    /// returns self * num / denom
    pub fn multiply_ratio(&self, num: Uint128, denom: Uint128) -> Uint128 {
        // TODO: minimize rounding that takes place (using gcd algorithm)
        let val = self.u128() * num.u128() / denom.u128();
        Uint128::from(val)
    }

    /// Returns the ratio (self / denom) as Decimal9 fixed-point
    pub fn calc_ratio(&self, denom: Uint128) -> Decimal9 {
        let places: u128 = DECIMAL_FRACTIONAL.into();
        // TODO: better algorithm with less rounding potential
        let val: u128 = self.u128() * places / denom.u128();
        // TODO: better error handling
        return Decimal9(val.try_into().unwrap());
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
    fn has_coins_matches() {
        let wallet = vec![coin(12345, "ETH"), coin(555, "BTC")];

        // less than same type
        assert!(has_coins(&wallet, &coin(777, "ETH")));
    }

    #[test]
    fn to_and_from_uint128() {
        let a: Uint128 = 12345.into();
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
}
