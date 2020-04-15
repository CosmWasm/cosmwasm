use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use std::convert::TryFrom;
use std::{fmt, ops};

use crate::dyn_contract_err;
use crate::errors::Error;

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
    type Error = Error;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val.parse::<u128>() {
            Ok(u) => Ok(Uint128(u)),
            Err(e) => dyn_contract_err(format!("Parsing coin: {}", e)),
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

impl ops::Sub for Uint128 {
    type Output = Uint128;

    fn sub(self, other: Self) -> Self {
        Uint128(self.u128() - other.u128())
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

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
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

        let a: Result<Uint128, Error> = "1.23".try_into();
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

        assert_eq!(a + b, Uint128(35801));
        assert_eq!(b - a, Uint128(11111));
    }

    #[test]
    #[should_panic]
    fn uint128_math_prevents_overflow() {
        let a = Uint128(12345);
        let b = Uint128(23456);
        // this will underflow, should panic
        let _ = a - b;
    }
}
