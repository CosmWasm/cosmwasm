//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.
//!
//! You can easily convert unit tests to integration tests as follows:
//! 1. Copy them over verbatim
//! 2. Then change
//!      let mut deps = mock_dependencies(20, &[]);
//!    to
//!      let mut deps = mock_instance(WASM, &[]);
//! 3. If you access raw storage, where ever you see something like:
//!      deps.storage.get(CONFIG_KEY).expect("no data stored");
//!    replace it with:
//!      deps.with_storage(|store| {
//!          let data = store.get(CONFIG_KEY).expect("no data stored");
//!          //...
//!      });
//! 4. Anywhere you see query(&deps, ...) you must replace it with query(&mut deps, ...)

use cosmwasm_std::{
    coins, from_binary, log, AllBalanceResponse, BankMsg, HandleResponse, HandleResult, HumanAddr,
    InitResponse, InitResult, MigrateResponse, StdError,
};
use cosmwasm_vm::{
    from_slice,
    testing::{
        handle, init, migrate, mock_env, mock_instance, mock_instance_with_balances, query,
        test_io, MOCK_CONTRACT_ADDR,
    },
    Api, BackendStorage,
};

use hackatom::contract::{HandleMsg, InitMsg, MigrateMsg, QueryMsg, State, CONFIG_KEY};

static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/hackatom.wasm");

#[test]
fn proper_initialization() {
    let mut deps = mock_instance(WASM, &[]);
    assert_eq!(deps.required_features.len(), 0);

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
    let env = mock_env(&deps.api, "creator", &coins(1000, "earth"));
    let res: InitResponse = init(&mut deps, env, msg).unwrap();
    assert_eq!(res.messages.len(), 0);
    assert_eq!(res.log.len(), 1);
    assert_eq!(res.log[0].key, "Let the");
    assert_eq!(res.log[0].value, "hacking begin");

    // it worked, let's check the state
    let state: State = deps
        .with_storage(|store| {
            let data = store
                .get(CONFIG_KEY)
                .expect("error reading db")
                .0
                .expect("no data stored");
            from_slice(&data)
        })
        .unwrap();
    assert_eq!(state, expected_state);
}

