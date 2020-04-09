use cosmwasm::mock::mock_env;
use cosmwasm::serde::from_slice;
use cosmwasm::traits::Api;
use cosmwasm::types::{coin, ContractResult, CosmosMsg, HumanAddr};

use cosmwasm_vm::testing::{handle, init, mock_instance, query};

use cw_mask::msg::{HandleMsg, InitMsg, OwnerResponse, QueryMsg};

/**
This integration test tries to run and call the generated wasm.
It depends on a release build being available already. You can create that with:

cargo wasm && wasm-gc ./target/wasm32-unknown-unknown/release/hackatom.wasm

Then running `cargo test` will validate we can properly call into that generated data.

You can easily convert unit tests to integration tests.
1. First copy them over verbatum,
2. Then change
    let mut deps = dependencies(20);
To
    let mut deps = mock_instance(WASM);
3. If you access raw storage, where ever you see something like:
    deps.storage.get(CONFIG_KEY).expect("no data stored");
 replace it with:
    deps.with_storage(|store| {
        let data = store.get(CONFIG_KEY).expect("no data stored");
        //...
    });
4. Anywhere you see query(&deps, ...) you must replace it with query(&mut deps, ...)
5. When matching on error codes, you can not use Error types, but rather must use strings:
     match res {
         Err(Error::Unauthorized{..}) => {},
         _ => panic!("Must return unauthorized error"),
     }
     becomes:
     match res {
        ContractResult::Err(msg) => assert_eq!(msg, "Unauthorized"),
        _ => panic!("Expected error"),
     }



**/

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/cw_mask.wasm");
// You can uncomment this line instead to test productionified build from cosmwasm-opt
// static WASM: &[u8] = include_bytes!("../contract.wasm");

#[test]
fn proper_initialization() {
    let mut deps = mock_instance(WASM);

    let msg = InitMsg {};
    let env = mock_env(&deps.api, "creator", &coin("1000", "earth"), &[]);

    // we can just call .unwrap() to assert this was a success
    let res = init(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(&mut deps, QueryMsg::Owner {}).unwrap();
    let value: OwnerResponse = from_slice(res.as_slice()).unwrap();
    assert_eq!("creator", value.owner.as_str());
}

#[test]
fn reflect() {
    let mut deps = mock_instance(WASM);

    let msg = InitMsg {};
    let env = mock_env(
        &deps.api,
        "creator",
        &coin("2", "token"),
        &coin("2", "token"),
    );
    let _res = init(&mut deps, env, msg).unwrap();

    let env = mock_env(&deps.api, "creator", &[], &coin("2", "token"));
    let payload = vec![CosmosMsg::Send {
        from_address: deps.api.human_address(&env.contract.address).unwrap(),
        to_address: HumanAddr::from("friend"),
        amount: coin("1", "token"),
    }];
    let msg = HandleMsg::ReflectMsg {
        msgs: payload.clone(),
    };
    let res = handle(&mut deps, env, msg).unwrap();

    // should return payload
    assert_eq!(payload, res.messages);
}

#[test]
fn reflect_requires_owner() {
    let mut deps = mock_instance(WASM);

    let msg = InitMsg {};
    let env = mock_env(
        &deps.api,
        "creator",
        &coin("2", "token"),
        &coin("2", "token"),
    );
    let _res = init(&mut deps, env, msg).unwrap();

    // signer is not owner
    let env = mock_env(&deps.api, "someone", &[], &coin("2", "token"));
    let payload = vec![CosmosMsg::Send {
        from_address: deps.api.human_address(&env.contract.address).unwrap(),
        to_address: HumanAddr::from("friend"),
        amount: coin("1", "token"),
    }];
    let msg = HandleMsg::ReflectMsg {
        msgs: payload.clone(),
    };

    let res = handle(&mut deps, env, msg);
    match res {
        ContractResult::Err(msg) => assert_eq!(msg, "Unauthorized"),
        _ => panic!("Must return unauthorized error"),
    }
}

#[test]
fn transfer() {
    let mut deps = mock_instance(WASM);

    let msg = InitMsg {};
    let env = mock_env(
        &deps.api,
        "creator",
        &coin("2", "token"),
        &coin("2", "token"),
    );
    let _res = init(&mut deps, env, msg).unwrap();

    let env = mock_env(&deps.api, "creator", &[], &coin("2", "token"));
    let new_owner = HumanAddr::from("friend");
    let msg = HandleMsg::ChangeOwner {
        owner: new_owner.clone(),
    };
    let res = handle(&mut deps, env, msg).unwrap();

    // should change state
    assert_eq!(0, res.messages.len());
    let res = query(&mut deps, QueryMsg::Owner {}).unwrap();
    let value: OwnerResponse = from_slice(res.as_slice()).unwrap();
    assert_eq!("friend", value.owner.as_str());
}

#[test]
fn transfer_requires_owner() {
    let mut deps = mock_instance(WASM);

    let msg = InitMsg {};
    let env = mock_env(
        &deps.api,
        "creator",
        &coin("2", "token"),
        &coin("2", "token"),
    );
    let _res = init(&mut deps, env, msg).unwrap();

    let env = mock_env(&deps.api, "random", &[], &coin("2", "token"));
    let new_owner = HumanAddr::from("friend");
    let msg = HandleMsg::ChangeOwner {
        owner: new_owner.clone(),
    };

    let res = handle(&mut deps, env, msg);
    match res {
        ContractResult::Err(msg) => assert_eq!(msg, "Unauthorized"),
        _ => panic!("Must return unauthorized error"),
    }
}
