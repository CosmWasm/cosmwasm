use std::any::type_name;
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

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

impl FromStr for Coins {
    type Err = StdError;

    fn from_str(s: &str) -> StdResult<Self> {
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
            .into_iter()
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

    /// Return the denoms as a vector of strings.
    /// The vector is guaranteed to not contain duplicates and sorted alphabetically.
    pub fn denoms(&self) -> Vec<String> {
        self.0.keys().cloned().collect()
    }

    pub fn add(&mut self, coin: &Coin) -> StdResult<()> {
        let amount = self
            .0
            .entry(coin.denom.clone())
            .or_insert_with(Uint128::zero);
        *amount = amount.checked_add(coin.amount)?;
        Ok(())
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
            coins.add(&coin).unwrap();
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
}
