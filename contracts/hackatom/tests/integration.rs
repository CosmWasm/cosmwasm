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
    coins, from_binary, to_vec, Addr, AllBalanceResponse, BankMsg, Binary, ContractResult, Empty,
    Response, SubMsg,
};
use cosmwasm_vm::{
    call_execute, from_slice,
    testing::{
        execute, instantiate, migrate, mock_env, mock_info, mock_instance,
        mock_instance_with_balances, mock_instance_with_gas_limit, query, sudo, test_io,
        MOCK_CONTRACT_ADDR,
    },
    Storage, VmError,
};

use hackatom::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, SudoMsg};
use hackatom::state::{State, CONFIG_KEY};

static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/hackatom.wasm");

const DESERIALIZATION_LIMIT: usize = 20_000;

fn make_init_msg() -> (InstantiateMsg, String) {
    let verifier = String::from("verifies");
    let beneficiary = String::from("benefits");
    let creator = String::from("creator");
    (
        InstantiateMsg {
            verifier,
            beneficiary,
        },
        creator,
    )
}

#[test]
fn proper_initialization() {
    let mut deps = mock_instance(WASM, &[]);
    assert_eq!(deps.required_features().len(), 0);

    let verifier = String::from("verifies");
    let beneficiary = String::from("benefits");
    let creator = String::from("creator");
    let expected_state = State {
        verifier: Addr::unchecked(&verifier),
        beneficiary: Addr::unchecked(&beneficiary),
        funder: Addr::unchecked(&creator),
    };

    let msg = InstantiateMsg {
        verifier,
        beneficiary,
    };
    let info = mock_info(&creator, &coins(1000, "earth"));
    let res: Response = instantiate(&mut deps, mock_env(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);
    assert_eq!(res.attributes, [("Let the", "hacking begin")]);

    // it worked, let's check the state
    let state: State = deps
        .with_storage(|store| {
            let data = store
                .get(CONFIG_KEY)
                .0
                .expect("error reading db")
                .expect("no data stored");
            from_slice(&data, DESERIALIZATION_LIMIT)
        })
        .unwrap();
    assert_eq!(state, expected_state);
}

#[test]
fn instantiate_and_query() {
    let mut deps = mock_instance(WASM, &[]);

    let verifier = String::from("verifies");
    let beneficiary = String::from("benefits");
    let creator = String::from("creator");
    let msg = InstantiateMsg {
        verifier,
        beneficiary,
    };
    let info = mock_info(&creator, &coins(1000, "earth"));
    let res: Response = instantiate(&mut deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // now let's query
    let query_response = query(&mut deps, mock_env(), QueryMsg::Verifier {}).unwrap();
    assert_eq!(query_response.as_slice(), b"{\"verifier\":\"verifies\"}");

    // bad query returns parse error (pass wrong type - this connection is not enforced)
    let qres = query(&mut deps, mock_env(), ExecuteMsg::Release {});
    let msg = qres.unwrap_err();
    assert!(msg.contains("Error parsing"));
}

#[test]
fn migrate_verifier() {
    let mut deps = mock_instance(WASM, &[]);

    let verifier = String::from("verifies");
    let beneficiary = String::from("benefits");
    let creator = String::from("creator");
    let msg = InstantiateMsg {
        verifier,
        beneficiary,
    };
    let info = mock_info(&creator, &[]);
    let res: Response = instantiate(&mut deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // check it is 'verifies'
    let query_response = query(&mut deps, mock_env(), QueryMsg::Verifier {}).unwrap();
    assert_eq!(query_response.as_slice(), b"{\"verifier\":\"verifies\"}");

    // change the verifier via migrate
    let msg = MigrateMsg {
        verifier: String::from("someone else"),
    };
    let res: Response = migrate(&mut deps, mock_env(), msg).unwrap();
    assert_eq!(0, res.messages.len());

    // check it is 'someone else'
    let query_response = query(&mut deps, mock_env(), QueryMsg::Verifier {}).unwrap();
    assert_eq!(
        query_response.as_slice(),
        b"{\"verifier\":\"someone else\"}"
    );
}

#[test]
fn sudo_can_steal_tokens() {
    let mut deps = mock_instance(WASM, &[]);

    let verifier = String::from("verifies");
    let beneficiary = String::from("benefits");
    let creator = String::from("creator");
    let msg = InstantiateMsg {
        verifier,
        beneficiary,
    };
    let info = mock_info(&creator, &[]);
    let res: Response = instantiate(&mut deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // sudo takes any tax it wants
    let to_address = String::from("community-pool");
    let amount = coins(700, "gold");
    let sys_msg = SudoMsg::StealFunds {
        recipient: to_address.clone(),
        amount: amount.clone(),
    };
    let res: Response = sudo(&mut deps, mock_env(), sys_msg).unwrap();
    assert_eq!(1, res.messages.len());
    let msg = res.messages.get(0).expect("no message");
    assert_eq!(msg, &SubMsg::new(BankMsg::Send { to_address, amount }));
}

#[test]
fn querier_callbacks_work() {
    let rich_addr = String::from("foobar");
    let rich_balance = coins(10000, "gold");
    let mut deps = mock_instance_with_balances(WASM, &[(&rich_addr, &rich_balance)]);

    // querying with balance gets the balance
    let query_msg = QueryMsg::OtherBalance { address: rich_addr };
    let query_response = query(&mut deps, mock_env(), query_msg).unwrap();
    let bal: AllBalanceResponse = from_binary(&query_response).unwrap();
    assert_eq!(bal.amount, rich_balance);

    // querying other accounts gets none
    let query_msg = QueryMsg::OtherBalance {
        address: String::from("someone else"),
    };
    let query_response = query(&mut deps, mock_env(), query_msg).unwrap();
    let bal: AllBalanceResponse = from_binary(&query_response).unwrap();
    assert_eq!(bal.amount, vec![]);
}

#[test]
fn fails_on_bad_init() {
    let mut deps = mock_instance(WASM, &[]);
    let info = mock_info("creator", &coins(1000, "earth"));
    // bad init returns parse error (pass wrong type - this connection is not enforced)
    let res: ContractResult<Response> =
        instantiate(&mut deps, mock_env(), info, ExecuteMsg::Release {});
    let msg = res.unwrap_err();
    assert!(msg.contains("Error parsing"));
}

#[test]
fn execute_release_works() {
    let mut deps = mock_instance(WASM, &[]);

    // initialize the store
    let creator = String::from("creator");
    let verifier = String::from("verifies");
    let beneficiary = String::from("benefits");

    let instantiate_msg = InstantiateMsg {
        verifier: verifier.clone(),
        beneficiary: beneficiary.clone(),
    };
    let init_amount = coins(1000, "earth");
    let init_info = mock_info(&creator, &init_amount);
    let init_res: Response =
        instantiate(&mut deps, mock_env(), init_info, instantiate_msg).unwrap();
    assert_eq!(init_res.messages.len(), 0);

    // balance changed in init
    deps.with_querier(|querier| {
        querier.update_balance(MOCK_CONTRACT_ADDR, init_amount);
        Ok(())
    })
    .unwrap();

    // beneficiary can release it
    let execute_info = mock_info(&verifier, &[]);
    let execute_res: Response =
        execute(&mut deps, mock_env(), execute_info, ExecuteMsg::Release {}).unwrap();
    assert_eq!(execute_res.messages.len(), 1);
    let msg = execute_res.messages.get(0).expect("no message");
    assert_eq!(
        msg,
        &SubMsg::new(BankMsg::Send {
            to_address: beneficiary,
            amount: coins(1000, "earth"),
        }),
    );
    assert_eq!(
        execute_res.attributes,
        vec![("action", "release"), ("destination", "benefits")],
    );
    assert_eq!(execute_res.data, Some(vec![0xF0, 0x0B, 0xAA].into()));
}

#[test]
fn execute_release_fails_for_wrong_sender() {
    let mut deps = mock_instance(WASM, &[]);

    // initialize the store
    let creator = String::from("creator");
    let verifier = String::from("verifies");
    let beneficiary = String::from("benefits");

    let instantiate_msg = InstantiateMsg {
        verifier: verifier.clone(),
        beneficiary: beneficiary.clone(),
    };
    let init_amount = coins(1000, "earth");
    let init_info = mock_info(&creator, &init_amount);
    let init_res: Response =
        instantiate(&mut deps, mock_env(), init_info, instantiate_msg).unwrap();
    assert_eq!(init_res.messages.len(), 0);

    // balance changed in init
    deps.with_querier(|querier| {
        querier.update_balance(MOCK_CONTRACT_ADDR, init_amount);
        Ok(())
    })
    .unwrap();

    // beneficiary cannot release it
    let execute_info = mock_info(&beneficiary, &[]);
    let execute_res: ContractResult<Response> =
        execute(&mut deps, mock_env(), execute_info, ExecuteMsg::Release {});
    let msg = execute_res.unwrap_err();
    assert!(msg.contains("Unauthorized"));

    // state should not change
    let data = deps
        .with_storage(|store| {
            Ok(store
                .get(CONFIG_KEY)
                .0
                .expect("error reading db")
                .expect("no data stored"))
        })
        .unwrap();
    let state: State = from_slice(&data, DESERIALIZATION_LIMIT).unwrap();
    assert_eq!(
        state,
        State {
            verifier: Addr::unchecked(&verifier),
            beneficiary: Addr::unchecked(&beneficiary),
            funder: Addr::unchecked(&creator),
        }
    );
}

#[test]
fn execute_argon2() {
    let mut deps = mock_instance_with_gas_limit(WASM, 100_000_000_000_000);

    let (instantiate_msg, creator) = make_init_msg();
    let init_info = mock_info(creator.as_str(), &[]);
    let init_res: Response =
        instantiate(&mut deps, mock_env(), init_info, instantiate_msg).unwrap();
    assert_eq!(0, init_res.messages.len());

    let gas_before = deps.get_gas_left();
    let _execute_res: Response = execute(
        &mut deps,
        mock_env(),
        mock_info(creator.as_str(), &[]),
        ExecuteMsg::Argon2 {
            mem_cost: 256,
            time_cost: 5,
        },
    )
    .unwrap();
    let gas_used = gas_before - deps.get_gas_left();
    // Note: the exact gas usage depends on the Rust version used to compile Wasm,
    // which we only fix when using rust-optimizer, not integration tests.
    let expected = 15428758650000; // +/- 20%
    assert!(gas_used > expected * 80 / 100, "Gas used: {}", gas_used);
    assert!(gas_used < expected * 120 / 100, "Gas used: {}", gas_used);
}

#[test]
fn execute_cpu_loop() {
    let mut deps = mock_instance(WASM, &[]);

    let (instantiate_msg, creator) = make_init_msg();
    let init_info = mock_info(creator.as_str(), &[]);
    let init_res: Response =
        instantiate(&mut deps, mock_env(), init_info, instantiate_msg).unwrap();
    assert_eq!(0, init_res.messages.len());

    let execute_info = mock_info(creator.as_str(), &[]);
    // Note: we need to use the production-call, not the testing call (which unwraps any vm error)
    let execute_res = call_execute::<_, _, _, Empty>(
        &mut deps,
        &mock_env(),
        &execute_info,
        &to_vec(&ExecuteMsg::CpuLoop {}).unwrap(),
    );
    assert!(execute_res.is_err());
    assert_eq!(deps.get_gas_left(), 0);
}

#[test]
fn execute_storage_loop() {
    let mut deps = mock_instance(WASM, &[]);

    let (instantiate_msg, creator) = make_init_msg();
    let init_info = mock_info(creator.as_str(), &[]);
    let init_res: Response =
        instantiate(&mut deps, mock_env(), init_info, instantiate_msg).unwrap();
    assert_eq!(0, init_res.messages.len());

    let execute_info = mock_info(creator.as_str(), &[]);
    // Note: we need to use the production-call, not the testing call (which unwraps any vm error)
    let execute_res = call_execute::<_, _, _, Empty>(
        &mut deps,
        &mock_env(),
        &execute_info,
        &to_vec(&ExecuteMsg::StorageLoop {}).unwrap(),
    );
    assert!(execute_res.is_err());
    assert_eq!(deps.get_gas_left(), 0);
}

#[test]
fn execute_memory_loop() {
    let mut deps = mock_instance(WASM, &[]);

    let (instantiate_msg, creator) = make_init_msg();
    let init_info = mock_info(creator.as_str(), &[]);
    let init_res: Response =
        instantiate(&mut deps, mock_env(), init_info, instantiate_msg).unwrap();
    assert_eq!(0, init_res.messages.len());

    let execute_info = mock_info(creator.as_str(), &[]);
    // Note: we need to use the production-call, not the testing call (which unwraps any vm error)
    let execute_res = call_execute::<_, _, _, Empty>(
        &mut deps,
        &mock_env(),
        &execute_info,
        &to_vec(&ExecuteMsg::MemoryLoop {}).unwrap(),
    );
    assert!(execute_res.is_err());
    assert_eq!(deps.get_gas_left(), 0);

    // Ran out of gas before consuming a significant amount of memory
    assert!(deps.memory_pages() < 200);
}

#[test]
fn execute_allocate_large_memory() {
    let mut deps = mock_instance(WASM, &[]);

    let (instantiate_msg, creator) = make_init_msg();
    let init_info = mock_info(creator.as_str(), &[]);
    let init_res: Response =
        instantiate(&mut deps, mock_env(), init_info, instantiate_msg).unwrap();
    assert_eq!(0, init_res.messages.len());
    let mut pages_before = deps.memory_pages();
    assert_eq!(pages_before, 18);

    // Grow by 48 pages (3 MiB)
    let execute_info = mock_info(creator.as_str(), &[]);
    let gas_before = deps.get_gas_left();
    let execute_res: Response = execute(
        &mut deps,
        mock_env(),
        execute_info,
        ExecuteMsg::AllocateLargeMemory { pages: 48 },
    )
    .unwrap();
    assert_eq!(
        execute_res.data.unwrap(),
        Binary::from((pages_before as u32).to_be_bytes())
    );
    let gas_used = gas_before - deps.get_gas_left();
    // Gas consumption is relatively small
    // Note: the exact gas usage depends on the Rust version used to compile Wasm,
    // which we only fix when using rust-optimizer, not integration tests.
    let expected = 4413600000; // +/- 20%
    assert!(gas_used > expected * 80 / 100, "Gas used: {}", gas_used);
    assert!(gas_used < expected * 120 / 100, "Gas used: {}", gas_used);
    let used = deps.memory_pages();
    assert_eq!(used, pages_before + 48, "Memory used: {} pages", used);
    pages_before += 48;

    // Grow by 1600 pages (100 MiB)
    let execute_info = mock_info(creator.as_str(), &[]);
    let gas_before = deps.get_gas_left();
    let result: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        execute_info,
        ExecuteMsg::AllocateLargeMemory { pages: 1600 },
    );
    assert_eq!(result.unwrap_err(), "Generic error: memory.grow failed");
    let gas_used = gas_before - deps.get_gas_left();
    // Gas consumption is relatively small
    // Note: the exact gas usage depends on the Rust version used to compile Wasm,
    // which we only fix when using rust-optimizer, not integration tests.
    let expected = 4859700000; // +/- 20%
    assert!(gas_used > expected * 80 / 100, "Gas used: {}", gas_used);
    assert!(gas_used < expected * 120 / 100, "Gas used: {}", gas_used);
    let used = deps.memory_pages();
    assert_eq!(used, pages_before, "Memory used: {} pages", used);
}

#[test]
fn execute_panic() {
    let mut deps = mock_instance(WASM, &[]);

    let (instantiate_msg, creator) = make_init_msg();
    let init_info = mock_info(creator.as_str(), &[]);
    let init_res: Response =
        instantiate(&mut deps, mock_env(), init_info, instantiate_msg).unwrap();
    assert_eq!(0, init_res.messages.len());

    let execute_info = mock_info(creator.as_str(), &[]);
    // panic inside contract should not panic out here
    // Note: we need to use the production-call, not the testing call (which unwraps any vm error)
    let execute_res = call_execute::<_, _, _, Empty>(
        &mut deps,
        &mock_env(),
        &execute_info,
        &to_vec(&ExecuteMsg::Panic {}).unwrap(),
    );
    match execute_res.unwrap_err() {
        VmError::RuntimeErr { msg, .. } => {
            assert!(
                msg.contains("Aborted: panicked at 'This page intentionally faulted'"),
                "Must contain panic message"
            );
            assert!(msg.contains("contract.rs:"), "Must contain file and line");
        }
        err => panic!("Unexpected error: {:?}", err),
    }
}

#[test]
fn execute_user_errors_in_api_calls() {
    let mut deps = mock_instance(WASM, &[]);

    let (instantiate_msg, creator) = make_init_msg();
    let init_info = mock_info(creator.as_str(), &[]);
    let _init_res: Response =
        instantiate(&mut deps, mock_env(), init_info, instantiate_msg).unwrap();

    let execute_info = mock_info(creator.as_str(), &[]);
    let _execute_res: Response = execute(
        &mut deps,
        mock_env(),
        execute_info,
        ExecuteMsg::UserErrorsInApiCalls {},
    )
    .unwrap();
}

#[test]
fn passes_io_tests() {
    let mut deps = mock_instance(WASM, &[]);
    test_io(&mut deps);
}
