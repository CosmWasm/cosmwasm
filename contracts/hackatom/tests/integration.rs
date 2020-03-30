//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.
//!
//! You can easily convert unit tests to integration tests.
//! 1. First copy them over verbatum,
//! 2. Then change
//!      let mut deps = mock_dependencies(20);
//!    to
//!      let mut deps = mock_instance(WASM);
//! 3. If you access raw storage, where ever you see something like:
//!      deps.storage.get(CONFIG_KEY).expect("no data stored");
//!    replace it with:
//!      deps.with_storage(|store| {
//!          let data = store.get(CONFIG_KEY).expect("no data stored");
//!          //...
//!      });
//! 4. Anywhere you see query(&deps, ...) you must replace it with query(&mut deps, ...)
//! 5. When matching on error codes, you can not use Error types, but rather must use strings:
//!      match res {
//!          Err(Error::Unauthorized{..}) => {},
//!          _ => panic!("Must return unauthorized error"),
//!      }
//!    becomes:
//!      match res {
//!         ContractResult::Err(msg) => assert_eq!(msg, "Unauthorized"),
//!         _ => panic!("Expected error"),
//!      }

use std::str::from_utf8;

use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{coin, from_slice, log, Api, ApiError, CosmosMsg, HumanAddr, ReadonlyStorage};
use cosmwasm_vm::testing::{handle, init, mock_instance, query, test_io};

use hackatom::contract::{HandleMsg, InitMsg, QueryMsg, State, CONFIG_KEY};

static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/hackatom.wasm");

#[test]
fn proper_initialization() {
    let mut deps = mock_instance(WASM);
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
    let env = mock_env(&deps.api, "creator", &coin("1000", "earth"), &[]);
    let res = init(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's check the state
    deps.with_storage(|store| {
        let data = store.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_slice(&data).unwrap();
        assert_eq!(state, expected_state);
    });
}

#[test]
fn init_and_query() {
    let mut deps = mock_instance(WASM);

    let verifier = HumanAddr(String::from("verifies"));
    let beneficiary = HumanAddr(String::from("benefits"));
    let creator = HumanAddr(String::from("creator"));
    let msg = InitMsg {
        verifier: verifier.clone(),
        beneficiary,
    };
    let env = mock_env(&deps.api, creator.as_str(), &coin("1000", "earth"), &[]);
    let res = init(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // now let's query
    let qres = query(&mut deps, QueryMsg::Verifier {}).unwrap();
    let returned = from_utf8(qres.as_slice()).unwrap();
    assert_eq!(verifier.as_str(), returned);

    // bad query returns parse error (pass wrong type - this connection is not enforced)
    let qres = query(&mut deps, HandleMsg::Release {});
    match qres.unwrap_err() {
        ApiError::ParseErr { .. } => {}
        _ => panic!("Expected parse error"),
    }
}

#[test]
fn fails_on_bad_init() {
    let mut deps = mock_instance(WASM);
    let env = mock_env(&deps.api, "creator", &coin("1000", "earth"), &[]);
    // bad init returns parse error (pass wrong type - this connection is not enforced)
    let res = init(&mut deps, env, HandleMsg::Release {});
    assert_eq!(true, res.is_err());
}

#[test]
fn proper_handle() {
    let mut deps = mock_instance(WASM);

    // initialize the store
    let verifier = HumanAddr(String::from("verifies"));
    let beneficiary = HumanAddr(String::from("benefits"));

    let init_msg = InitMsg {
        verifier: verifier.clone(),
        beneficiary: beneficiary.clone(),
    };
    let init_env = mock_env(
        &deps.api,
        "creator",
        &coin("1000", "earth"),
        &coin("1000", "earth"),
    );
    let init_res = init(&mut deps, init_env, init_msg).unwrap();
    assert_eq!(0, init_res.messages.len());

    // beneficiary can release it
    let handle_env = mock_env(
        &deps.api,
        verifier.as_str(),
        &coin("15", "earth"),
        &coin("1015", "earth"),
    );
    let handle_res = handle(&mut deps, handle_env, HandleMsg::Release {}).unwrap();
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
        handle_res.log,
        vec![log("action", "release"), log("destination", "benefits"),],
    );
}

#[test]
fn failed_handle() {
    let mut deps = mock_instance(WASM);

    // initialize the store
    let verifier = HumanAddr(String::from("verifies"));
    let beneficiary = HumanAddr(String::from("benefits"));
    let creator = HumanAddr(String::from("creator"));

    let init_msg = InitMsg {
        verifier: verifier.clone(),
        beneficiary: beneficiary.clone(),
    };
    let init_env = mock_env(
        &deps.api,
        creator.as_str(),
        &coin("1000", "earth"),
        &coin("1000", "earth"),
    );
    let init_res = init(&mut deps, init_env, init_msg).unwrap();
    assert_eq!(0, init_res.messages.len());

    // beneficiary can release it
    let handle_env = mock_env(&deps.api, beneficiary.as_str(), &[], &coin("1000", "earth"));
    let handle_res = handle(&mut deps, handle_env, HandleMsg::Release {});
    assert!(handle_res.is_err());

    // state should not change
    deps.with_storage(|store| {
        let data = store.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_slice(&data).unwrap();
        assert_eq!(
            state,
            State {
                verifier: deps.api.canonical_address(&verifier).unwrap(),
                beneficiary: deps.api.canonical_address(&beneficiary).unwrap(),
                funder: deps.api.canonical_address(&creator).unwrap(),
            }
        );
    });
}

#[test]
fn passes_io_tests() {
    let mut deps = mock_instance(WASM);
    test_io(&mut deps);
}

#[cfg(feature = "singlepass")]
mod singlepass_tests {
    use super::*;

    use cosmwasm_std::to_vec;
    use cosmwasm_vm::call_handle;
    use cosmwasm_vm::testing::mock_instance_with_gas_limit;

    #[test]
    fn handle_panic_and_loops() {
        // Gas must be set so we die early on infinite loop
        let mut deps = mock_instance_with_gas_limit(WASM, 1_000_000);

        // initialize the store
        let verifier = HumanAddr(String::from("verifies"));
        let beneficiary = HumanAddr(String::from("benefits"));
        let creator = HumanAddr(String::from("creator"));

        let init_msg = InitMsg {
            verifier: verifier.clone(),
            beneficiary: beneficiary.clone(),
        };
        let init_env = mock_env(
            &deps.api,
            creator.as_str(),
            &coin("1000", "earth"),
            &coin("1000", "earth"),
        );
        let init_res = init(&mut deps, init_env, init_msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        // TRY PANIC
        let handle_env = mock_env(&deps.api, beneficiary.as_str(), &[], &coin("1000", "earth"));
        // panic inside contract should not panic out here
        // Note: we need to use the production-call, not the testing call (which unwraps any vm error)
        let handle_res = call_handle(
            &mut deps,
            &handle_env,
            &to_vec(&HandleMsg::Panic {}).unwrap(),
        );
        assert!(handle_res.is_err());

        // TRY INFINITE LOOP
        // Note: we need to use the production-call, not the testing call (which unwraps any vm error)
        let handle_res = call_handle(
            &mut deps,
            &handle_env,
            &to_vec(&HandleMsg::CpuLoop {}).unwrap(),
        );
        assert!(handle_res.is_err());
        assert_eq!(deps.get_gas(), 0);
    }
}
