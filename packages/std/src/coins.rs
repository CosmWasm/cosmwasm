use alloc::collections::BTreeMap;
use core::fmt;
use core::str::FromStr;

use crate::prelude::*;
use crate::{Coin, CoinsError, OverflowError, OverflowOperation, StdError, StdResult, Uint256};

/// A collection of coins, similar to Cosmos SDK's `sdk.Coins` struct.
///
/// Differently from `sdk.Coins`, which is a vector of `sdk.Coin`, here we
/// implement Coins as a BTreeMap that maps from coin denoms to `Coin`.
/// This has a number of advantages:
///
/// - coins are naturally sorted alphabetically by denom
/// - duplicate denoms are automatically removed
/// - cheaper for searching/inserting/deleting: O(log(n)) compared to O(n)
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Coins(BTreeMap<String, Coin>);

/// Casting a Vec<Coin> to Coins.
/// The Vec can be out of order, but must not contain duplicate denoms.
/// If you want to sum up duplicates, create an empty instance using `Coins::default` and
/// use `Coins::add` to add your coins.
impl TryFrom<Vec<Coin>> for Coins {
    type Error = CoinsError;

    fn try_from(vec: Vec<Coin>) -> Result<Self, CoinsError> {
        let mut map = BTreeMap::new();
        for coin in vec {
            if coin.amount.is_zero() {
                continue;
            }

            // if the insertion returns a previous value, we have a duplicate denom
            if map.insert(coin.denom.clone(), coin).is_some() {
                return Err(CoinsError::DuplicateDenom);
            }
        }

        Ok(Self(map))
    }
}

impl TryFrom<&[Coin]> for Coins {
    type Error = CoinsError;

    fn try_from(slice: &[Coin]) -> Result<Self, CoinsError> {
        slice.to_vec().try_into()
    }
}

impl From<Coin> for Coins {
    fn from(value: Coin) -> Self {
        let mut coins = Coins::default();
        // this can never overflow (because there are no coins in there yet), so we can unwrap
        coins.add(value).unwrap();
        coins
    }
}

impl<const N: usize> TryFrom<[Coin; N]> for Coins {
    type Error = CoinsError;

    fn try_from(slice: [Coin; N]) -> Result<Self, CoinsError> {
        slice.to_vec().try_into()
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
        if s.is_empty() {
            return Ok(Self::default());
        }

        Ok(s.split(',')
            .map(Coin::from_str)
            .collect::<Result<Vec<_>, _>>()?
            .try_into()?)
    }
}

impl fmt::Display for Coins {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self
            .0
            .values()
            .map(|coin| coin.to_string())
            .collect::<Vec<_>>()
            .join(",");
        write!(f, "{s}")
    }
}

impl Coins {
    /// Conversion to Vec<Coin>, while NOT consuming the original object.
    ///
    /// This produces a vector of coins that is sorted alphabetically by denom with
    /// no duplicate denoms.
    pub fn to_vec(&self) -> Vec<Coin> {
        self.0.values().cloned().collect()
    }

    /// Conversion to Vec<Coin>, consuming the original object.
    ///
    /// This produces a vector of coins that is sorted alphabetically by denom with
    /// no duplicate denoms.
    pub fn into_vec(self) -> Vec<Coin> {
        self.0.into_values().collect()
    }

    /// Returns the number of different denoms in this collection.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if this collection contains no coins.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the denoms as a vector of strings.
    /// The vector is guaranteed to not contain duplicates and sorted alphabetically.
    pub fn denoms(&self) -> Vec<String> {
        self.0.keys().cloned().collect()
    }

    /// Returns the amount of the given denom or zero if the denom is not present.
    pub fn amount_of(&self, denom: &str) -> Uint256 {
        self.0
            .get(denom)
            .map(|c| c.amount)
            .unwrap_or_else(Uint256::zero)
    }

    /// Returns the amount of the given denom if and only if this collection contains only
    /// the given denom. Otherwise `None` is returned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmwasm_std::{Coin, Coins, coin, Uint256};
    ///
    /// let coins: Coins = [coin(100, "uatom")].try_into().unwrap();
    /// assert_eq!(coins.contains_only("uatom").unwrap(), Uint256::new(100));
    /// assert_eq!(coins.contains_only("uluna"), None);
    /// ```
    ///
    /// ```rust
    /// use cosmwasm_std::{Coin, Coins, coin};
    ///
    /// let coins: Coins = [coin(100, "uatom"), coin(200, "uusd")].try_into().unwrap();
    /// assert_eq!(coins.contains_only("uatom"), None);
    /// ```
    pub fn contains_only(&self, denom: &str) -> Option<Uint256> {
        if self.len() == 1 {
            self.0.get(denom).map(|c| c.amount)
        } else {
            None
        }
    }

