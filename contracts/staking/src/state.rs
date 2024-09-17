use std::any::type_name;

use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use cosmwasm_std::{
    from_json,
    storage_keys::{namespace_with_key, to_length_prefixed},
    to_json_vec, Addr, CanonicalAddr, Decimal, StdError, StdResult, Storage, Uint128,
};

pub const KEY_INVESTMENT: &[u8] = b"invest";
pub const KEY_TOKEN_INFO: &[u8] = b"token";
pub const KEY_TOTAL_SUPPLY: &[u8] = b"total_supply";

pub const PREFIX_BALANCE: &[u8] = b"balance";
pub const PREFIX_CLAIMS: &[u8] = b"claim";

pub fn may_load_map(
    storage: &dyn Storage,
    prefix: &[u8],
    key: &CanonicalAddr,
) -> StdResult<Option<Uint128>> {
    storage
        .get(&namespace_with_key(&[prefix], key))
        .map(from_json)
        .transpose()
}

pub fn save_map(
    storage: &mut dyn Storage,
    prefix: &[u8],
    key: &CanonicalAddr,
    value: Uint128,
) -> StdResult<()> {
    storage.set(&namespace_with_key(&[prefix], key), &to_json_vec(&value)?);
    Ok(())
}

pub fn load_map(storage: &dyn Storage, prefix: &[u8], key: &CanonicalAddr) -> StdResult<Uint128> {
    may_load_map(storage, prefix, key)?
        .ok_or_else(|| StdError::not_found(format!("map value for {key}")))
}

/// Investment info is fixed at initialization, and is used to control the function of the contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InvestmentInfo {
    /// owner created the contract and takes a cut
    pub owner: Addr,
    /// this is the denomination we can stake (and only one we accept for payments)
    pub bond_denom: String,
    /// this is how much the owner takes as a cut when someone unbonds
    pub exit_tax: Decimal,
    /// All tokens are bonded to this validator
    /// addr_humanize/addr_canonicalize doesn't work for validator addrresses (e.g. cosmosvaloper1...)
    pub validator: String,
    /// This is the minimum amount we will pull out to reinvest, as well as a minimum
    /// that can be unbonded (to avoid needless staking tx)
    pub min_withdrawal: Uint128,
}

/// Info to display the derivative token in a UI
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct TokenInfo {
    /// name of the derivative token
    pub name: String,
    /// symbol / ticker of the derivative token
    pub symbol: String,
    /// decimal places of the derivative token (for UI)
    pub decimals: u8,
}

/// Supply is dynamic and tracks the current supply of staked and ERC20 tokens.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default, JsonSchema)]
pub struct Supply {
    /// issued is how many derivative tokens this contract has issued
    pub issued: Uint128,
    /// bonded is how many native tokens exist bonded to the validator
    pub bonded: Uint128,
    /// claims is how many tokens need to be reserved paying back those who unbonded
    pub claims: Uint128,
}

pub fn load_item<T: DeserializeOwned>(storage: &dyn Storage, key: &[u8]) -> StdResult<T> {
    storage
        .get(&to_length_prefixed(key))
        .ok_or_else(|| StdError::not_found(type_name::<T>()))
        .and_then(from_json)
}

pub fn save_item<T: Serialize>(storage: &mut dyn Storage, key: &[u8], item: &T) -> StdResult<()> {
    storage.set(&to_length_prefixed(key), &to_json_vec(item)?);
    Ok(())
}

pub fn update_item<T, A, E>(storage: &mut dyn Storage, key: &[u8], action: A) -> Result<T, E>
where
    T: Serialize + DeserializeOwned,
    A: FnOnce(T) -> Result<T, E>,
    E: From<StdError>,
{
    let input = load_item(storage, key)?;
    let output = action(input)?;
    save_item(storage, key, &output)?;
    Ok(output)
}
