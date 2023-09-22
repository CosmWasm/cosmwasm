use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    from_json,
    storage_keys::{namespace_with_key, to_length_prefixed},
    to_json_vec, Addr, Reply, StdError, StdResult, Storage,
};

const CONFIG_KEY: &[u8] = b"config";
const RESULT_PREFIX: &[u8] = b"result";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub owner: Addr,
}

pub fn load_reply(storage: &dyn Storage, id: u64) -> StdResult<Reply> {
    storage
        .get(&namespace_with_key(&[RESULT_PREFIX], &id.to_be_bytes()))
        .ok_or_else(|| StdError::not_found(format!("reply {id}")))
        .and_then(from_json)
}

pub fn save_reply(storage: &mut dyn Storage, id: u64, reply: &Reply) -> StdResult<()> {
    storage.set(
        &namespace_with_key(&[RESULT_PREFIX], &id.to_be_bytes()),
        &to_json_vec(reply)?,
    );
    Ok(())
}

pub fn remove_reply(storage: &mut dyn Storage, id: u64) {
    storage.remove(&namespace_with_key(&[RESULT_PREFIX], &id.to_be_bytes()));
}

pub fn load_config(storage: &dyn Storage) -> StdResult<State> {
    storage
        .get(&to_length_prefixed(CONFIG_KEY))
        .ok_or_else(|| StdError::not_found("config"))
        .and_then(from_json)
}

pub fn save_config(storage: &mut dyn Storage, item: &State) -> StdResult<()> {
    storage.set(&to_length_prefixed(CONFIG_KEY), &to_json_vec(item)?);
    Ok(())
}
