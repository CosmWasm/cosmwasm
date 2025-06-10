//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.

use cosmwasm_std::{
    assert_approx_eq, coins, to_json_vec, Addr, BankMsg, Binary, ContractResult, Empty,
    MigrateInfo, Response, SubMsg,
};
use cosmwasm_vm::{
    call_execute, from_slice,
    testing::{
        execute, instantiate, migrate_with_info, mock_env, mock_info, mock_instance, query, sudo,
        test_io, MockApi, MOCK_CONTRACT_ADDR,
    },
    Storage, VmError,
};

use hackatom::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, SudoMsg};
use hackatom::state::{State, CONFIG_KEY};

static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/hackatom.wasm");

const DESERIALIZATION_LIMIT: usize = 20_000;

fn make_init_msg(api: &MockApi) -> (InstantiateMsg, String) {
    let verifier = api.addr_make("verifies");
    let beneficiary = api.addr_make("benefits");
    let creator = api.addr_make("creator");
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
    assert_eq!(deps.required_capabilities().len(), 7);

    let verifier = deps.api().addr_make("verifies");
    let beneficiary = deps.api().addr_make("benefits");
    let creator = deps.api().addr_make("creator");
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

    let verifier = deps.api().addr_make("verifies");
    let beneficiary = deps.api().addr_make("benefits");
    let creator = deps.api().addr_make("creator");
    let msg = InstantiateMsg {
        verifier: verifier.clone(),
        beneficiary,
    };
    let info = mock_info(&creator, &coins(1000, "earth"));
    let res: Response = instantiate(&mut deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // now let's query
    let query_response = query(&mut deps, mock_env(), QueryMsg::Verifier {}).unwrap();
    assert_eq!(
        query_response,
        format!("{{\"verifier\":\"{verifier}\"}}").as_bytes()
    );

    // bad query returns parse error (pass wrong type - this connection is not enforced)
    let qres = query(&mut deps, mock_env(), ExecuteMsg::Panic {});
    let msg = qres.unwrap_err();
    assert!(msg.contains("Error parsing"));
}

#[test]
fn migrate_verifier() {
    let mut deps = mock_instance(WASM, &[]);

    let verifier = deps.api().addr_make("verifies");
    let beneficiary = deps.api().addr_make("benefits");
    let creator = deps.api().addr_make("creator");
    let msg = InstantiateMsg {
        verifier: verifier.clone(),
        beneficiary,
    };
    let info = mock_info(&creator, &[]);
    let res: Response = instantiate(&mut deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // check it is 'verifies'
    let query_response = query(&mut deps, mock_env(), QueryMsg::Verifier {}).unwrap();
    assert_eq!(
        query_response,
        format!("{{\"verifier\":\"{verifier}\"}}").as_bytes()
    );

    // change the verifier via migrate
    let someone_else = deps.api().addr_make("someone else");
    let msg = MigrateMsg {
        verifier: someone_else.clone(),
    };
    let migrate_info = MigrateInfo {
        sender: Addr::unchecked(creator),
        old_migrate_version: None,
    };
    let res: Response = migrate_with_info(&mut deps, mock_env(), msg, migrate_info).unwrap();
    assert_eq!(0, res.messages.len());

    // check it is 'someone else'
    let query_response = query(&mut deps, mock_env(), QueryMsg::Verifier {}).unwrap();
    assert_eq!(
        query_response,
        format!("{{\"verifier\":\"{someone_else}\"}}").as_bytes()
    );
}

#[test]
fn sudo_can_steal_tokens() {
    let mut deps = mock_instance(WASM, &[]);

    let verifier = deps.api().addr_make("verifies");
    let beneficiary = deps.api().addr_make("benefits");
    let creator = deps.api().addr_make("creator");
    let msg = InstantiateMsg {
        verifier,
        beneficiary,
    };
    let info = mock_info(&creator, &[]);
    let res: Response = instantiate(&mut deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // sudo takes any tax it wants
    let to_address = deps.api().addr_make("community-pool");
    let amount = coins(700, "gold");
    let sys_msg = SudoMsg::StealFunds {
        recipient: to_address.clone(),
        amount: amount.clone(),
    };
    let res: Response = sudo(&mut deps, mock_env(), sys_msg).unwrap();
    assert_eq!(1, res.messages.len());
    let msg = res.messages.first().expect("no message");
    assert_eq!(msg, &SubMsg::new(BankMsg::Send { to_address, amount }));
}

#[test]
fn fails_on_bad_init() {
    let mut deps = mock_instance(WASM, &[]);
    let info = mock_info("creator", &coins(1000, "earth"));
    // bad init returns parse error (pass wrong type - this connection is not enforced)
    let res: ContractResult<Response> =
        instantiate(&mut deps, mock_env(), info, ExecuteMsg::Panic {});
    let msg = res.unwrap_err();
    assert!(msg.contains("Error parsing"));
}

#[test]
fn execute_release_works() {
    let mut deps = mock_instance(WASM, &[]);

    // initialize the store
    let creator = deps.api().addr_make("creator");
    let verifier = deps.api().addr_make("verifies");
    let beneficiary = deps.api().addr_make("benefits");

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
    let execute_res: Response = execute(
        &mut deps,
        mock_env(),
        execute_info,
        ExecuteMsg::Release {
            denom: "earth".to_string(),
        },
    )
    .unwrap();
    assert_eq!(execute_res.messages.len(), 1);
    let msg = execute_res.messages.first().expect("no message");
    assert_eq!(
        msg,
        &SubMsg::new(BankMsg::Send {
            to_address: beneficiary.clone(),
            amount: coins(1000, "earth"),
        }),
    );
    assert_eq!(
        execute_res.attributes,
        vec![("action", "release"), ("destination", beneficiary.as_str())],
    );
    assert_eq!(execute_res.data, Some(vec![0xF0, 0x0B, 0xAA].into()));
}

#[test]
fn execute_release_fails_for_wrong_sender() {
    let mut deps = mock_instance(WASM, &[]);

    // initialize the store
    let creator = deps.api().addr_make("creator");
    let verifier = deps.api().addr_make("verifies");
    let beneficiary = deps.api().addr_make("benefits");

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
    let execute_res: ContractResult<Response> = execute(
        &mut deps,
        mock_env(),
        execute_info,
        ExecuteMsg::Release {
            denom: "earth".to_string(),
        },
    );
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
fn execute_cpu_loop() {
    let mut deps = mock_instance(WASM, &[]);

    let (instantiate_msg, creator) = make_init_msg(deps.api());
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
        &to_json_vec(&ExecuteMsg::CpuLoop {}).unwrap(),
    );
    assert!(execute_res.is_err());
    assert_eq!(deps.get_gas_left(), 0);
}

#[test]
fn execute_storage_loop() {
    let mut deps = mock_instance(WASM, &[]);

    let (instantiate_msg, creator) = make_init_msg(deps.api());
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
        &to_json_vec(&ExecuteMsg::StorageLoop {}).unwrap(),
    );
    assert!(execute_res.is_err());
    assert_eq!(deps.get_gas_left(), 0);
}

#[test]
fn execute_memory_loop() {
    let mut deps = mock_instance(WASM, &[]);

    let (instantiate_msg, creator) = make_init_msg(deps.api());
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
        &to_json_vec(&ExecuteMsg::MemoryLoop {}).unwrap(),
    );
    assert!(execute_res.is_err());
    assert_eq!(deps.get_gas_left(), 0);

    // Ran out of gas before consuming a significant amount of memory
    assert!(deps.memory_pages() < 200);
}

#[test]
fn execute_allocate_large_memory() {
    let mut deps = mock_instance(WASM, &[]);

    let (instantiate_msg, creator) = make_init_msg(deps.api());
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
    assert_approx_eq!(gas_used, 9470400, "0.2");
    let used = deps.memory_pages();
    assert_eq!(used, pages_before + 48, "Memory used: {used} pages");
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
    assert_approx_eq!(gas_used, 8623090, "0.2");
    let used = deps.memory_pages();
    assert_eq!(used, pages_before, "Memory used: {used} pages");
}

#[test]
fn execute_panic() {
    let mut deps = mock_instance(WASM, &[]);

    let (instantiate_msg, creator) = make_init_msg(deps.api());
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
        &to_json_vec(&ExecuteMsg::Panic {}).unwrap(),
    );
    match execute_res.unwrap_err() {
        VmError::RuntimeErr { msg, .. } => {
            assert!(
                msg.contains("Aborted: panicked")
                    && msg.contains("This page intentionally faulted"),
                "Must contain panic message"
            );
            assert!(msg.contains("contract.rs:"), "Must contain file and line");
        }
        err => panic!("Unexpected error: {err:?}"),
    }
}

#[test]
fn execute_user_errors_in_api_calls() {
    let mut deps = mock_instance(WASM, &[]);

    let (instantiate_msg, creator) = make_init_msg(deps.api());
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
