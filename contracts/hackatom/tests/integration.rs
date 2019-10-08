use std::fs;

use serde_json::{from_slice, to_vec};

use cosmwasm::storage::Storage;
use cosmwasm::types::{coin, mock_params, CosmosMsg};
use cosmwasm_vm::{call_handle, call_init, instantiate, with_storage};
use hackatom::contract::{RegenHandleMsg, RegenInitMsg, RegenState, CONFIG_KEY};

/**
This integration test tries to run and call the generated wasm.
It depends on a release build being available already. You can create that with:

cargo wasm && wasm-gc ./target/wasm32-unknown-unknown/release/hackatom.wasm

Then running `cargo test` will validate we can properly call into that generated data.
**/

// Note this is very similar in scope and size to proper_handle in contracts.rs tests
// Making it as easy to write vm external integration tests as rust unit tests
#[test]
fn successful_init_and_handle() {
    let wasm_file = "./target/wasm32-unknown-unknown/release/hackatom.wasm";
    let wasm = fs::read(wasm_file).unwrap();
    assert!(wasm.len() > 100000);
    let mut instance = instantiate(&wasm);

    // prepare arguments
    let params = mock_params("creator", &coin("1000", "earth"), &[]);
    let msg = to_vec(&RegenInitMsg {
        verifier: String::from("verifies"),
        beneficiary: String::from("benefits"),
    })
    .unwrap();

    // call and check
    let res = call_init(&mut instance, &params, &msg).unwrap();
    let msgs = res.unwrap();
    assert_eq!(msgs.len(), 0);

    // now try to handle this one
    let params = mock_params("verifies", &coin("15", "earth"), &coin("1015", "earth"));
    let msg = to_vec(&RegenHandleMsg {}).unwrap();
    let res = call_handle(&mut instance, &params, &msg).unwrap();
    let msgs = res.unwrap();
    assert_eq!(1, msgs.len());
    let msg = msgs.get(0).expect("no message");
    match &msg {
        CosmosMsg::SendTx {
            from_address,
            to_address,
            amount,
        } => {
            assert_eq!("cosmos2contract", from_address);
            assert_eq!("benefits", to_address);
            assert_eq!(1, amount.len());
            let coin = amount.get(0).expect("No coin");
            assert_eq!(coin.denom, "earth");
            assert_eq!(coin.amount, "1015");
        }
    }

    // we can check the storage as well
    with_storage(&instance, |store| {
        let foo = store.get(b"foo");
        assert!(foo.is_none());
        let data = store.get(CONFIG_KEY).expect("no data stored");
        let state: RegenState = from_slice(&data).unwrap();
        assert_eq!(state.verifier, String::from("verifies"));
    });
}

#[test]
fn failed_handle() {
    let wasm_file = "./target/wasm32-unknown-unknown/release/hackatom.wasm";
    let wasm = fs::read(wasm_file).unwrap();
    assert!(wasm.len() > 100000);
    let mut instance = instantiate(&wasm);

    // initialize the store
    let init_msg = serde_json::to_vec(&RegenInitMsg {
        verifier: String::from("verifies"),
        beneficiary: String::from("benefits"),
    })
    .unwrap();
    let init_params = mock_params("creator", &coin("1000", "earth"), &coin("1000", "earth"));
    let init_res = call_init(&mut instance, &init_params, &init_msg)
        .unwrap()
        .unwrap();
    assert_eq!(0, init_res.len());

    // beneficiary can release it
    let handle_params = mock_params("benefits", &[], &coin("1000", "earth"));
    let handle_res = call_handle(&mut instance, &handle_params, b"").unwrap();
    assert!(handle_res.is_err());

    // state should be saved
    with_storage(&instance, |store| {
        let data = store.get(CONFIG_KEY).expect("no data stored");
        let state: RegenState = from_slice(&data).unwrap();
        assert_eq!(state.verifier, String::from("verifies"));
    });
}
