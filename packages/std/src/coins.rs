use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use std::convert::TryFrom;
use std::{fmt, ops};

use crate::dyn_contract_err;
use crate::errors::Error;

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct Coin {
    pub denom: String,
    pub amount: BigInt,
}

// coin is a shortcut constructor for a set of one denomination of coins
pub fn coin(amount: u128, denom: &str) -> Vec<Coin> {
    vec![one_coin(amount, denom)]
}

// one_coin makes a singular coin
pub fn one_coin(amount: u128, denom: &str) -> Coin {
    Coin {
        amount: BigInt(amount),
        denom: denom.to_string(),
    }
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
pub struct BigInt(#[schemars(with = "String")] pub u128);

impl BigInt {
    pub fn u128(&self) -> u128 {
        self.0
    }
}

impl From<u128> for BigInt {
    fn from(val: u128) -> Self {
        BigInt(val)
    }
}

impl TryFrom<&str> for BigInt {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val.parse::<u128>() {
            Ok(u) => Ok(BigInt(u)),
            Err(e) => dyn_contract_err(format!("Parsing coin: {}", e)),
        }
    }
}

impl Into<String> for BigInt {
    fn into(self) -> String {
        self.0.to_string()
    }
}

impl Into<u128> for BigInt {
    fn into(self) -> u128 {
        self.u128()
    }
}

impl fmt::Display for BigInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ops::Add for BigInt {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        BigInt(self.u128() + other.u128())
    }
}

impl ops::Sub for BigInt {
    type Output = BigInt;

    fn sub(self, other: Self) -> Self {
        BigInt(self.u128() - other.u128())
    }
}

/// Serializes as a base64 string
impl Serialize for BigInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Deserializes as a base64 string
impl<'de> Deserialize<'de> for BigInt {
    fn deserialize<D>(deserializer: D) -> Result<BigInt, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(BigIntVisitor)
    }
}

struct BigIntVisitor;

impl<'de> de::Visitor<'de> for BigIntVisitor {
    type Value = BigInt;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded integer")
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v.parse::<u128>() {
            Ok(u) => Ok(BigInt(u)),
            Err(e) => Err(E::custom(format!("invalid BigInt '{}' - {}", v, e))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{from_slice, to_vec};
    use std::convert::TryInto;

    #[test]
    fn has_coins_matches() {
        let wallet = vec![one_coin(12345, "ETH"), one_coin(555, "BTC")];

        // less than same type
        assert!(has_coins(&wallet, &one_coin(777, "ETH")));
    }

    #[test]
    fn to_and_from_bigint() {
        let a: BigInt = 12345.into();
        assert_eq!(12345, a.u128());
        assert_eq!("12345", a.to_string());

        let a: BigInt = "34567".try_into().unwrap();
        assert_eq!(34567, a.u128());
        assert_eq!("34567", a.to_string());

        let a: Result<BigInt, Error> = "1.23".try_into();
        assert!(a.is_err());
    }

    #[test]
    fn bigint_json() {
        let orig = BigInt(1234567890987654321);
        let serialized = to_vec(&orig).unwrap();
        assert_eq!(serialized.as_slice(), b"\"1234567890987654321\"");
        let parsed: BigInt = from_slice(&serialized).unwrap();
        assert_eq!(parsed, orig);
    }

    #[test]
    fn bigint_compare() {
        let a = BigInt(12345);
        let b = BigInt(23456);

        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, BigInt(12345));
    }

    #[test]
    fn bigint_math() {
        let a = BigInt(12345);
        let b = BigInt(23456);

        assert_eq!(a + b, BigInt(35801));
        assert_eq!(b - a, BigInt(11111));
    }

    #[test]
    #[should_panic]
    fn bigint_math_prevents_overflow() {
        let a = BigInt(12345);
        let b = BigInt(23456);
        // this will underflow, should panic
        let _ = a - b;
    }
}
