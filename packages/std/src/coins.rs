use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use std::convert::TryFrom;
use std::{fmt, ops};

use crate::errors::{dyn_contract_err, underflow, Error};

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

// Wallet wraps Vec<Coin> and provides some nice helpers. It mutates the Vec and can be
// unwrapped when done.
//
// This is meant to be used for calculations and not serialized.
// (FIXME: we can add derives if we want to include this in serialization)
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Wallet(pub Vec<Coin>);

impl Wallet {
    pub fn into_vec(self) -> Vec<Coin> {
        self.0
    }

    /// returns true if the list of coins has at least the required amount
    pub fn has(&self, required: &Coin) -> bool {
        self.0
            .iter()
            .find(|c| c.denom == required.denom)
            .map(|m| m.amount >= required.amount)
            .unwrap_or(false)
    }
}

impl ops::Add<Coin> for Wallet {
    type Output = Self;

    fn add(mut self, mut other: Coin) -> Self {
        let existing = self
            .0
            .iter()
            .enumerate()
            .find(|(_i, c)| c.denom == other.denom);
        match existing {
            Some((i, c)) => {
                other.amount += c.amount;
                self.0[i] = other;
            }
            None => self.0.push(other),
        };
        self
    }
}

// impl ops::Sub<Coin> for Wallet {
//     type Output = Result<Self, Error>;
//
//     fn sub(mut self, mut other: Coin) -> Result<Self, Error> {
//         let existing = self
//             .0
//             .iter()
//             .enumerate()
//             .find(|(_i, c)| c.denom == other.denom);
//         match existing {
//             Some((i, c)) => {
//                 other.amount += c.amount;
//                 self.0[i] = other;
//             }
//             None => self.0.push(other),
//         };
//         self
//     }
// }

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

impl ops::AddAssign for Uint128 {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.u128();
    }
}

impl ops::Sub for Uint128 {
    type Output = Result<Self, Error>;

    fn sub(self, other: Self) -> Result<Self, Error> {
        let (min, sub) = (self.u128(), other.u128());
        if sub > min {
            underflow(min, sub)
        } else {
            Ok(Uint128(min - sub))
        }
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
        deserializer.deserialize_str(BigIntVisitor)
    }
}

struct BigIntVisitor;

impl<'de> de::Visitor<'de> for BigIntVisitor {
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
    fn wallet_has_works() {
        let wallet = Wallet(vec![coin(12345, "ETH"), coin(555, "BTC")]);

        // less than same type
        assert!(wallet.has(&coin(777, "ETH")));
        // equal to same type
        assert!(wallet.has(&coin(555, "BTC")));

        // too high
        assert!(!wallet.has(&coin(12346, "ETH")));
        // wrong type
        assert!(!wallet.has(&coin(456, "ETC")));
    }

    #[test]
    fn wallet_add_works() {
        let wallet = Wallet(vec![coin(12345, "ETH"), coin(555, "BTC")]);

        // add an existing coin
        let more_eth = wallet.clone() + coin(54321, "ETH");
        assert_eq!(more_eth, Wallet(vec![coin(66666, "ETH"), coin(555, "BTC")]));

        // add an new coin
        let add_atom = wallet.clone() + coin(777, "ATOM");
        assert_eq!(
            add_atom,
            Wallet(vec![
                coin(12345, "ETH"),
                coin(555, "BTC"),
                coin(777, "ATOM")
            ])
        );
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
        assert_eq!((b - a).unwrap(), Uint128(11111));

        // error result on underflow
        let underflow = a - b;
        match underflow {
            Ok(_) => panic!("should error"),
            Err(Error::Underflow {
                minuend,
                subtrahend,
                ..
            }) => assert_eq!((minuend, subtrahend), (a.u128(), b.u128())),
            _ => panic!("expected underflow error"),
        }
    }
}
