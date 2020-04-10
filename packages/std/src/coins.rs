use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use std::convert::{TryFrom, TryInto};
use std::fmt;

use crate::dyn_contract_err;
use crate::errors::Error;

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct Coin {
    pub denom: String,
    pub amount: BigInt,
}

// coin is a shortcut constructor for a set of one denomination of coins
pub fn coin(amount: u128, denom: &str) -> Vec<Coin> {
    vec![Coin {
        amount: BigInt(amount),
        denom: denom.to_string(),
    }]
}

pub fn coin_str(amount: &str, denom: &str) -> Result<Vec<Coin>, Error> {
    Ok(vec![Coin {
        amount: amount.try_into()?,
        denom: denom.to_string(),
    }])
}

#[derive(Clone, Default, Debug, PartialEq, PartialOrd, JsonSchema)]
pub struct BigInt(#[schemars(with = "String")] pub u128);

impl BigInt {
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    pub fn u128(&self) -> u128 {
        self.0
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
        self.to_string()
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
