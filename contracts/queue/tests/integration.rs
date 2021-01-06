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

use cosmwasm_std::{from_binary, from_slice, HandleResponse, HumanAddr, InitResponse, MessageAuth};
use cosmwasm_vm::{
    testing::{
        handle, init, mock_auth, mock_env, mock_instance, query, MockApi, MockQuerier, MockStorage,
    },
    Instance,
};

use queue::contract::{
    CountResponse, HandleMsg, InitMsg, Item, ListResponse, QueryMsg, ReducerResponse, SumResponse,
};

static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/queue.wasm");

fn create_contract() -> (Instance<MockStorage, MockApi, MockQuerier>, MessageAuth) {
    let mut deps = mock_instance(WASM, &[]);
    let creator = HumanAddr(String::from("creator"));
    let auth = mock_auth(creator.as_str(), &[]);
    let res: InitResponse = init(&mut deps, mock_env(), auth.clone(), InitMsg {}).unwrap();
    assert_eq!(0, res.messages.len());
    (deps, auth)
}

fn get_count(deps: &mut Instance<MockStorage, MockApi, MockQuerier>) -> u32 {
    let data = query(deps, mock_env(), QueryMsg::Count {}).unwrap();
    let res: CountResponse = from_binary(&data).unwrap();
    res.count
}

fn get_sum(deps: &mut Instance<MockStorage, MockApi, MockQuerier>) -> i32 {
    let data = query(deps, mock_env(), QueryMsg::Sum {}).unwrap();
    let res: SumResponse = from_binary(&data).unwrap();
    res.sum
}

#[test]
fn init_and_query() {
    let (mut deps, _) = create_contract();
    assert_eq!(get_count(&mut deps), 0);
    assert_eq!(get_sum(&mut deps), 0);
}

#[test]
fn push_and_query() {
    let (mut deps, auth) = create_contract();
    let _: HandleResponse = handle(
        &mut deps,
        mock_env(),
        auth.clone(),
        HandleMsg::Enqueue { value: 25 },
    )
    .unwrap();
    assert_eq!(get_count(&mut deps), 1);
    assert_eq!(get_sum(&mut deps), 25);
}

#[test]
fn multiple_push() {
    let (mut deps, auth) = create_contract();
    let _: HandleResponse = handle(
        &mut deps,
        mock_env(),
        auth.clone(),
        HandleMsg::Enqueue { value: 25 },
    )
    .unwrap();
    let _: HandleResponse = handle(
        &mut deps,
        mock_env(),
        auth.clone(),
        HandleMsg::Enqueue { value: 35 },
    )
    .unwrap();
    let _: HandleResponse = handle(
        &mut deps,
        mock_env(),
        auth.clone(),
        HandleMsg::Enqueue { value: 45 },
    )
    .unwrap();
    assert_eq!(get_count(&mut deps), 3);
    assert_eq!(get_sum(&mut deps), 105);
}

#[test]
fn push_and_pop() {
    let (mut deps, auth) = create_contract();
    let _: HandleResponse = handle(
        &mut deps,
        mock_env(),
        auth.clone(),
        HandleMsg::Enqueue { value: 25 },
    )
    .unwrap();
    let _: HandleResponse = handle(
        &mut deps,
        mock_env(),
        auth.clone(),
        HandleMsg::Enqueue { value: 17 },
    )
    .unwrap();
    let res: HandleResponse =
        handle(&mut deps, mock_env(), auth.clone(), HandleMsg::Dequeue {}).unwrap();
    // ensure we popped properly
    assert!(res.data.is_some());
    let data = res.data.unwrap();
    let item: Item = from_slice(data.as_slice()).unwrap();
    assert_eq!(item.value, 25);

    assert_eq!(get_count(&mut deps), 1);
    assert_eq!(get_sum(&mut deps), 17);
}

#[test]
fn push_and_reduce() {
    let (mut deps, auth) = create_contract();
    let _: HandleResponse = handle(
        &mut deps,
        mock_env(),
        auth.clone(),
        HandleMsg::Enqueue { value: 40 },
    )
    .unwrap();
    let _: HandleResponse = handle(
        &mut deps,
        mock_env(),
        auth.clone(),
        HandleMsg::Enqueue { value: 15 },
    )
    .unwrap();
    let _: HandleResponse = handle(
        &mut deps,
        mock_env(),
        auth.clone(),
        HandleMsg::Enqueue { value: 85 },
    )
    .unwrap();
    let _: HandleResponse = handle(
        &mut deps,
        mock_env(),
        auth.clone(),
        HandleMsg::Enqueue { value: -10 },
    )
    .unwrap();
    assert_eq!(get_count(&mut deps), 4);
    assert_eq!(get_sum(&mut deps), 130);
    let data = query(&mut deps, mock_env(), QueryMsg::Reducer {}).unwrap();
    let counters = from_binary::<ReducerResponse>(&data).unwrap().counters;
    assert_eq!(counters, vec![(40, 85), (15, 125), (85, 0), (-10, 140)]);
}

#[test]
fn query_list() {
    let (mut deps, auth) = create_contract();

    for _ in 0..0x25 {
        let _: HandleResponse = handle(
            &mut deps,
            mock_env(),
            auth.clone(),
            HandleMsg::Enqueue { value: 40 },
        )
        .unwrap();
    }
    for _ in 0..0x19 {
        let _: HandleResponse =
            handle(&mut deps, mock_env(), auth.clone(), HandleMsg::Dequeue {}).unwrap();
    }
    // we add 0x25 items and then remove the first 0x19, leaving [0x19, 0x1a, 0x1b, ..., 0x24]
    // since we count up to 0x20 in early, we get early and late both with data

    let query_msg = QueryMsg::List {};
    let ids: ListResponse = from_binary(&query(&mut deps, mock_env(), query_msg).unwrap()).unwrap();
    assert_eq!(ids.empty, Vec::<u32>::new());
    assert_eq!(ids.early, vec![0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f]);
    assert_eq!(ids.late, vec![0x20, 0x21, 0x22, 0x23, 0x24]);
}