    /// Adds the given coin to this `Coins` instance.
    /// Errors in case of overflow.
    pub fn add(&mut self, coin: Coin) -> StdResult<()> {
        if coin.amount.is_zero() {
            return Ok(());
        }

        // if the coin is not present yet, insert it, otherwise add to existing amount
        match self.0.get_mut(&coin.denom) {
            None => {
                self.0.insert(coin.denom.clone(), coin);
            }
            Some(existing) => {
                existing.amount = existing.amount.checked_add(coin.amount)?;
            }
        }
        Ok(())
    }

    /// Subtracts the given coin from this `Coins` instance.
    /// Errors in case of overflow or if the denom is not present.
    pub fn sub(&mut self, coin: Coin) -> StdResult<()> {
        match self.0.get_mut(&coin.denom) {
            Some(existing) => {
                existing.amount = existing.amount.checked_sub(coin.amount)?;
                // make sure to remove zero coin
                if existing.amount.is_zero() {
                    self.0.remove(&coin.denom);
                }
            }
            None => {
                // ignore zero subtraction
                if coin.amount.is_zero() {
                    return Ok(());
                }
                return Err(OverflowError::new(OverflowOperation::Sub).into());
            }
        }

        Ok(())
    }

    /// Returns an iterator over the coins.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cosmwasm_std::{coin, Coin, Coins, Uint256};
    /// let mut coins = Coins::default();
    /// coins.add(coin(500, "uluna")).unwrap();
    /// coins.add(coin(1000, "uatom")).unwrap();
    /// let mut iterator = coins.iter();
    ///
    /// let uatom = iterator.next().unwrap();
    /// assert_eq!(uatom.denom, "uatom");
    /// assert_eq!(uatom.amount, Uint256::new(1000));
    ///
    /// let uluna = iterator.next().unwrap();
    /// assert_eq!(uluna.denom, "uluna");
    /// assert_eq!(uluna.amount, Uint256::new(500));
    ///
    /// assert_eq!(iterator.next(), None);
    /// ```
    pub fn iter(&self) -> CoinsIter<'_> {
        CoinsIter(self.0.iter())
    }
}

impl IntoIterator for Coins {
    type Item = Coin;
    type IntoIter = CoinsIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        CoinsIntoIter(self.0.into_iter())
    }
}

impl<'a> IntoIterator for &'a Coins {
    type Item = &'a Coin;
    type IntoIter = CoinsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug)]
pub struct CoinsIntoIter(alloc::collections::btree_map::IntoIter<String, Coin>);

impl Iterator for CoinsIntoIter {
    type Item = Coin;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, coin)| coin)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Since btree_map::IntoIter implements ExactSizeIterator, this is guaranteed to return the exact length
        self.0.size_hint()
    }
}

impl DoubleEndedIterator for CoinsIntoIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(_, coin)| coin)
    }
}

impl ExactSizeIterator for CoinsIntoIter {
    fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug)]
pub struct CoinsIter<'a>(alloc::collections::btree_map::Iter<'a, String, Coin>);

impl<'a> Iterator for CoinsIter<'a> {
    type Item = &'a Coin;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, coin)| coin)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Since btree_map::Iter implements ExactSizeIterator, this is guaranteed to return the exact length
        self.0.size_hint()
    }
}

impl DoubleEndedIterator for CoinsIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(_, coin)| coin)
    }
}

