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

use cosmwasm_std::testing::{mock_env, MockApi, MockStorage};
use cosmwasm_std::{coin, from_slice, Env, HumanAddr};
use cosmwasm_vm::testing::{handle, init, mock_instance, query};
use cosmwasm_vm::Instance;

use queue::contract::{CountResponse, HandleMsg, InitMsg, Item, QueryMsg, SumResponse};

static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/queue.wasm");

fn create_contract() -> (Instance<MockStorage, MockApi>, Env) {
    let mut deps = mock_instance(WASM);
    let creator = HumanAddr(String::from("creator"));
    let env = mock_env(&deps.api, creator.as_str(), &coin("1000", "earth"), &[]);
    let res = init(&mut deps, env.clone(), InitMsg {}).unwrap();
    assert_eq!(0, res.messages.len());
    (deps, env)
}

fn get_count(deps: &mut Instance<MockStorage, MockApi>) -> u32 {
    let data = query(deps, QueryMsg::Count {}).unwrap();
    let res: CountResponse = from_slice(data.as_slice()).unwrap();
    res.count
}

fn get_sum(deps: &mut Instance<MockStorage, MockApi>) -> i32 {
    let data = query(deps, QueryMsg::Sum {}).unwrap();
    let res: SumResponse = from_slice(data.as_slice()).unwrap();
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
    let (mut deps, env) = create_contract();
    handle(&mut deps, env.clone(), HandleMsg::Enqueue { value: 25 }).unwrap();
    assert_eq!(get_count(&mut deps), 1);
    assert_eq!(get_sum(&mut deps), 25);
}

#[test]
fn multiple_push() {
    let (mut deps, env) = create_contract();
    handle(&mut deps, env.clone(), HandleMsg::Enqueue { value: 25 }).unwrap();
    handle(&mut deps, env.clone(), HandleMsg::Enqueue { value: 35 }).unwrap();
    handle(&mut deps, env.clone(), HandleMsg::Enqueue { value: 45 }).unwrap();
    assert_eq!(get_count(&mut deps), 3);
    assert_eq!(get_sum(&mut deps), 105);
}

#[test]
fn push_and_pop() {
    let (mut deps, env) = create_contract();
    handle(&mut deps, env.clone(), HandleMsg::Enqueue { value: 25 }).unwrap();
    handle(&mut deps, env.clone(), HandleMsg::Enqueue { value: 17 }).unwrap();
    let res = handle(&mut deps, env.clone(), HandleMsg::Dequeue {}).unwrap();
    // ensure we popped properly
    assert!(res.data.is_some());
    let data = res.data.unwrap();
    let item: Item = from_slice(data.as_slice()).unwrap();
    assert_eq!(item.value, 25);

    assert_eq!(get_count(&mut deps), 1);
    assert_eq!(get_sum(&mut deps), 17);
}
