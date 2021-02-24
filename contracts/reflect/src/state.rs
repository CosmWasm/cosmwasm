#![allow(clippy::field_reassign_with_default)] // see https://github.com/CosmWasm/cosmwasm/issues/685

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Storage, SubCallResult};
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

pub fn subcalls(storage: &mut dyn Storage) -> Bucket<SubCallResult> {
    bucket(storage, RESULT_PREFIX)
}

pub fn subcalls_read(storage: &dyn Storage) -> ReadonlyBucket<SubCallResult> {
    bucket_read(storage, RESULT_PREFIX)
}
