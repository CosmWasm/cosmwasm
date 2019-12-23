use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use snafu::{OptionExt, ResultExt};

use cosmwasm::errors::{ContractErr, ParseErr, Result, SerializeErr, Unauthorized};
use cosmwasm::serde::{from_slice, to_vec};
use cosmwasm::traits::{Api, Extern, Storage};
use cosmwasm::types::{CanonicalAddr, CosmosMsg, HumanAddr, Params, Response};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub verifier: HumanAddr,
    pub beneficiary: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub verifier: CanonicalAddr,
    pub beneficiary: CanonicalAddr,
    pub funder: CanonicalAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HandleMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum QueryMsg {
    // returns a human-readable representation of the verifier
    // use to ensure query path works in integration tests
    Verifier {},
}

pub static CONFIG_KEY: &[u8] = b"config";

pub fn init<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    params: Params,
    msg: InitMsg,
) -> Result<Response> {
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
    _: HandleMsg,
) -> Result<Response> {
    let data = deps.storage.get(CONFIG_KEY).context(ContractErr {
        msg: "uninitialized data",
    })?;
    let state: State = from_slice(&data).context(ParseErr { kind: "State" })?;

    if params.message.signer == state.verifier {
        let to_addr = deps.api.human_address(&state.beneficiary)?;
        let from_addr = deps.api.human_address(&params.contract.address)?;
        let res = Response {
            log: Some(format!("released funds to {}", to_addr)),
            messages: vec![CosmosMsg::Send {
                from_address: from_addr,
                to_address: to_addr,
                amount: params.contract.balance.unwrap_or_default(),
            }],
            data: None,
        };
        Ok(res)
    } else {
        Unauthorized {}.fail()
    }
}

pub fn query<S: Storage, A: Api>(deps: &Extern<S, A>, msg: QueryMsg) -> Result<Vec<u8>> {
    match msg {
        QueryMsg::Verifier {} => query_verifier(deps),
    }
}

fn query_verifier<S: Storage, A: Api>(deps: &Extern<S, A>) -> Result<Vec<u8>> {
    let data = deps.storage.get(CONFIG_KEY).context(ContractErr {
        msg: "uninitialized data",
    })?;
    let state: State = from_slice(&data).context(ParseErr { kind: "State" })?;
    let addr = deps.api.human_address(&state.verifier)?;
    // we just pass the address as raw bytes
    // these will be base64 encoded into the json we return, and parsed on the way out.
    // maybe we should wrap this in a struct then json encode it into a vec?
    // other ideas?
    Ok(addr.as_str().as_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    use super::*;
    use cosmwasm::mock::{dependencies, mock_params};
    use cosmwasm::storage::transactional_deps;
    // import trait to get access to read
    use cosmwasm::traits::ReadonlyStorage;
    use cosmwasm::types::coin;

    #[test]
    fn proper_initialization() {
        let mut deps = dependencies(20);

        let verifier = HumanAddr(String::from("verifies"));
        let beneficiary = HumanAddr(String::from("benefits"));
        let creator = HumanAddr(String::from("creator"));
        let expected_state = State {
            verifier: deps.api.canonical_address(&verifier).unwrap(),
            beneficiary: deps.api.canonical_address(&beneficiary).unwrap(),
            funder: deps.api.canonical_address(&creator).unwrap(),
        };

        let msg = InitMsg {
            verifier,
            beneficiary,
        };
        let params = mock_params(&deps.api, creator.as_str(), &coin("1000", "earth"), &[]);
        let res = init(&mut deps, params, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's check the state
        let data = deps.storage.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_slice(&data).unwrap();
        assert_eq!(state, expected_state);
    }

    #[test]
    fn init_and_query() {
        let mut deps = dependencies(20);

        let verifier = HumanAddr(String::from("verifies"));
        let beneficiary = HumanAddr(String::from("benefits"));
        let creator = HumanAddr(String::from("creator"));
        let msg = InitMsg {
            verifier: verifier.clone(),
            beneficiary,
        };
        let params = mock_params(&deps.api, creator.as_str(), &coin("1000", "earth"), &[]);
        let res = init(&mut deps, params, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // now let's query
        let qres = query(&deps, QueryMsg::Verifier {}).unwrap();
        let returned = from_utf8(&qres).unwrap();
        assert_eq!(verifier.as_str(), returned);
    }

    #[test]
    fn checkpointing_works_on_contract() {
        let mut deps = dependencies(20);

        let verifier = HumanAddr(String::from("verifies"));
        let beneficiary = HumanAddr(String::from("benefits"));
        let creator = HumanAddr(String::from("creator"));
        let expected_state = State {
            verifier: deps.api.canonical_address(&verifier).unwrap(),
            beneficiary: deps.api.canonical_address(&beneficiary).unwrap(),
            funder: deps.api.canonical_address(&creator).unwrap(),
        };

        // let's see if we can checkpoint on a contract
        let res = transactional_deps(&mut deps, &|deps| {
            let msg = InitMsg {
                verifier: verifier.clone(),
                beneficiary: beneficiary.clone(),
            };
            let params = mock_params(&deps.api, creator.as_str(), &coin("1000", "earth"), &[]);

            init(deps, params, msg)
        })
        .unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's check the state
        let data = deps.storage.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_slice(&data).unwrap();
        assert_eq!(state, expected_state);
    }

    #[test]
    fn proper_handle() {
        let mut deps = dependencies(20);

        // initialize the store
        let verifier = HumanAddr(String::from("verifies"));
        let beneficiary = HumanAddr(String::from("benefits"));

        let init_msg = InitMsg {
            verifier: verifier.clone(),
            beneficiary: beneficiary.clone(),
        };
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
            verifier.as_str(),
            &coin("15", "earth"),
            &coin("1015", "earth"),
        );
        let handle_res = handle(&mut deps, handle_params, HandleMsg {}).unwrap();
        assert_eq!(1, handle_res.messages.len());
        let msg = handle_res.messages.get(0).expect("no message");
        assert_eq!(
            msg,
            &CosmosMsg::Send {
                from_address: HumanAddr("cosmos2contract".to_string()),
                to_address: beneficiary,
                amount: coin("1015", "earth"),
            }
        );
        assert_eq!(
            Some("released funds to benefits".to_string()),
            handle_res.log
        );
    }

    #[test]
    fn failed_handle() {
        let mut deps = dependencies(20);

        // initialize the store
        let verifier = HumanAddr(String::from("verifies"));
        let beneficiary = HumanAddr(String::from("benefits"));
        let creator = HumanAddr(String::from("creator"));

        let init_msg = InitMsg {
            verifier: verifier.clone(),
            beneficiary: beneficiary.clone(),
        };
        let init_params = mock_params(
            &deps.api,
            creator.as_str(),
            &coin("1000", "earth"),
            &coin("1000", "earth"),
        );
        let init_res = init(&mut deps, init_params, init_msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        // beneficiary can release it
        let handle_params =
            mock_params(&deps.api, beneficiary.as_str(), &[], &coin("1000", "earth"));
        let handle_res = handle(&mut deps, handle_params, HandleMsg {});
        assert!(handle_res.is_err());

        // state should not change
        let data = deps.storage.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_slice(&data).unwrap();
        assert_eq!(
            state,
            State {
                verifier: deps.api.canonical_address(&verifier).unwrap(),
                beneficiary: deps.api.canonical_address(&beneficiary).unwrap(),
                funder: deps.api.canonical_address(&creator).unwrap(),
            }
        );
    }
}
