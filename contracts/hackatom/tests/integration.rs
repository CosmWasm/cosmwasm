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
//! 5. When matching on error codes, you can not use Error types, but rather corresponding ApiError variants.
//!    Note that you don't have backtrace field and can often skip the .. filler:
//!      match res.unwrap_err() {
//!          Error::Unauthorized { .. } => {}
//!          _ => panic!("Must return unauthorized error"),
//!      }
//!    becomes:
//!      match res.unwrap_err() {
//!          ApiError::Unauthorized {} => {}
//!          _ => panic!("Must return unauthorized error"),
//!      }

use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{
    coins, from_binary, log, Api, ApiError, BalanceResponse, CosmosMsg, HumanAddr, ReadonlyStorage,
};
use cosmwasm_vm::from_slice;
use cosmwasm_vm::testing::{
    handle, init, mock_instance, mock_instance_with_balances, query, test_io,
};

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
    let env = mock_env(&deps.api, "creator", &coins(1000, "earth"), &[]);
    let res = init(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's check the state
    let state: State = deps
        .with_storage(|store| {
            let data = store
                .get(CONFIG_KEY)
                .expect("error reading db")
                .expect("no data stored");
            from_slice(&data)
        })
        .unwrap();
    assert_eq!(state, expected_state);
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
    let env = mock_env(&deps.api, creator.as_str(), &coins(1000, "earth"), &[]);
    let res = init(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // now let's query
    let query_response = query(&mut deps, QueryMsg::Verifier {}).unwrap();
    assert_eq!(query_response.as_slice(), b"{\"verifier\":\"verifies\"}");

    // bad query returns parse error (pass wrong type - this connection is not enforced)
    let qres = query(&mut deps, HandleMsg::Release {});
    match qres.unwrap_err() {
        ApiError::ParseErr { .. } => {}
        _ => panic!("Expected parse error"),
    }
}

#[test]
fn querier_callbacks_work() {
    let rich_addr = HumanAddr::from("foobar");
    let rich_balance = coins(10000, "gold");
    let mut deps = mock_instance_with_balances(WASM, &[(&rich_addr, &rich_balance)]);

    // querying with balance gets the balance
    let query_msg = QueryMsg::OtherBalance { address: rich_addr };
    let query_response = query(&mut deps, query_msg).unwrap();
    let bal: BalanceResponse = from_binary(&query_response).unwrap();
    assert_eq!(bal.amount, Some(rich_balance));

    // querying other accounts gets none
    let query_msg = QueryMsg::OtherBalance {
        address: HumanAddr::from("someone else"),
    };
    let query_response = query(&mut deps, query_msg).unwrap();
    let bal: BalanceResponse = from_binary(&query_response).unwrap();
    assert_eq!(bal.amount, None);
}

#[test]
fn fails_on_bad_init() {
    let mut deps = mock_instance(WASM);
    let env = mock_env(&deps.api, "creator", &coins(1000, "earth"), &[]);
    // bad init returns parse error (pass wrong type - this connection is not enforced)
    let res = init(&mut deps, env, HandleMsg::Release {});
    match res.unwrap_err() {
        ApiError::ParseErr { .. } => {}
        _ => panic!("Expected parse error"),
    }
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
        &coins(1000, "earth"),
        &coins(1000, "earth"),
    );
    let init_res = init(&mut deps, init_env, init_msg).unwrap();
    assert_eq!(0, init_res.messages.len());

    // beneficiary can release it
    let handle_env = mock_env(
        &deps.api,
        verifier.as_str(),
        &coins(15, "earth"),
        &coins(1015, "earth"),
    );
    let handle_res = handle(&mut deps, handle_env, HandleMsg::Release {}).unwrap();
    assert_eq!(1, handle_res.messages.len());
    let msg = handle_res.messages.get(0).expect("no message");
    assert_eq!(
        msg,
        &CosmosMsg::Send {
            from_address: HumanAddr("cosmos2contract".to_string()),
            to_address: beneficiary,
            amount: coins(1015, "earth"),
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
        &coins(1000, "earth"),
        &coins(1000, "earth"),
    );
    let init_res = init(&mut deps, init_env, init_msg).unwrap();
    assert_eq!(0, init_res.messages.len());

    // beneficiary cannot release it
    let handle_env = mock_env(&deps.api, beneficiary.as_str(), &[], &coins(1000, "earth"));
    let handle_res = handle(&mut deps, handle_env, HandleMsg::Release {});
    match handle_res.unwrap_err() {
        ApiError::Unauthorized {} => {}
        _ => panic!("Expect unauthorized error"),
    }

    // state should not change
    let state: State = deps
        .with_storage(|store| {
            let data = store
                .get(CONFIG_KEY)
                .expect("error reading db")
                .expect("no data stored");
            from_slice(&data)
        })
        .unwrap();
    assert_eq!(
        state,
        State {
            verifier: deps.api.canonical_address(&verifier).unwrap(),
            beneficiary: deps.api.canonical_address(&beneficiary).unwrap(),
            funder: deps.api.canonical_address(&creator).unwrap(),
        }
    );
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

    fn make_init_msg() -> (InitMsg, HumanAddr) {
        let verifier = HumanAddr::from("verifies");
        let beneficiary = HumanAddr::from("benefits");
        let creator = HumanAddr::from("creator");
        (
            InitMsg {
                verifier: verifier.clone(),
                beneficiary: beneficiary.clone(),
            },
            creator,
        )
    }

    #[test]
    fn handle_panic() {
        let mut deps = mock_instance(WASM);

        let (init_msg, creator) = make_init_msg();
        let init_env = mock_env(&deps.api, creator.as_str(), &[], &[]);
        let init_res = init(&mut deps, init_env, init_msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        let handle_env = mock_env(&deps.api, creator.as_str(), &[], &[]);
        // panic inside contract should not panic out here
        // Note: we need to use the production-call, not the testing call (which unwraps any vm error)
        let handle_res = call_handle(
            &mut deps,
            &handle_env,
            &to_vec(&HandleMsg::Panic {}).unwrap(),
        );
        assert!(handle_res.is_err());
    }

    #[test]
    fn handle_cpu_loop() {
        // Gas must be set so we die early on infinite loop
        let mut deps = mock_instance_with_gas_limit(WASM, &[], 1_000_000);

        let (init_msg, creator) = make_init_msg();
        let init_env = mock_env(&deps.api, creator.as_str(), &[], &[]);
        let init_res = init(&mut deps, init_env, init_msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        let handle_env = mock_env(&deps.api, creator.as_str(), &[], &[]);
        // Note: we need to use the production-call, not the testing call (which unwraps any vm error)
        let handle_res = call_handle(
            &mut deps,
            &handle_env,
            &to_vec(&HandleMsg::CpuLoop {}).unwrap(),
        );
        assert!(handle_res.is_err());
        assert_eq!(deps.get_gas(), 0);
    }

    #[test]
    fn handle_storage_loop() {
        // Gas must be set so we die early on infinite loop
        let mut deps = mock_instance_with_gas_limit(WASM, &[], 1_000_000);

        let (init_msg, creator) = make_init_msg();
        let init_env = mock_env(&deps.api, creator.as_str(), &[], &[]);
        let init_res = init(&mut deps, init_env, init_msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        let handle_env = mock_env(&deps.api, creator.as_str(), &[], &[]);
        // Note: we need to use the production-call, not the testing call (which unwraps any vm error)
        let handle_res = call_handle(
            &mut deps,
            &handle_env,
            &to_vec(&HandleMsg::StorageLoop {}).unwrap(),
        );
        assert!(handle_res.is_err());
        assert_eq!(deps.get_gas(), 0);
    }

    #[test]
    fn handle_memory_loop() {
        // Gas must be set so we die early on infinite loop
        let mut deps = mock_instance_with_gas_limit(WASM, &[], 1_000_000);

        let (init_msg, creator) = make_init_msg();
        let init_env = mock_env(&deps.api, creator.as_str(), &[], &[]);
        let init_res = init(&mut deps, init_env, init_msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        let handle_env = mock_env(&deps.api, creator.as_str(), &[], &[]);
        // Note: we need to use the production-call, not the testing call (which unwraps any vm error)
        let handle_res = call_handle(
            &mut deps,
            &handle_env,
            &to_vec(&HandleMsg::MemoryLoop {}).unwrap(),
        );
        assert!(handle_res.is_err());
        assert_eq!(deps.get_gas(), 0);
    }
}
