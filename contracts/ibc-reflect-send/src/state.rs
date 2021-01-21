#![allow(clippy::field_reassign_with_default)] // see https://github.com/CosmWasm/cosmwasm/issues/685

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Coin, HumanAddr, Storage};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};

pub const KEY_CONFIG: &[u8] = b"config";
pub const PREFIX_ACCOUNTS: &[u8] = b"accounts";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AccountData {
    /// last block balance was updated (0 is never)
    pub last_update_height: u64,
    /// in normal cases, it should be set, but there is a delay between binding
    /// the channel and making a query and in that time it is empty
    pub remote_addr: Option<HumanAddr>,
    pub remote_balance: Vec<Coin>,
}

/// accounts is lookup of channel_id to reflect contract
pub fn accounts(storage: &mut dyn Storage) -> Bucket<AccountData> {
    bucket(storage, PREFIX_ACCOUNTS)
}

pub fn accounts_read(storage: &dyn Storage) -> ReadonlyBucket<AccountData> {
    bucket_read(storage, PREFIX_ACCOUNTS)
}

pub fn config(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, KEY_CONFIG)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, KEY_CONFIG)
}
