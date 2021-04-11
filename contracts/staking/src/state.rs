use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Storage, Uint128};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};

pub const KEY_INVESTMENT: &[u8] = b"invest";
pub const KEY_TOKEN_INFO: &[u8] = b"token";
pub const KEY_TOTAL_SUPPLY: &[u8] = b"total_supply";

pub const PREFIX_BALANCE: &[u8] = b"balance";
pub const PREFIX_CLAIMS: &[u8] = b"claim";

/// balances are state of the erc20 tokens
pub fn balances(storage: &mut dyn Storage) -> Bucket<Uint128> {
    bucket(storage, PREFIX_BALANCE)
}

pub fn balances_read(storage: &dyn Storage) -> ReadonlyBucket<Uint128> {
    bucket_read(storage, PREFIX_BALANCE)
}

/// claims are the claims to money being unbonded
pub fn claims(storage: &mut dyn Storage) -> Bucket<Uint128> {
    bucket(storage, PREFIX_CLAIMS)
}

pub fn claims_read(storage: &dyn Storage) -> ReadonlyBucket<Uint128> {
    bucket_read(storage, PREFIX_CLAIMS)
}

/// Investment info is fixed at initialization, and is used to control the function of the contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
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
    /// This is the minimum amount we will pull out to reinvest, as well as a minumum
    /// that can be unbonded (to avoid needless staking tx)
    pub min_withdrawal: Uint128,
}

/// Info to display the derivative token in a UI
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo {
    /// name of the derivative token
    pub name: String,
    /// symbol / ticker of the derivative token
    pub symbol: String,
    /// decimal places of the derivative token (for UI)
    pub decimals: u8,
}

/// Supply is dynamic and tracks the current supply of staked and ERC20 tokens.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, JsonSchema)]
pub struct Supply {
    /// issued is how many derivative tokens this contract has issued
    pub issued: Uint128,
    /// bonded is how many native tokens exist bonded to the validator
    pub bonded: Uint128,
    /// claims is how many tokens need to be reserved paying back those who unbonded
    pub claims: Uint128,
}

pub fn invest_info(storage: &mut dyn Storage) -> Singleton<InvestmentInfo> {
    singleton(storage, KEY_INVESTMENT)
}

pub fn invest_info_read(storage: &dyn Storage) -> ReadonlySingleton<InvestmentInfo> {
    singleton_read(storage, KEY_INVESTMENT)
}

pub fn token_info(storage: &mut dyn Storage) -> Singleton<TokenInfo> {
    singleton(storage, KEY_TOKEN_INFO)
}

pub fn token_info_read(storage: &dyn Storage) -> ReadonlySingleton<TokenInfo> {
    singleton_read(storage, KEY_TOKEN_INFO)
}

pub fn total_supply(storage: &mut dyn Storage) -> Singleton<Supply> {
    singleton(storage, KEY_TOTAL_SUPPLY)
}

pub fn total_supply_read(storage: &dyn Storage) -> ReadonlySingleton<Supply> {
    singleton_read(storage, KEY_TOTAL_SUPPLY)
}
