use std::str::from_utf8;

use serde::{Deserialize, Serialize};
use snafu::{OptionExt, ResultExt};

use cosmwasm::errors::{ContractErr, ParseErr, Result, SerializeErr, Unauthorized, Utf8Err};
use cosmwasm::query::perform_raw_query;
use cosmwasm::serde::{from_slice, to_vec};
use cosmwasm::storage::Storage;
use cosmwasm::types::{CosmosMsg, Params, QueryResponse, RawQuery, Response};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InitMsg {
    pub verifier: String,
    pub beneficiary: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct State {
    pub verifier: String,
    pub beneficiary: String,
    pub funder: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HandleMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum QueryMsg {
    Raw(RawQuery),
}

// raw_query is a helper to generate a serialized format of a raw_query
// meant for test code and integration tests
pub fn raw_query(key: &[u8]) -> Result<Vec<u8>> {
    let key = from_utf8(key).context(Utf8Err {})?.to_string();
    to_vec(&QueryMsg::Raw(RawQuery { key })).context(SerializeErr { kind: "QueryMsg" })
}

pub static CONFIG_KEY: &[u8] = b"config";

pub fn init<T: Storage>(store: &mut T, params: Params, msg: Vec<u8>) -> Result<Response> {
    let msg: InitMsg = from_slice(&msg).context(ParseErr { kind: "InitMsg" })?;
    store.set(
        CONFIG_KEY,
        &to_vec(&State {
            verifier: msg.verifier,
            beneficiary: msg.beneficiary,
            funder: params.message.signer,
        })
        .context(SerializeErr { kind: "State" })?,
    );
    Ok(Response::default())
}

pub fn handle<T: Storage>(store: &mut T, params: Params, _: Vec<u8>) -> Result<Response> {
    let data = store.get(CONFIG_KEY).context(ContractErr {
        msg: "uninitialized data",
    })?;
    let state: State = from_slice(&data).context(ParseErr { kind: "State" })?;

    if params.message.signer == state.verifier {
        let res = Response {
            messages: vec![CosmosMsg::Send {
                from_address: params.contract.address,
                to_address: state.beneficiary,
                amount: params.contract.balance.unwrap_or_default(),
            }],
            log: Some("released funds!".to_string()),
            data: None,
        };
        Ok(res)
    } else {
        Unauthorized {}.fail()
    }
}

pub fn query<T: Storage>(store: &T, msg: Vec<u8>) -> Result<QueryResponse> {
    let msg: QueryMsg = from_slice(&msg).context(ParseErr { kind: "QueryMsg" })?;
    match msg {
        QueryMsg::Raw(raw) => perform_raw_query(store, raw),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm::mock::MockStorage;
    use cosmwasm::types::{coin, mock_params};

    #[test]
    fn proper_initialization() {
        let mut store = MockStorage::new();
        let msg = to_vec(&InitMsg {
            verifier: String::from("verifies"),
            beneficiary: String::from("benefits"),
        })
        .unwrap();
        let params = mock_params("creator", &coin("1000", "earth"), &[]);
        let res = init(&mut store, params, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's check the state
        let data = store.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_slice(&data).unwrap();
        assert_eq!(
            state,
            State {
                verifier: "verifies".to_string(),
                beneficiary: "benefits".to_string(),
                funder: "creator".to_string(),
            }
        );
    }

    #[test]
    fn proper_init_and_query() {
        let mut store = MockStorage::new();
        let msg = to_vec(&InitMsg {
            verifier: String::from("foo"),
            beneficiary: String::from("bar"),
        })
        .unwrap();
        let params = mock_params("creator", &coin("1000", "earth"), &[]);
        let _res = init(&mut store, params, msg).unwrap();

        let q_res = query(&store, raw_query(b"random").unwrap()).unwrap();
        assert_eq!(q_res.results.len(), 0);

        // query for state
        let mut q_res = query(&store, raw_query(CONFIG_KEY).unwrap()).unwrap();
        let model = q_res.results.pop().unwrap();
        let state: State = from_slice(&model.val).unwrap();
        assert_eq!(
            state,
            State {
                verifier: "foo".to_string(),
                beneficiary: "bar".to_string(),
                funder: "creator".to_string(),
            }
        );
    }

    #[test]
    fn fails_on_bad_init() {
        let mut store = MockStorage::new();
        let bad_msg = b"{}".to_vec();
        let params = mock_params("creator", &coin("1000", "earth"), &[]);
        let res = init(&mut store, params, bad_msg);
        assert_eq!(true, res.is_err());
    }

    #[test]
    fn proper_handle() {
        let mut store = MockStorage::new();

        // initialize the store
        let init_msg = to_vec(&InitMsg {
            verifier: String::from("verifies"),
            beneficiary: String::from("benefits"),
        })
        .unwrap();
        let init_params = mock_params("creator", &coin("1000", "earth"), &coin("1000", "earth"));
        let init_res = init(&mut store, init_params, init_msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        // beneficiary can release it
        let handle_params = mock_params("verifies", &coin("15", "earth"), &coin("1015", "earth"));
        let handle_res = handle(&mut store, handle_params, Vec::new()).unwrap();
        assert_eq!(1, handle_res.messages.len());
        let msg = handle_res.messages.get(0).expect("no message");
        assert_eq!(
            msg,
            &CosmosMsg::Send {
                from_address: "cosmos2contract".to_string(),
                to_address: "benefits".to_string(),
                amount: coin("1015", "earth"),
            }
        );

        // it worked, let's check the state
        let data = store.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_slice(&data).unwrap();
        assert_eq!(
            state,
            State {
                verifier: "verifies".to_string(),
                beneficiary: "benefits".to_string(),
                funder: "creator".to_string(),
            }
        );
    }

    #[test]
    fn failed_handle() {
        let mut store = MockStorage::new();

        // initialize the store
        let init_msg = to_vec(&InitMsg {
            verifier: String::from("verifies"),
            beneficiary: String::from("benefits"),
        })
        .unwrap();
        let init_params = mock_params("creator", &coin("1000", "earth"), &coin("1000", "earth"));
        let init_res = init(&mut store, init_params, init_msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        // beneficiary can release it
        let handle_params = mock_params("benefits", &[], &coin("1000", "earth"));
        let handle_res = handle(&mut store, handle_params, Vec::new());
        assert!(handle_res.is_err());

        // state should not change
        let data = store.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_slice(&data).unwrap();
        assert_eq!(
            state,
            State {
                verifier: "verifies".to_string(),
                beneficiary: "benefits".to_string(),
                funder: "creator".to_string(),
            }
        );
    }
}
