use cosmwasm::mock::MockStorage;
use cosmwasm::serde::{from_slice, to_vec};
use cosmwasm::storage::Storage;
use cosmwasm::types::{coin, mock_params, CosmosMsg};
use cosmwasm_vm::{call_handle, call_init, Instance};
use cosmwasm_vm::testing::{handle, init, mock_instance};

use hackatom::contract::{HandleMsg, InitMsg, State, CONFIG_KEY};

/**
This integration test tries to run and call the generated wasm.
It depends on a release build being available already. You can create that with:

cargo wasm && wasm-gc ./target/wasm32-unknown-unknown/release/hackatom.wasm

Then running `cargo test` will validate we can properly call into that generated data.
**/
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/hackatom.wasm");

#[test]
fn proper_initialization() {
    let mut store = mock_instance(WASM);
    let msg = to_vec(&InitMsg {
        verifier: String::from("verifies"),
        beneficiary: String::from("benefits"),
    })
        .unwrap();
    let params = mock_params("creator", &coin("1000", "earth"), &[]);
    let res = init(&mut store, params, msg).unwrap();
    assert_eq!(0, res.messages.len());

//    // it worked, let's check the state
//    let data = store.get(CONFIG_KEY).expect("no data stored");
//    let state: State = from_slice(&data).unwrap();
//    assert_eq!(state, State{
//        verifier: "verifies".to_string(),
//        beneficiary: "benefits".to_string(),
//        funder: "creator".to_string(),
//    });
}

#[test]
fn fails_on_bad_init() {
    let mut store = mock_instance(WASM);
    let bad_msg = b"{}".to_vec();
    let params = mock_params("creator", &coin("1000", "earth"), &[]);
    let res = init(&mut store, params, bad_msg);
    assert_eq!(true, res.is_err());
}


// Note this is very similar in scope and size to proper_handle in contracts.rs tests
// Making it as easy to write vm external integration tests as rust unit tests
#[test]
fn successful_init_and_handle() {
    assert!(WASM.len() > 100000);
    let storage = MockStorage::new();
    let mut instance = Instance::from_code(&WASM, storage).unwrap();

    // prepare arguments
    let params = mock_params("creator", &coin("1000", "earth"), &[]);
    let msg = to_vec(&InitMsg {
        verifier: String::from("verifies"),
        beneficiary: String::from("benefits"),
    })
    .unwrap();

    // call and check
    let res = call_init(&mut instance, &params, &msg).unwrap();
    let msgs = res.unwrap().messages;
    assert_eq!(msgs.len(), 0);

    // now try to handle this one
    let params = mock_params("verifies", &coin("15", "earth"), &coin("1015", "earth"));
    let msg = to_vec(&HandleMsg {}).unwrap();
    let res = call_handle(&mut instance, &params, &msg).unwrap();
    let msgs = res.unwrap().messages;
    assert_eq!(1, msgs.len());
    let msg = msgs.get(0).expect("no message");
    assert_eq!(msg, &CosmosMsg::Send{
        from_address: "cosmos2contract".to_string(),
        to_address: "benefits".to_string(),
        amount: coin("1015", "earth"),
    });

    // we can check the storage as well
    instance.with_storage(|store| {
        let foo = store.get(b"foo");
        assert!(foo.is_none());
        let data = store.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_slice(&data).unwrap();
        assert_eq!(state, State{
            verifier: "verifies".to_string(),
            beneficiary: "benefits".to_string(),
            funder: "creator".to_string(),
        });
    });
}

#[test]
fn failed_handle() {
    let storage = MockStorage::new();
    let mut instance = Instance::from_code(&WASM, storage).unwrap();

    // initialize the store
    let init_msg = to_vec(&InitMsg {
        verifier: String::from("verifies"),
        beneficiary: String::from("benefits"),
    })
    .unwrap();
    let init_params = mock_params("creator", &coin("1000", "earth"), &coin("1000", "earth"));
    let init_res = call_init(&mut instance, &init_params, &init_msg).unwrap();
    let init_msgs = init_res.unwrap().messages;
    assert_eq!(0, init_msgs.len());

    // beneficiary can release it
    let handle_params = mock_params("benefits", &[], &coin("1000", "earth"));
    let handle_res = call_handle(&mut instance, &handle_params, b"").unwrap();
    assert!(handle_res.is_err());

    // state should be saved
    instance.with_storage(|store| {
        let data = store.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_slice(&data).unwrap();
        assert_eq!(state.verifier, String::from("verifies"));
    });
}
