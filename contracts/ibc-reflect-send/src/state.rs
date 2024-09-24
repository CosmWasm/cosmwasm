use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    from_json,
    storage_keys::{namespace_with_key, to_length_prefixed},
    to_json_vec, Addr, Coin, Order, StdError, StdResult, Storage, Timestamp,
};

pub const KEY_CONFIG: &[u8] = b"config";
/// accounts is lookup of channel_id to reflect contract
pub const PREFIX_ACCOUNTS: &[u8] = b"accounts";
/// Upper bound for ranging over accounts
const PREFIX_ACCOUNTS_UPPER_BOUND: &[u8] = b"accountt"; // spellchecker:disable-line

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub admin: Addr,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
pub struct AccountData {
    /// last block balance was updated (0 is never)
    pub last_update_time: Timestamp,
    /// In normal cases, it should be set, but there is a delay between binding
    /// the channel and making a query and in that time it is empty.
    ///
    /// Since we do not have a way to validate the remote address format, this
    /// must not be of type `Addr`.
    pub remote_addr: Option<String>,
    pub remote_balance: Vec<Coin>,
}

pub fn may_load_account(storage: &dyn Storage, id: &str) -> StdResult<Option<AccountData>> {
    storage
        .get(&namespace_with_key(&[PREFIX_ACCOUNTS], id.as_bytes()))
        .map(from_json)
        .transpose()
}

pub fn load_account(storage: &dyn Storage, id: &str) -> StdResult<AccountData> {
    may_load_account(storage, id)?.ok_or_else(|| StdError::not_found(format!("account {id}")))
}

pub fn save_account(storage: &mut dyn Storage, id: &str, account: &AccountData) -> StdResult<()> {
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
) -> impl Iterator<Item = StdResult<(String, AccountData)>> + '_ {
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

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    storage
        .get(&to_length_prefixed(KEY_CONFIG))
        .ok_or_else(|| StdError::not_found("config"))
        .and_then(from_json)
}

pub fn save_config(storage: &mut dyn Storage, item: &Config) -> StdResult<()> {
    storage.set(&to_length_prefixed(KEY_CONFIG), &to_json_vec(item)?);
    Ok(())
}
