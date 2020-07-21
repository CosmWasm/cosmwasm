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

use cosmwasm_std::{coins, BankMsg, HumanAddr, InitResult, MigrateResponse, Order, StdError};
use cosmwasm_vm::testing::{init, migrate, mock_env, mock_instance, MOCK_CONTRACT_ADDR};
use cosmwasm_vm::StorageIterator;

use burner::msg::{InitMsg, MigrateMsg};
use cosmwasm_vm::Storage;

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/burner.wasm");
// You can uncomment this line instead to test productionified build from rust-optimizer
// static WASM: &[u8] = include_bytes!("../contract.wasm");

#[test]
fn init_fails() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg {};
    let env = mock_env(&deps.api, "creator", &coins(1000, "earth"));
    // we can just call .unwrap() to assert this was a success
    let res: InitResult = init(&mut deps, env, msg);
    match res.unwrap_err() {
        StdError::GenericErr { msg, .. } => {
            assert_eq!(msg, "You can only use this contract for migrations")
        }
        _ => panic!("expected migrate error message"),
    }
}

#[test]
fn migrate_cleans_up_data() {
    let mut deps = mock_instance(WASM, &coins(123456, "gold"));

    // store some sample data
    deps.with_storage(|storage| {
        storage.set(b"foo", b"bar").0.unwrap();
        storage.set(b"key2", b"data2").0.unwrap();
        storage.set(b"key3", b"cool stuff").0.unwrap();
        let cnt = storage
            .range(None, None, Order::Ascending)
            .0
            .unwrap()
            .elements()
            .unwrap()
            .len();
        assert_eq!(3, cnt);
        Ok(())
    })
    .unwrap();

    // change the verifier via migrate
    let payout = HumanAddr::from("someone else");
    let msg = MigrateMsg {
        payout: payout.clone(),
    };
    let env = mock_env(&deps.api, "creator", &[]);
    let res: MigrateResponse = migrate(&mut deps, env, msg).unwrap();
    // check payout
    assert_eq!(1, res.messages.len());
    let msg = res.messages.get(0).expect("no message");
    assert_eq!(
        msg,
        &BankMsg::Send {
            from_address: HumanAddr::from(MOCK_CONTRACT_ADDR),
            to_address: payout,
            amount: coins(123456, "gold"),
        }
        .into(),
    );

    // check there is no data in storage
    deps.with_storage(|storage| {
        let cnt = storage
            .range(None, None, Order::Ascending)
            .0
            .unwrap()
            .elements()
            .unwrap()
            .len();
        assert_eq!(0, cnt);
        Ok(())
    })
    .unwrap();
}
