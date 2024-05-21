use std::any::type_name;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    from_json, storage_keys::to_length_prefixed, to_json_vec, IbcAckCallbackMsg,
    IbcDestinationCallbackMsg, IbcTimeoutCallbackMsg, StdError, StdResult, Storage,
};
use serde::{de::DeserializeOwned, Serialize};

pub const KEY_STATS: &[u8] = b"counts";

/// A counter for the number of times the respective callback has been called
#[cw_serde]
#[derive(Default)]
pub struct CallbackStats {
    pub ibc_ack_callbacks: Vec<IbcAckCallbackMsg>,
    pub ibc_timeout_callbacks: Vec<IbcTimeoutCallbackMsg>,
    pub ibc_destination_callbacks: Vec<IbcDestinationCallbackMsg>,
}

pub fn load_stats(storage: &dyn Storage) -> StdResult<CallbackStats> {
    load_item(storage, KEY_STATS)
}

pub fn save_stats(storage: &mut dyn Storage, counts: &CallbackStats) -> StdResult<()> {
    save_item(storage, KEY_STATS, counts)
}

fn load_item<T: DeserializeOwned>(storage: &dyn Storage, key: &[u8]) -> StdResult<T> {
    storage
        .get(&to_length_prefixed(key))
        .ok_or_else(|| StdError::not_found(type_name::<T>()))
        .and_then(from_json)
}

fn save_item<T: Serialize>(storage: &mut dyn Storage, key: &[u8], item: &T) -> StdResult<()> {
    storage.set(&to_length_prefixed(key), &to_json_vec(item)?);
    Ok(())
}