#[test]
fn init_and_query() {
    let mut deps = mock_instance(WASM, &[]);

    let verifier = HumanAddr(String::from("verifies"));
    let beneficiary = HumanAddr(String::from("benefits"));
    let creator = HumanAddr(String::from("creator"));
    let msg = InitMsg {
        verifier: verifier.clone(),
        beneficiary,
    };
    let env = mock_env(&deps.api, creator.as_str(), &coins(1000, "earth"));
    let res: InitResponse = init(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // now let's query
    let query_response = query(&mut deps, QueryMsg::Verifier {}).unwrap();
    assert_eq!(query_response.as_slice(), b"{\"verifier\":\"verifies\"}");

    // bad query returns parse error (pass wrong type - this connection is not enforced)
    let qres = query(&mut deps, HandleMsg::Release {});
    match qres.unwrap_err() {
        StdError::ParseErr { .. } => {}
        _ => panic!("Expected parse error"),
    }
}

#[test]
fn migrate_verifier() {
    let mut deps = mock_instance(WASM, &[]);

    let verifier = HumanAddr::from("verifies");
    let beneficiary = HumanAddr::from("benefits");
    let creator = HumanAddr::from("creator");
    let msg = InitMsg {
        verifier: verifier.clone(),
        beneficiary,
    };
    let env = mock_env(&deps.api, creator.as_str(), &[]);
    let res: InitResponse = init(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // check it is 'verifies'
    let query_response = query(&mut deps, QueryMsg::Verifier {}).unwrap();
    assert_eq!(query_response.as_slice(), b"{\"verifier\":\"verifies\"}");

    // change the verifier via migrate
    let msg = MigrateMsg {
        verifier: HumanAddr::from("someone else"),
    };
    let env = mock_env(&deps.api, creator.as_str(), &[]);
    let res: MigrateResponse = migrate(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // check it is 'someone else'
    let query_response = query(&mut deps, QueryMsg::Verifier {}).unwrap();
    assert_eq!(
        query_response.as_slice(),
        b"{\"verifier\":\"someone else\"}"
    );
}

#[test]
fn querier_callbacks_work() {
    let rich_addr = HumanAddr::from("foobar");
    let rich_balance = coins(10000, "gold");
    let mut deps = mock_instance_with_balances(WASM, &[(&rich_addr, &rich_balance)]);

    // querying with balance gets the balance
    let query_msg = QueryMsg::OtherBalance { address: rich_addr };
    let query_response = query(&mut deps, query_msg).unwrap();
    let bal: AllBalanceResponse = from_binary(&query_response).unwrap();
    assert_eq!(bal.amount, rich_balance);

    // querying other accounts gets none
    let query_msg = QueryMsg::OtherBalance {
        address: HumanAddr::from("someone else"),
    };
    let query_response = query(&mut deps, query_msg).unwrap();
    let bal: AllBalanceResponse = from_binary(&query_response).unwrap();
    assert_eq!(bal.amount, vec![]);
}

#[test]
fn fails_on_bad_init() {
    let mut deps = mock_instance(WASM, &[]);
    let env = mock_env(&deps.api, "creator", &coins(1000, "earth"));
    // bad init returns parse error (pass wrong type - this connection is not enforced)
    let res: InitResult = init(&mut deps, env, HandleMsg::Release {});
    match res.unwrap_err() {
        StdError::ParseErr { .. } => {}
        _ => panic!("Expected parse error"),
    }
}

#[test]
fn handle_release_works() {
    let mut deps = mock_instance(WASM, &[]);

    // initialize the store
    let creator = HumanAddr::from("creator");
    let verifier = HumanAddr::from("verifies");
    let beneficiary = HumanAddr::from("benefits");

    let init_msg = InitMsg {
        verifier: verifier.clone(),
        beneficiary: beneficiary.clone(),
    };
    let init_amount = coins(1000, "earth");
    let init_env = mock_env(&deps.api, creator.as_str(), &init_amount);
    let contract_addr = deps.api.human_address(&init_env.contract.address).unwrap();
    let init_res: InitResponse = init(&mut deps, init_env, init_msg).unwrap();
    assert_eq!(init_res.messages.len(), 0);

    // balance changed in init
    deps.with_querier(|querier| {
        querier.update_balance(&contract_addr, init_amount);
        Ok(())
    })
    .unwrap();

    // beneficiary can release it
    let handle_env = mock_env(&deps.api, verifier.as_str(), &[]);
    let handle_res: HandleResponse = handle(&mut deps, handle_env, HandleMsg::Release {}).unwrap();
    assert_eq!(handle_res.messages.len(), 1);
    let msg = handle_res.messages.get(0).expect("no message");
    assert_eq!(
        msg,
        &BankMsg::Send {
            from_address: HumanAddr::from(MOCK_CONTRACT_ADDR),
            to_address: beneficiary,
            amount: coins(1000, "earth"),
        }
        .into(),
    );
    assert_eq!(
        handle_res.log,
        vec![log("action", "release"), log("destination", "benefits"),],
    );
    assert_eq!(handle_res.data, Some(vec![0xF0, 0x0B, 0xAA].into()));
}

#[test]
fn handle_release_fails_for_wrong_sender() {
    let mut deps = mock_instance(WASM, &[]);

    // initialize the store
    let creator = HumanAddr::from("creator");
    let verifier = HumanAddr::from("verifies");
    let beneficiary = HumanAddr::from("benefits");

    let init_msg = InitMsg {
        verifier: verifier.clone(),
        beneficiary: beneficiary.clone(),
    };
    let init_amount = coins(1000, "earth");
    let init_env = mock_env(&deps.api, creator.as_str(), &init_amount);
    let contract_addr = deps.api.human_address(&init_env.contract.address).unwrap();
    let init_res: InitResponse = init(&mut deps, init_env, init_msg).unwrap();
    assert_eq!(init_res.messages.len(), 0);

    // balance changed in init
    deps.with_querier(|querier| {
        querier.update_balance(&contract_addr, init_amount);
        Ok(())
    })
    .unwrap();

    // beneficiary cannot release it
    let handle_env = mock_env(&deps.api, beneficiary.as_str(), &[]);
    let handle_res: HandleResult = handle(&mut deps, handle_env, HandleMsg::Release {});
    match handle_res.unwrap_err() {
        StdError::Unauthorized { .. } => {}
        _ => panic!("Expect unauthorized error"),
    }

    // state should not change
    let data = deps
        .with_storage(|store| {
            Ok(store
                .get(CONFIG_KEY)
                .expect("error reading db")
                .0
                .expect("no data stored"))
        })
        .unwrap();
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

#[test]
fn passes_io_tests() {
    let mut deps = mock_instance(WASM, &[]);
    test_io(&mut deps);
}

#[cfg(feature = "singlepass")]
mod singlepass_tests {
    use super::*;

    use cosmwasm_std::{to_vec, Never};
    use cosmwasm_vm::call_handle;

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
        let mut deps = mock_instance(WASM, &[]);

        let (init_msg, creator) = make_init_msg();
        let init_env = mock_env(&deps.api, creator.as_str(), &[]);
        let init_res: InitResponse = init(&mut deps, init_env, init_msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        let handle_env = mock_env(&deps.api, creator.as_str(), &[]);
        // panic inside contract should not panic out here
        // Note: we need to use the production-call, not the testing call (which unwraps any vm error)
        let handle_res = call_handle::<_, _, _, Never>(
            &mut deps,
            &handle_env,
            &to_vec(&HandleMsg::Panic {}).unwrap(),
        );
        assert!(handle_res.is_err());
    }

    #[test]
    fn handle_cpu_loop() {
        let mut deps = mock_instance(WASM, &[]);

        let (init_msg, creator) = make_init_msg();
        let init_env = mock_env(&deps.api, creator.as_str(), &[]);
        let init_res: InitResponse = init(&mut deps, init_env, init_msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        let handle_env = mock_env(&deps.api, creator.as_str(), &[]);
        // Note: we need to use the production-call, not the testing call (which unwraps any vm error)
        let handle_res = call_handle::<_, _, _, Never>(
            &mut deps,
            &handle_env,
            &to_vec(&HandleMsg::CpuLoop {}).unwrap(),
        );
        assert!(handle_res.is_err());
        assert_eq!(deps.get_gas_left(), 0);
    }

    #[test]
    fn handle_storage_loop() {
        let mut deps = mock_instance(WASM, &[]);

        let (init_msg, creator) = make_init_msg();
        let init_env = mock_env(&deps.api, creator.as_str(), &[]);
        let init_res: InitResponse = init(&mut deps, init_env, init_msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        let handle_env = mock_env(&deps.api, creator.as_str(), &[]);
        // Note: we need to use the production-call, not the testing call (which unwraps any vm error)
        let handle_res = call_handle::<_, _, _, Never>(
            &mut deps,
            &handle_env,
            &to_vec(&HandleMsg::StorageLoop {}).unwrap(),
        );
        assert!(handle_res.is_err());
        assert_eq!(deps.get_gas_left(), 0);
    }

    #[test]
    fn handle_memory_loop() {
        let mut deps = mock_instance(WASM, &[]);

        let (init_msg, creator) = make_init_msg();
        let init_env = mock_env(&deps.api, creator.as_str(), &[]);
        let init_res: InitResponse = init(&mut deps, init_env, init_msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        let handle_env = mock_env(&deps.api, creator.as_str(), &[]);
        // Note: we need to use the production-call, not the testing call (which unwraps any vm error)
        let handle_res = call_handle::<_, _, _, Never>(
            &mut deps,
            &handle_env,
            &to_vec(&HandleMsg::MemoryLoop {}).unwrap(),
        );
        assert!(handle_res.is_err());
        assert_eq!(deps.get_gas_left(), 0);

        // Ran out of gas before consuming a significant amount of memory
        assert!(deps.get_memory_size() < 2 * 1024 * 1024);
    }

    #[test]
    fn handle_allocate_large_memory() {
        let mut deps = mock_instance(WASM, &[]);

        let (init_msg, creator) = make_init_msg();
        let init_env = mock_env(&deps.api, creator.as_str(), &[]);
        let init_res: InitResponse = init(&mut deps, init_env, init_msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        let handle_env = mock_env(&deps.api, creator.as_str(), &[]);
        let gas_before = deps.get_gas_left();
        // Note: we need to use the production-call, not the testing call (which unwraps any vm error)
        let handle_res = call_handle::<_, _, _, Never>(
            &mut deps,
            &handle_env,
            &to_vec(&HandleMsg::AllocateLargeMemory {}).unwrap(),
        );
        let gas_used = gas_before - deps.get_gas_left();

        // TODO: this must fail, see https://github.com/CosmWasm/cosmwasm/issues/81
        assert_eq!(handle_res.is_err(), false);

        // Gas consumtion is relatively small
        // Note: the exact gas usage depends on the Rust version used to compile WASM,
        // which we only fix when using cosmwasm-opt, not integration tests.
        let expected = 42000; // +/- 20%
        assert!(gas_used > expected * 80 / 100, "Gas used: {}", gas_used);
        assert!(gas_used < expected * 120 / 100, "Gas used: {}", gas_used);

        // Used between 100 and 102 MiB of memory
        assert!(deps.get_memory_size() > 100 * 1024 * 1024);
        assert!(deps.get_memory_size() < 102 * 1024 * 1024);
    }
}
