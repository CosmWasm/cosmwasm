use std::any::type_name;

use cosmwasm_std::{
    from_json,
    storage_keys::{namespace_with_key, to_length_prefixed},
    to_json_vec, Addr, Order, StdError, StdResult, Storage,
};
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub const KEY_CONFIG: &[u8] = b"config";
pub const KEY_PENDING_CHANNEL: &[u8] = b"pending";
pub const PREFIX_ACCOUNTS: &[u8] = b"accounts";
/// Upper bound for ranging over accounts
const PREFIX_ACCOUNTS_UPPER_BOUND: &[u8] = b"accountt"; // spellchecker:disable-line

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub reflect_code_id: u64,
}

pub fn may_load_account(storage: &dyn Storage, id: &str) -> StdResult<Option<Addr>> {
    storage
        .get(&namespace_with_key(&[PREFIX_ACCOUNTS], id.as_bytes()))
        .map(from_json)
        .transpose()
}

pub fn load_account(storage: &dyn Storage, id: &str) -> StdResult<Addr> {
    may_load_account(storage, id)?.ok_or_else(|| StdError::not_found(format!("account {id}")))
}

pub fn save_account(storage: &mut dyn Storage, id: &str, account: &Addr) -> StdResult<()> {
    storage.set(
        &namespace_with_key(&[PREFIX_ACCOUNTS], id.as_bytes()),
        &to_json_vec(account)?,
    );
    Ok(())
}

pub fn remove_account(storage: &mut dyn Storage, id: &str) {
    storage.remove(&namespace_with_key(&[PREFIX_ACCOUNTS], id.as_bytes()));
}

pub fn range_accounts(
    storage: &dyn Storage,
) -> impl Iterator<Item = StdResult<(String, Addr)>> + '_ {
    let prefix = to_length_prefixed(PREFIX_ACCOUNTS);
    let upper_bound = to_length_prefixed(PREFIX_ACCOUNTS_UPPER_BOUND);
    storage
        .range(Some(&prefix), Some(&upper_bound), Order::Ascending)
        .map(|(key, val)| {
            Ok((
                String::from_utf8(key[PREFIX_ACCOUNTS.len() + 2..].to_vec())?,
                from_json(val)?,
            ))
        })
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
