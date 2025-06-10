//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.

use cosmwasm_std::{from_json, MessageInfo, Response};
use cosmwasm_vm::{
    testing::{
        execute, instantiate, migrate, mock_env, mock_info, mock_instance_with_gas_limit, query,
        MockApi, MockQuerier, MockStorage,
    },
    Instance,
};

use queue::msg::{
    CountResponse, ExecuteMsg, InstantiateMsg, ListResponse, MigrateMsg, QueryMsg, ReducerResponse,
    SumResponse,
};
use queue::state::Item;

static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/queue.wasm");

/// Instantiates a contract with no elements
fn create_contract() -> (Instance<MockApi, MockStorage, MockQuerier>, MessageInfo) {
    let gas_limit = 1_000_000_000; // ~1ms, enough for many executions within one instance
    let mut deps = mock_instance_with_gas_limit(WASM, gas_limit);
    let creator = String::from("creator");
    let info = mock_info(&creator, &[]);
    let res: Response =
        instantiate(&mut deps, mock_env(), info.clone(), InstantiateMsg {}).unwrap();
    assert_eq!(0, res.messages.len());
    (deps, info)
}

fn get_count(deps: &mut Instance<MockApi, MockStorage, MockQuerier>) -> u32 {
    let data = query(deps, mock_env(), QueryMsg::Count {}).unwrap();
    let res: CountResponse = from_json(data).unwrap();
    res.count
}

fn get_sum(deps: &mut Instance<MockApi, MockStorage, MockQuerier>) -> i32 {
    let data = query(deps, mock_env(), QueryMsg::Sum {}).unwrap();
    let res: SumResponse = from_json(data).unwrap();
    res.sum
}

#[test]
fn instantiate_and_query() {
    let (mut deps, _) = create_contract();
    assert_eq!(get_count(&mut deps), 0);
    assert_eq!(get_sum(&mut deps), 0);
}

#[test]
fn push_and_query() {
    let (mut deps, info) = create_contract();
    let _: Response = execute(
        &mut deps,
        mock_env(),
        info,
        ExecuteMsg::Enqueue { value: 25 },
    )
    .unwrap();
    assert_eq!(get_count(&mut deps), 1);
    assert_eq!(get_sum(&mut deps), 25);
}

#[test]
fn multiple_push() {
    let (mut deps, info) = create_contract();
    let _: Response = execute(
        &mut deps,
        mock_env(),
        info.clone(),
        ExecuteMsg::Enqueue { value: 25 },
    )
    .unwrap();
    let _: Response = execute(
        &mut deps,
        mock_env(),
        info.clone(),
        ExecuteMsg::Enqueue { value: 35 },
    )
    .unwrap();
    let _: Response = execute(
        &mut deps,
        mock_env(),
        info,
        ExecuteMsg::Enqueue { value: 45 },
    )
    .unwrap();
    assert_eq!(get_count(&mut deps), 3);
    assert_eq!(get_sum(&mut deps), 105);
}

#[test]
fn push_and_pop() {
    let (mut deps, info) = create_contract();
    let _: Response = execute(
        &mut deps,
        mock_env(),
        info.clone(),
        ExecuteMsg::Enqueue { value: 25 },
    )
    .unwrap();
    let _: Response = execute(
        &mut deps,
        mock_env(),
        info.clone(),
        ExecuteMsg::Enqueue { value: 17 },
    )
    .unwrap();
    let res: Response = execute(&mut deps, mock_env(), info, ExecuteMsg::Dequeue {}).unwrap();
    // ensure we popped properly
    assert!(res.data.is_some());
    let data = res.data.unwrap();
    let item: Item = from_json(data).unwrap();
    assert_eq!(item.value, 25);

    assert_eq!(get_count(&mut deps), 1);
    assert_eq!(get_sum(&mut deps), 17);
}

#[test]
fn push_and_reduce() {
    let (mut deps, info) = create_contract();
    let _: Response = execute(
        &mut deps,
        mock_env(),
        info.clone(),
        ExecuteMsg::Enqueue { value: 40 },
    )
    .unwrap();
    let _: Response = execute(
        &mut deps,
        mock_env(),
        info.clone(),
        ExecuteMsg::Enqueue { value: 15 },
    )
    .unwrap();
    let _: Response = execute(
        &mut deps,
        mock_env(),
        info.clone(),
        ExecuteMsg::Enqueue { value: 85 },
    )
    .unwrap();
    let _: Response = execute(
        &mut deps,
        mock_env(),
        info,
        ExecuteMsg::Enqueue { value: -10 },
    )
    .unwrap();
    assert_eq!(get_count(&mut deps), 4);
    assert_eq!(get_sum(&mut deps), 130);
    let data = query(&mut deps, mock_env(), QueryMsg::Reducer {}).unwrap();
    let counters = from_json::<ReducerResponse>(&data).unwrap().counters;
    assert_eq!(counters, vec![(40, 85), (15, 125), (85, 0), (-10, 140)]);
}

#[test]
fn migrate_works() {
    let (mut deps, info) = create_contract();

    let _: Response = execute(
        &mut deps,
        mock_env(),
        info.clone(),
        ExecuteMsg::Enqueue { value: 25 },
    )
    .unwrap();
    let _: Response = execute(
        &mut deps,
        mock_env(),
        info,
        ExecuteMsg::Enqueue { value: 17 },
    )
    .unwrap();
    assert_eq!(get_count(&mut deps), 2);
    assert_eq!(get_sum(&mut deps), 25 + 17);

    let msg = MigrateMsg {};
    let res: Response = migrate(&mut deps, mock_env(), msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    assert_eq!(get_count(&mut deps), 3);
    assert_eq!(get_sum(&mut deps), 100 + 101 + 102);
}

#[test]
fn query_list() {
    let (mut deps, info) = create_contract();

    for _ in 0..0x25 {
        let _: Response = execute(
            &mut deps,
            mock_env(),
            info.clone(),
            ExecuteMsg::Enqueue { value: 40 },
        )
        .unwrap();
    }
    for _ in 0..0x19 {
        let _: Response =
            execute(&mut deps, mock_env(), info.clone(), ExecuteMsg::Dequeue {}).unwrap();
    }
    // we add 0x25 items and then remove the first 0x19, leaving [0x19, 0x1a, 0x1b, ..., 0x24]
    // since we count up to 0x20 in early, we get early and late both with data

    let query_msg = QueryMsg::List {};
    let ids: ListResponse = from_json(query(&mut deps, mock_env(), query_msg).unwrap()).unwrap();
    assert_eq!(ids.empty, Vec::<u32>::new());
    assert_eq!(ids.early, vec![0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f]);
    assert_eq!(ids.late, vec![0x20, 0x21, 0x22, 0x23, 0x24]);
}

#[test]
fn query_open_iterators() {
    let (mut deps, _info) = create_contract();

    let query_msg = QueryMsg::OpenIterators { count: 0 };
    let _ = query(&mut deps, mock_env(), query_msg).unwrap();

    let query_msg = QueryMsg::OpenIterators { count: 1 };
    let _ = query(&mut deps, mock_env(), query_msg).unwrap();

    let query_msg = QueryMsg::OpenIterators { count: 321 };
    let _ = query(&mut deps, mock_env(), query_msg).unwrap();
}
