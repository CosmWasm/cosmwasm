use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Storage};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};

pub const KEY_CONFIG: &[u8] = b"config";
pub const PREFIX_ACCOUNTS: &[u8] = b"accounts";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub reflect_code_id: u64,
}

/// accounts is lookup of channel_id to reflect contract
pub fn accounts(storage: &mut dyn Storage) -> Bucket<Addr> {
    bucket(storage, PREFIX_ACCOUNTS)
}

pub fn accounts_read(storage: &dyn Storage) -> ReadonlyBucket<Addr> {
    bucket_read(storage, PREFIX_ACCOUNTS)
}

pub fn config(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, KEY_CONFIG)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, KEY_CONFIG)
}
