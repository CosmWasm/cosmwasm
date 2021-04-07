use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::math::Uint128;

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct Coin {
    pub denom: String,
    pub amount: Uint128,
}

impl Coin {
    pub fn new<S: Into<String>>(amount: u128, denom: S) -> Self {
        Coin {
            amount: Uint128(amount),
            denom: denom.into(),
        }
    }
}

/// A shortcut constructor for a set of one denomination of coins
///
/// # Examples
///
/// ```
/// # use cosmwasm_std::{coins, BankMsg, CosmosMsg, Response};
/// # use cosmwasm_std::testing::{mock_env, mock_info};
/// # let env = mock_env();
/// # let info = mock_info("sender", &[]);
/// let tip = coins(123, "ucosm");
///
/// let mut response: Response = Default::default();
/// response.messages = vec![CosmosMsg::Bank(BankMsg::Send {
///   to_address: info.sender.into(),
///   amount: tip,
/// })];
/// ```
pub fn coins<S: Into<String>>(amount: u128, denom: S) -> Vec<Coin> {
    vec![coin(amount, denom)]
}

/// A shorthand constructor for Coin
///
/// # Examples
///
/// ```
/// # use cosmwasm_std::{coin, BankMsg, CosmosMsg, Response};
/// # use cosmwasm_std::testing::{mock_env, mock_info};
/// # let env = mock_env();
/// # let info = mock_info("sender", &[]);
/// let tip = vec![
///     coin(123, "ucosm"),
///     coin(24, "ustake"),
/// ];
///
/// let mut response: Response = Default::default();
/// response.messages = vec![CosmosMsg::Bank(BankMsg::Send {
///     to_address: info.sender.into(),
///     amount: tip,
/// })];
/// ```
pub fn coin<S: Into<String>>(amount: u128, denom: S) -> Coin {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coin_works() {
        let a = coin(123, "ucosm");
        assert_eq!(
            a,
            Coin {
                amount: Uint128(123),
                denom: "ucosm".to_string()
            }
        );

        let zero = coin(0, "ucosm");
        assert_eq!(
            zero,
            Coin {
                amount: Uint128(0),
                denom: "ucosm".to_string()
            }
        );

        let string_denom = coin(42, String::from("ucosm"));
        assert_eq!(
            string_denom,
            Coin {
                amount: Uint128(42),
                denom: "ucosm".to_string()
            }
        );
    }

    #[test]
    fn coins_works() {
        let a = coins(123, "ucosm");
        assert_eq!(
            a,
            vec![Coin {
                amount: Uint128(123),
                denom: "ucosm".to_string()
            }]
        );

        let zero = coins(0, "ucosm");
        assert_eq!(
            zero,
            vec![Coin {
                amount: Uint128(0),
                denom: "ucosm".to_string()
            }]
        );

        let string_denom = coins(42, String::from("ucosm"));
        assert_eq!(
            string_denom,
            vec![Coin {
                amount: Uint128(42),
                denom: "ucosm".to_string()
            }]
        );
    }

    #[test]
    fn has_coins_matches() {
        let wallet = vec![coin(12345, "ETH"), coin(555, "BTC")];

        // less than same type
        assert!(has_coins(&wallet, &coin(777, "ETH")));
    }
}
