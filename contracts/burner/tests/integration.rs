//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.
//!
//! You can easily convert unit tests to integration tests.
//! 1. First copy them over verbatum,
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

use cosmwasm_std::{coins, Attribute, BankMsg, ContractResult, Order, Response, SubMsg};
use cosmwasm_vm::testing::{execute, instantiate, migrate, mock_env, mock_info, mock_instance};

use burner::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use cosmwasm_vm::Storage;

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/burner.wasm");
// You can uncomment this line instead to test productionified build from rust-optimizer
// static WASM: &[u8] = include_bytes!("../contract.wasm");

/// Gets the value of the first attribute with the given key
fn first_attr(data: impl AsRef<[Attribute]>, search_key: &str) -> Option<String> {
    data.as_ref().iter().find_map(|a| {
        if a.key == search_key {
            Some(a.value.clone())
        } else {
            None
        }
    })
}

#[test]
fn instantiate_fails() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InstantiateMsg {};
    let info = mock_info("creator", &coins(1000, "earth"));
    // we can just call .unwrap() to assert this was a success
    let res: ContractResult<Response> = instantiate(&mut deps, mock_env(), info, msg);
    let msg = res.unwrap_err();
    assert_eq!(
        msg,
        "Generic error: You can only use this contract for migrations"
    );
}

#[test]
fn migrate_sends_funds() {
    let mut deps = mock_instance(WASM, &coins(123456, "gold"));

    // change the verifier via migrate
    let payout = String::from("someone else");
    let msg = MigrateMsg {
        payout: payout.clone(),
    };
    let res: Response = migrate(&mut deps, mock_env(), msg).unwrap();
    // check payout
    assert_eq!(1, res.messages.len());
    let msg = res.messages.get(0).expect("no message");
    assert_eq!(
        msg,
        &SubMsg::new(BankMsg::Send {
            to_address: payout,
            amount: coins(123456, "gold"),
        }),
    );
}

#[test]
fn execute_cleans_up_data() {
    let mut deps = mock_instance(WASM, &coins(123456, "gold"));

    // store some sample data
    deps.with_storage(|storage| {
        storage.set(b"foo", b"bar").0.unwrap();
        storage.set(b"key2", b"data2").0.unwrap();
        storage.set(b"key3", b"cool stuff").0.unwrap();
        let iter_id = storage.scan(None, None, Order::Ascending).0.unwrap();
        let cnt = storage.all(iter_id).0.unwrap().len();
        assert_eq!(cnt, 3);
        Ok(())
    })
    .unwrap();

    // change the verifier via migrate
    let payout = String::from("someone else");
    let msg = MigrateMsg { payout };
    let _res: Response = migrate(&mut deps, mock_env(), msg).unwrap();

    let res: Response = execute(
        &mut deps,
        mock_env(),
        mock_info("anon", &[]),
        ExecuteMsg::Cleanup { limit: Some(2) },
    )
    .unwrap();
    assert_eq!(first_attr(res.attributes, "deleted_entries").unwrap(), "2");

    // One item should be left
    deps.with_storage(|storage| {
        let iter_id = storage.scan(None, None, Order::Ascending).0.unwrap();
        let cnt = storage.all(iter_id).0.unwrap().len();
        assert_eq!(cnt, 1);
        Ok(())
    })
    .unwrap();

    let res: Response = execute(
        &mut deps,
        mock_env(),
        mock_info("anon", &[]),
        ExecuteMsg::Cleanup { limit: Some(2) },
    )
    .unwrap();
    assert_eq!(first_attr(res.attributes, "deleted_entries").unwrap(), "1");

    // check there is no data in storage
    deps.with_storage(|storage| {
        let iter_id = storage.scan(None, None, Order::Ascending).0.unwrap();
        let cnt = storage.all(iter_id).0.unwrap().len();
        assert_eq!(cnt, 0);
        Ok(())
    })
    .unwrap();
}
