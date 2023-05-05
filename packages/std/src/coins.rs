use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;
use std::{any::type_name, collections::btree_map};

use crate::{Coin, StdError, StdResult, Uint128};

/// A collection of coins, similar to Cosmos SDK's `sdk.Coins` struct.
///
/// Differently from `sdk.Coins`, which is a vector of `sdk.Coin`, here we
/// implement Coins as a BTreeMap that maps from coin denoms to amounts.
/// This has a number of advantages:
///
/// - coins are naturally sorted alphabetically by denom
/// - duplicate denoms are automatically removed
/// - cheaper for searching/inserting/deleting: O(log(n)) compared to O(n)
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Coins(BTreeMap<String, Uint128>);

// Casting a Vec<Coin> to Coins.
// The Vec can be out of order, but must not contain duplicate denoms or zero amounts.
impl TryFrom<Vec<Coin>> for Coins {
    type Error = StdError;

    fn try_from(vec: Vec<Coin>) -> StdResult<Self> {
        let vec_len = vec.len();

        let map = vec
            .into_iter()
            .filter(|coin| !coin.amount.is_zero())
            .map(|coin| (coin.denom, coin.amount))
            .collect::<BTreeMap<_, _>>();

        // the map having a different length from the vec means the vec must either
        // 1) contain duplicate denoms, or 2) contain zero amounts
        if map.len() != vec_len {
            return Err(StdError::parse_err(
                type_name::<Self>(),
                "duplicate denoms or zero amount",
            ));
        }

        Ok(Self(map))
    }
}

impl TryFrom<&[Coin]> for Coins {
    type Error = StdError;

    fn try_from(slice: &[Coin]) -> StdResult<Self> {
        slice.to_vec().try_into()
    }
}

impl<const N: usize> TryFrom<[Coin; N]> for Coins {
    type Error = StdError;

    fn try_from(slice: [Coin; N]) -> StdResult<Self> {
        slice.to_vec().try_into()
    }
}

impl TryFrom<Coin> for Coins {
    type Error = StdError;

    fn try_from(coin: Coin) -> StdResult<Self> {
        let mut coins = Coins::default();
        coins.add(coin)?;
        Ok(coins)
    }
}

impl From<Coins> for Vec<Coin> {
    fn from(value: Coins) -> Self {
        value.into_vec()
    }
}

impl FromStr for Coins {
    type Err = StdError;

    fn from_str(s: &str) -> StdResult<Self> {
        // TODO: use FromStr impl for Coin once it's merged

        // Parse a string into a `Coin`.
        //
        // Parsing the string with regex doesn't work, because the resulting
        // wasm binary would be too big from including the `regex` library.
        //
        // We opt for the following solution: enumerate characters in the string,
        // and break before the first non-number character. Split the string at
        // that index.
        //
        // This assumes the denom never starts with a number, which is the case:
        // https://github.com/cosmos/cosmos-sdk/blob/v0.46.0/types/coin.go#L854-L856
        let parse_coin_str = |s: &str| -> StdResult<Coin> {
            for (i, c) in s.chars().enumerate() {
                if !c.is_ascii_digit() {
                    let amount = Uint128::from_str(&s[..i])?;
                    let denom = String::from(&s[i..]);
                    return Ok(Coin { amount, denom });
                }
            }

            Err(StdError::parse_err(
                type_name::<Coin>(),
                format!("invalid coin string: {s}"),
            ))
        };

        s.split(',')
            .map(parse_coin_str)
            .collect::<StdResult<Vec<_>>>()?
            .try_into()
    }
}

impl fmt::Display for Coins {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self
            .0
            .iter()
            .map(|(denom, amount)| format!("{amount}{denom}"))
            .collect::<Vec<_>>()
            .join(",");
        write!(f, "{s}")
    }
}

impl PartialEq<Coin> for Coins {
    fn eq(&self, other: &Coin) -> bool {
        self.0.len() == 1 && self.amount_of(&other.denom) == other.amount
    }
}