impl ExactSizeIterator for CoinsIter<'_> {
    fn len(&self) -> usize {
        self.0.len()
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
    fn converting_vec() {
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
    fn converting_str() {
        // not in order
        let s1 = "88888factory/osmo1234abcd/subdenom,12345uatom,69420ibc/1234ABCD";
        // in order
        let s2 = "88888factory/osmo1234abcd/subdenom,69420ibc/1234ABCD,12345uatom";

        let invalid = "12345uatom,noamount";

        let coins = mock_coins();

        // &str --> Coins
        // NOTE: should generate the same Coins, regardless of input order
        assert_eq!(Coins::from_str(s1).unwrap(), coins);
        assert_eq!(Coins::from_str(s2).unwrap(), coins);
        assert_eq!(Coins::from_str("").unwrap(), Coins::default());

        // Coins --> String
        // NOTE: the generated string should be sorted
        assert_eq!(coins.to_string(), s2);
        assert_eq!(Coins::default().to_string(), "");
        assert_eq!(
            Coins::from_str(invalid).unwrap_err().to_string(),
            "kind: Parsing, error: Missing amount or non-digit characters in amount"
        );
    }

    #[test]
    fn handling_duplicates() {
        // create a Vec<Coin> that contains duplicate denoms
        let mut vec = mock_vec();
        vec.push(coin(67890, "uatom"));

        let err = Coins::try_from(vec).unwrap_err();
        assert_eq!(err, CoinsError::DuplicateDenom);
    }

    #[test]
    fn handling_zero_amount() {
        // create a Vec<Coin> that contains zero amounts
        let mut vec = mock_vec();
        vec[0].amount = Uint256::zero();

        let coins = Coins::try_from(vec).unwrap();
        assert_eq!(coins.len(), 2);
        assert_ne!(coins.amount_of("ibc/1234ABCD"), Uint256::zero());
        assert_ne!(
            coins.amount_of("factory/osmo1234abcd/subdenom"),
            Uint256::zero()
        );

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

        // existing denom
        coins.add(coin(12345, "uatom")).unwrap();
        assert_eq!(coins.len(), 3);
        assert_eq!(coins.amount_of("uatom"), Uint256::new(24690));

        // new denom
        coins.add(coin(123, "uusd")).unwrap();
        assert_eq!(coins.len(), 4);

        // zero amount
        coins.add(coin(0, "uusd")).unwrap();
        assert_eq!(coins.amount_of("uusd"), Uint256::new(123));

        // zero amount, new denom
        coins.add(coin(0, "utest")).unwrap();
        assert_eq!(coins.len(), 4);
    }

    #[test]
    fn sub_coins() {
        let mut coins: Coins = coin(12345, "uatom").into();

        // sub more than available
        assert!(coins.sub(coin(12346, "uatom")).is_err());

        // sub non-existent denom
        assert!(coins.sub(coin(12345, "uusd")).is_err());

        // partial sub
        coins.sub(coin(1, "uatom")).unwrap();
        assert_eq!(coins.len(), 1);
        assert_eq!(coins.amount_of("uatom"), Uint256::new(12344));

        // full sub
        coins.sub(coin(12344, "uatom")).unwrap();
        assert!(coins.is_empty());

        // sub zero, existing denom
        coins.sub(coin(0, "uusd")).unwrap();
        assert!(coins.is_empty());
        let mut coins: Coins = coin(12345, "uatom").into();

        // sub zero, non-existent denom
        coins.sub(coin(0, "uatom")).unwrap();
        assert_eq!(coins.len(), 1);
        assert_eq!(coins.amount_of("uatom"), Uint256::new(12345));
    }

    #[test]
    fn coin_to_coins() {
        // zero coin results in empty collection
        let coins: Coins = coin(0, "uusd").into();
        assert!(coins.is_empty());

        // happy path
        let coins = Coins::from(coin(12345, "uatom"));
        assert_eq!(coins.len(), 1);
        assert_eq!(coins.amount_of("uatom"), Uint256::new(12345));
    }

    #[test]
    fn exact_size_iterator() {
        let coins = mock_coins();
        let iter = coins.iter();
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.size_hint(), (3, Some(3)));

        let iter = coins.into_iter();
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.size_hint(), (3, Some(3)));
    }

    #[test]
    fn can_iterate_owned() {
        let coins = mock_coins();
        let mut moved = Coins::default();
        for c in coins {
            moved.add(c).unwrap();
        }
        assert_eq!(moved.len(), 3);

        assert!(mock_coins().into_iter().eq(mock_coins().to_vec()));
    }

    #[test]
    fn can_iterate_borrowed() {
        let coins = mock_coins();
        assert!(coins
            .iter()
            .map(|c| &c.denom)
            .eq(coins.to_vec().iter().map(|c| &c.denom)));

        // can still use the coins afterwards
        assert_eq!(coins.amount_of("uatom"), Uint256::new(12345));
    }

    #[test]
    fn contains_only_works() {
        // doctests copied here because coverage does not work for doctests
        let coins: Coins = [coin(100, "uatom")].try_into().unwrap();
        assert_eq!(coins.contains_only("uatom").unwrap(), Uint256::new(100));
        assert_eq!(coins.contains_only("uluna"), None);

        let coins: Coins = [coin(100, "uatom"), coin(200, "uusd")].try_into().unwrap();
        assert_eq!(coins.contains_only("uatom"), None);
    }
}
