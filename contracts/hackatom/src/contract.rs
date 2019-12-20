use std::str::from_utf8;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use snafu::{OptionExt, ResultExt};

use cosmwasm::errors::{ContractErr, ParseErr, Result, SerializeErr, Unauthorized, Utf8Err};
use cosmwasm::query::perform_raw_query;
use cosmwasm::serde::{from_slice, to_vec};
use cosmwasm::traits::{Api, Extern, Storage};
use cosmwasm::types::{CosmosMsg, Params, QueryResponse, RawQuery, Response};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    // these use humanized addresses
    pub verifier: String,
    pub beneficiary: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    // these are stored as canonical addresses
    pub verifier: Vec<u8>,
    pub beneficiary: Vec<u8>,
    pub funder: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HandleMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
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

pub fn init<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    params: Params,
    msg: Vec<u8>,
) -> Result<Response> {
    let msg: InitMsg = from_slice(&msg).context(ParseErr { kind: "InitMsg" })?;
    deps.storage.set(
        CONFIG_KEY,
        &to_vec(&State {
            verifier: deps.api.canonical_address(&msg.verifier)?,
            beneficiary: deps.api.canonical_address(&msg.beneficiary)?,
            funder: params.message.signer,
        })
        .context(SerializeErr { kind: "State" })?,
    );
    Ok(Response::default())
}

pub fn handle<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    params: Params,
    _: Vec<u8>,
) -> Result<Response> {
    let data = deps.storage.get(CONFIG_KEY).context(ContractErr {
        msg: "uninitialized data",
    })?;
    let state: State = from_slice(&data).context(ParseErr { kind: "State" })?;

    if params.message.signer == state.verifier {
        let res = Response {
            log: Some(format!(
                "released funds to {}",
                deps.api.human_address(&state.beneficiary)?
            )),
            messages: vec![CosmosMsg::Send {
                from_address: params.contract.address,
                to_address: state.beneficiary,
                amount: params.contract.balance.unwrap_or_default(),
            }],
            data: None,
        };
        Ok(res)
    } else {
        Unauthorized {}.fail()
    }
}

pub fn query<S: Storage, A: Api>(deps: &Extern<S, A>, msg: Vec<u8>) -> Result<QueryResponse> {
    let msg: QueryMsg = from_slice(&msg).context(ParseErr { kind: "QueryMsg" })?;
    match msg {
        QueryMsg::Raw(raw) => perform_raw_query(&deps.storage, raw),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm::mock::{dependencies, mock_params};
    // import trait to get access to read
    use cosmwasm::traits::{ReadonlyStorage};
    use cosmwasm::types::coin;

    #[test]
    fn proper_initialization() {
        let mut deps = dependencies(20);
        let msg = to_vec(&InitMsg {
            verifier: String::from("verifies"),
            beneficiary: String::from("benefits"),
        })
        .unwrap();
        let params = mock_params(&deps.api, "creator", &coin("1000", "earth"), &[]);
        let res = init(&mut deps, params, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's check the state
        let data = deps.storage.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_slice(&data).unwrap();
        assert_eq!(
            state,
            State {
                verifier: deps.api.canonical_address("verifies").unwrap(),
                beneficiary: deps.api.canonical_address("benefits").unwrap(),
                funder: deps.api.canonical_address("creator").unwrap(),
            }
        );
    }

    #[test]
    fn proper_init_and_query() {
        let mut deps = dependencies(20);
        let msg = to_vec(&InitMsg {
            verifier: String::from("foo"),
            beneficiary: String::from("bar"),
        })
        .unwrap();
        let params = mock_params(&deps.api, "creator", &coin("1000", "earth"), &[]);
        let _res = init(&mut deps, params, msg).unwrap();

        let q_res = query(&deps, raw_query(b"random").unwrap()).unwrap();
        assert_eq!(q_res.results.len(), 0);

        // query for state
        let mut q_res = query(&deps, raw_query(CONFIG_KEY).unwrap()).unwrap();
        let model = q_res.results.pop().unwrap();
        let state: State = from_slice(&model.val).unwrap();
        assert_eq!(
            state,
            State {
                verifier: deps.api.canonical_address("foo").unwrap(),
                beneficiary: deps.api.canonical_address("bar").unwrap(),
                funder: deps.api.canonical_address("creator").unwrap(),
            }
        );
    }

    #[test]
    fn fails_on_bad_init() {
        let mut deps = dependencies(20);
        let bad_msg = b"{}".to_vec();
        let params = mock_params(&deps.api, "creator", &coin("1000", "earth"), &[]);
        let res = init(&mut deps, params, bad_msg);
        assert_eq!(true, res.is_err());
    }

    #[test]
    fn proper_handle() {
        let mut deps = dependencies(20);

        // initialize the store
        let init_msg = to_vec(&InitMsg {
            verifier: String::from("verifies"),
            beneficiary: String::from("benefits"),
        })
        .unwrap();
        let init_params = mock_params(
            &deps.api,
            "creator",
            &coin("1000", "earth"),
            &coin("1000", "earth"),
        );
        let init_res = init(&mut deps, init_params, init_msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        // beneficiary can release it
        let handle_params = mock_params(
            &deps.api,
            "verifies",
            &coin("15", "earth"),
            &coin("1015", "earth"),
        );
        let handle_res = handle(&mut deps, handle_params, Vec::new()).unwrap();
        assert_eq!(1, handle_res.messages.len());
        let msg = handle_res.messages.get(0).expect("no message");
        assert_eq!(
            msg,
            &CosmosMsg::Send {
                from_address: deps.api.canonical_address("cosmos2contract").unwrap(),
                to_address: deps.api.canonical_address("benefits").unwrap(),
                amount: coin("1015", "earth"),
            }
        );
        assert_eq!(
            Some("released funds to benefits".to_string()),
            handle_res.log
        );

        // it worked, let's check the state
        let data = deps.storage.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_slice(&data).unwrap();
        assert_eq!(
            state,
            State {
                verifier: deps.api.canonical_address("verifies").unwrap(),
                beneficiary: deps.api.canonical_address("benefits").unwrap(),
                funder: deps.api.canonical_address("creator").unwrap(),
            }
        );
    }

    #[test]
    fn failed_handle() {
        let mut deps = dependencies(20);

        // initialize the store
        let init_msg = to_vec(&InitMsg {
            verifier: String::from("verifies"),
            beneficiary: String::from("benefits"),
        })
        .unwrap();
        let init_params = mock_params(
            &deps.api,
            "creator",
            &coin("1000", "earth"),
            &coin("1000", "earth"),
        );
        let init_res = init(&mut deps, init_params, init_msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        // beneficiary can release it
        let handle_params = mock_params(&deps.api, "benefits", &[], &coin("1000", "earth"));
        let handle_res = handle(&mut deps, handle_params, Vec::new());
        assert!(handle_res.is_err());

        // state should not change
        let data = deps.storage.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_slice(&data).unwrap();
        assert_eq!(
            state,
            State {
                verifier: deps.api.canonical_address("verifies").unwrap(),
                beneficiary: deps.api.canonical_address("benefits").unwrap(),
                funder: deps.api.canonical_address("creator").unwrap(),
            }
        );
    }
}