impl Coins {
    /// Cast to Vec<Coin>, while NOT consuming the original object
    pub fn to_vec(&self) -> Vec<Coin> {
        self.0
            .iter()
            .map(|(denom, amount)| Coin {
                denom: denom.clone(),
                amount: *amount,
            })
            .collect()
    }

    /// Cast to Vec<Coin>, consuming the original object
    pub fn into_vec(self) -> Vec<Coin> {
        self.0
            .into_iter()
            .map(|(denom, amount)| Coin { denom, amount })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the denoms as a vector of strings.
    /// The vector is guaranteed to not contain duplicates and sorted alphabetically.
    pub fn denoms(&self) -> Vec<String> {
        self.0.keys().cloned().collect()
    }

    /// Returns the amount of the given denom or zero if the denom is not present.
    pub fn amount_of(&self, denom: &str) -> Uint128 {
        self.0.get(denom).copied().unwrap_or_else(Uint128::zero)
    }

    /// Returns the amount of the given denom if and only if this collection contains only
    /// the given denom. Otherwise `None` is returned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmwasm_std::{Coin, Coins, coin};
    ///
    /// let coins: Coins = coin(100, "uatom").try_into().unwrap();
    /// assert_eq!(coins.contains_only("uatom").unwrap().u128(), 100);
    /// assert_eq!(coins.contains_only("uluna"), None);
    /// ```
    ///
    /// ```rust
    /// use cosmwasm_std::{Coin, Coins, coin};
    ///
    /// let coins: Coins = [coin(100, "uatom"), coin(200, "uusd")].try_into().unwrap();
    /// assert_eq!(coins.contains_only("uatom"), None);
    /// ```
    pub fn contains_only(&self, denom: &str) -> Option<Uint128> {
        if self.len() == 1 {
            self.0.get(denom).copied()
        } else {
            None
        }
    }

    /// Adds the given coin to the collection.
    /// This errors in case of overflow.
    pub fn add(&mut self, coin: Coin) -> StdResult<()> {
        if coin.amount.is_zero() {
            return Ok(());
        }

        let amount = self.0.entry(coin.denom).or_insert_with(Uint128::zero);
        *amount = amount.checked_add(coin.amount)?;
        Ok(())
    }

    /// Adds the given coins to the collection.
    /// This takes anything that yields `(denom, amount)` tuples when iterated over.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmwasm_std::{Coin, Coins, coin};
    ///
    /// let mut coins = Coins::default();
    /// let new_coins: Coins = coin(123u128, "ucosm").try_into()?;
    /// coins.extend(new_coins.clone())?;
    /// assert_eq!(coins, new_coins);
    /// # cosmwasm_std::StdResult::Ok(())
    /// ```
    pub fn extend<C>(&mut self, others: C) -> StdResult<()>
    where
        C: IntoIterator<Item = (String, Uint128)>,
    {
        for (denom, amount) in others {
            self.add(Coin { denom, amount })?;
        }
        Ok(())
    }
}

impl IntoIterator for Coins {
    type Item = (String, Uint128);
    // TODO: do we want to wrap the iterator type with our own to avoid exposing BTreeMap?
    // also: for the owned version we could return Coins instead of (String, Uint128),
    // but not for the borrowed version, so it would feel inconsistent
    type IntoIter = btree_map::IntoIter<String, Uint128>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Coins {
    type Item = (&'a String, &'a Uint128);
    type IntoIter = btree_map::Iter<'a, String, Uint128>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coin;

    /// Sort a Vec<Coin> by denom alphabetically
    fn sort_by_denom(vec: &mut [Coin]) {
        vec.sort_by(|a, b| a.denom.cmp(&b.denom));
    }

    /// Returns a mockup Vec<Coin>. In this example, the coins are not in order
    fn mock_vec() -> Vec<Coin> {
        vec![
            coin(12345, "uatom"),
            coin(69420, "ibc/1234ABCD"),
            coin(88888, "factory/osmo1234abcd/subdenom"),
        ]
    }

    /// Return a mockup Coins that contains the same coins as in `mock_vec`
    fn mock_coins() -> Coins {
        let mut coins = Coins::default();
        for coin in mock_vec() {
            coins.add(coin).unwrap();
        }
        coins
    }

    #[test]
    fn casting_vec() {
        let mut vec = mock_vec();
        let coins = mock_coins();

        // &[Coin] --> Coins
        assert_eq!(Coins::try_from(vec.as_slice()).unwrap(), coins);
        // Vec<Coin> --> Coins
        assert_eq!(Coins::try_from(vec.clone()).unwrap(), coins);

        sort_by_denom(&mut vec);

        // &Coins --> Vec<Coins>
        // NOTE: the returned vec should be sorted
        assert_eq!(coins.to_vec(), vec);
        // Coins --> Vec<Coins>
        // NOTE: the returned vec should be sorted
        assert_eq!(coins.into_vec(), vec);
    }

    #[test]
    fn casting_str() {
        // not in order
        let s1 = "88888factory/osmo1234abcd/subdenom,12345uatom,69420ibc/1234ABCD";
        // in order
        let s2 = "88888factory/osmo1234abcd/subdenom,69420ibc/1234ABCD,12345uatom";

        let coins = mock_coins();

        // &str --> Coins
        // NOTE: should generate the same Coins, regardless of input order
        assert_eq!(Coins::from_str(s1).unwrap(), coins);
        assert_eq!(Coins::from_str(s2).unwrap(), coins);

        // Coins --> String
        // NOTE: the generated string should be sorted
        assert_eq!(coins.to_string(), s2);
    }

    #[test]
    fn handling_duplicates() {
        // create a Vec<Coin> that contains duplicate denoms
        let mut vec = mock_vec();
        vec.push(coin(67890, "uatom"));

        let err = Coins::try_from(vec).unwrap_err();
        assert!(err.to_string().contains("duplicate denoms"));
    }

    #[test]
    fn handling_zero_amount() {
        // create a Vec<Coin> that contains zero amounts
        let mut vec = mock_vec();
        vec[0].amount = Uint128::zero();

        let err = Coins::try_from(vec).unwrap_err();
        assert!(err.to_string().contains("zero amount"));

        // adding a coin with zero amount should not be added
        let mut coins = Coins::default();
        coins.add(coin(0, "uusd")).unwrap();
        assert!(coins.is_empty());
    }

    #[test]
    fn length() {
        let coins = Coins::default();
        assert_eq!(coins.len(), 0);
        assert!(coins.is_empty());

        let coins = mock_coins();
        assert_eq!(coins.len(), 3);
        assert!(!coins.is_empty());
    }

    #[test]
    fn add_coin() {
        let mut coins = mock_coins();
        coins.add(coin(12345, "uatom")).unwrap();

        assert_eq!(coins.len(), 3);
        assert_eq!(coins.amount_of("uatom").u128(), 24690);

        coins.add(coin(123, "uusd")).unwrap();
        assert_eq!(coins.len(), 4);
    }

    #[test]
    fn extend_coins() {
        let mut coins: Coins = coin(12345, "uatom").try_into().unwrap();

        coins.extend(mock_coins()).unwrap();
        assert_eq!(coins.len(), 3);
        assert_eq!(coins.amount_of("uatom").u128(), 24690);

        coins
            .extend([("uusd".to_string(), Uint128::new(123u128))])
            .unwrap();
        assert_eq!(coins.len(), 4);
        assert_eq!(coins.amount_of("uusd").u128(), 123)
    }

    #[test]
    fn equality() {
        let coin = coin(54321, "uatom");
        let coins = Coins::try_from(coin.clone()).unwrap();

        assert_eq!(coins, coin);
    }
}
