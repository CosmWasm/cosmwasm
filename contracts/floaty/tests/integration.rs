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
    attr, coins, from_binary, Addr, AllBalanceResponse, BankMsg, ContractResult, Response, SubMsg,
};
use cosmwasm_vm::{
    from_slice,
    testing::{
        execute, instantiate, mock_env, mock_info, mock_instance, mock_instance_with_balances,
        query, test_io, MOCK_CONTRACT_ADDR,
    },
    Storage,
};

use floaty::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use floaty::state::{State, CONFIG_KEY};

static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/floaty.wasm");

const DESERIALIZATION_LIMIT: usize = 20_000;

#[test]
fn proper_initialization() {
    let mut deps = mock_instance(WASM, &[]);
    assert_eq!(deps.required_features.len(), 0);

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
    assert_eq!(res.attributes.len(), 1);
    assert_eq!(res.attributes[0].key, "Let the");
    assert_eq!(res.attributes[0].value, "hacking begin");

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
        vec![
            attr("action", "release"),
            attr("destination", "benefits"),
            attr("foo", "300")
        ],
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
fn passes_io_tests() {
    let mut deps = mock_instance(WASM, &[]);
    test_io(&mut deps);
}
