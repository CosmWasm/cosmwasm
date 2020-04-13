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
// (Note: we can add derives if we want to include this in serialization
// but then we have to think about normalization a bit more)
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

    /// normalize Wallet (sorted by denom, no 0 elements, no duplicate denoms)
    pub fn normalize(&mut self) {
        // drop 0's
        self.0.retain(|c| c.amount.u128() != 0);
        // sort
        self.0.sort_unstable_by(|a, b| a.denom.cmp(&b.denom));

        // find all i where (self[i-1].denom == self[i].denom).
        let mut dups: Vec<usize> = self
            .0
            .iter()
            .enumerate()
            .filter_map(|(i, c)| {
                if i != 0 && c.denom == self.0[i - 1].denom {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();
        dups.reverse();

        // we go through the dups in reverse order (to avoid shifting indexes of other ones)
        for dup in dups {
            let add = self.0[dup].amount;
            self.0[dup - 1].amount += add;
            self.0.remove(dup);
        }
    }

    fn find(&self, denom: &str) -> Option<(usize, &Coin)> {
        self.0.iter().enumerate().find(|(_i, c)| c.denom == denom)
    }

    /// insert_pos should only be called when denom is not in the Wallet.
    /// it returns the position where denom should be inserted at (via splice).
    /// It returns None if this should be appended
    fn insert_pos(&self, denom: &str) -> Option<usize> {
        self.0.iter().position(|c| c.denom.as_str() >= denom)
    }
}

impl ops::Add<Coin> for Wallet {
    type Output = Self;

    fn add(mut self, other: Coin) -> Self {
        match self.find(&other.denom) {
            Some((i, c)) => {
                self.0[i].amount = c.amount + other.amount;
            }
            // place this in proper sorted order
            None => match self.insert_pos(&other.denom) {
                Some(idx) => self.0.insert(idx, other),
                None => self.0.push(other),
            },
        };
        self
    }
}

impl ops::Sub<Coin> for Wallet {
    type Output = Result<Self, Error>;

    fn sub(mut self, other: Coin) -> Result<Self, Error> {
        match self.find(&other.denom) {
            Some((i, c)) => {
                let remainder = (c.amount - other.amount)?;
                if remainder.u128() == 0 {
                    self.0.remove(i);
                } else {
                    self.0[i].amount = remainder;
                }
            }
            // error if no tokens
            None => return underflow(0, other.amount.u128()),
        };
        Ok(self)
    }
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
        let wallet = Wallet(vec![coin(555, "BTC"), coin(12345, "ETH")]);

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
        let wallet = Wallet(vec![coin(555, "BTC"), coin(12345, "ETH")]);

        // add an existing coin
        let more_eth = wallet.clone() + coin(54321, "ETH");
        assert_eq!(more_eth, Wallet(vec![coin(555, "BTC"), coin(66666, "ETH")]));

        // add an new coin
        let add_atom = wallet.clone() + coin(777, "ATOM");
        assert_eq!(
            add_atom,
            Wallet(vec![
                coin(777, "ATOM"),
                coin(555, "BTC"),
                coin(12345, "ETH"),
            ])
        );
    }

    #[test]
    fn wallet_subtract_works() {
        let wallet = Wallet(vec![coin(555, "BTC"), coin(12345, "ETH")]);

        // subtract less than we have
        let less_eth = (wallet.clone() - coin(2345, "ETH")).unwrap();
        assert_eq!(less_eth, Wallet(vec![coin(555, "BTC"), coin(10000, "ETH")]));

        // subtract all of one coin (and remove with 0 amount)
        let no_btc = (wallet.clone() - coin(555, "BTC")).unwrap();
        assert_eq!(no_btc, Wallet(vec![coin(12345, "ETH")]));

        // subtract more than we have
        let underflow = wallet.clone() - coin(666, "BTC");
        assert!(underflow.is_err());

        // subtract non-existent denom
        let missing = wallet.clone() - coin(1, "ATOM");
        assert!(missing.is_err());
    }

    #[test]
    fn normalize_wallet() {
        // remove 0 value items and sort
        let mut wallet = Wallet(vec![coin(123, "ETH"), coin(0, "BTC"), coin(8990, "ATOM")]);
        wallet.normalize();
        assert_eq!(wallet, Wallet(vec![coin(8990, "ATOM"), coin(123, "ETH")]));

        // merge duplicate entries of same denom
        let mut wallet = Wallet(vec![
            coin(123, "ETH"),
            coin(789, "BTC"),
            coin(321, "ETH"),
            coin(11, "BTC"),
        ]);
        wallet.normalize();
        assert_eq!(wallet, Wallet(vec![coin(800, "BTC"), coin(444, "ETH")]));
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

    #[test]
    #[should_panic]
    fn uint128_math_overflow_panics() {
        // almost_max is 2^128 - 10
        let almost_max = Uint128(340282366920938463463374607431768211446);
        let _ = almost_max + Uint128(12);
    }
}
