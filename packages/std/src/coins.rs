use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::math::Uint128;

pub trait GenericCoin {
    fn key(&self) -> String;
    fn value(&self) -> Uint128;
    fn add_value(&mut self, add_value: Uint128);
}

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

impl GenericCoin for Coin {
    fn key(&self) -> String {
        self.denom.clone()
    }

    fn value(&self) -> Uint128 {
        self.amount
    }

    fn add_value(&mut self, add_value: Uint128) {
        self.amount += add_value;
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn has_coins_matches() {
        let wallet = vec![coin(12345, "ETH"), coin(555, "BTC")];

        // less than same type
        assert!(has_coins(&wallet, &coin(777, "ETH")));
    }
}
