use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Reply, Storage};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};

const CONFIG_KEY: &[u8] = b"config";
const RESULT_PREFIX: &[u8] = b"result";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: CanonicalAddr,
}

pub fn config(storage: &mut dyn Storage) -> Singleton<State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn replies(storage: &mut dyn Storage) -> Bucket<Reply> {
    bucket(storage, RESULT_PREFIX)
}

pub fn replies_read(storage: &dyn Storage) -> ReadonlyBucket<Reply> {
    bucket_read(storage, RESULT_PREFIX)
}
