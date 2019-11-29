use cosmwasm::serde::{from_slice, to_vec};
use cosmwasm::storage::Storage;
use cosmwasm::types::{coin, mock_params, CosmosMsg};
use cosmwasm_vm::testing::{handle, init, mock_instance, query};

use hackatom::contract::{raw_query, InitMsg, State, CONFIG_KEY};

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

    // it worked, let's check the state
    store.with_storage(|store| {
        let data = store.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_slice(&data).unwrap();
        assert_eq!(
            state,
            State {
                verifier: "verifies".to_string(),
                beneficiary: "benefits".to_string(),
                funder: "creator".to_string(),
            }
        );
    });
}

#[test]
fn proper_init_and_query() {
    let mut store = mock_instance(WASM);
    let msg = to_vec(&InitMsg {
        verifier: String::from("foo"),
        beneficiary: String::from("bar"),
    })
    .unwrap();
    let params = mock_params("creator", &coin("1000", "earth"), &[]);
    let _res = init(&mut store, params, msg).unwrap();

    let q_res = query(&mut store, raw_query(b"random").unwrap()).unwrap();
    assert_eq!(q_res.results.len(), 0);

    // query for state
    let mut q_res = query(&mut store, raw_query(CONFIG_KEY).unwrap()).unwrap();
    let model = q_res.results.pop().unwrap();
    let state: State = from_slice(&model.val).unwrap();
    assert_eq!(
        state,
        State {
            verifier: "foo".to_string(),
            beneficiary: "bar".to_string(),
            funder: "creator".to_string(),
        }
    );
}

#[test]
fn fails_on_bad_init() {
    let mut store = mock_instance(WASM);
    let bad_msg = b"{}".to_vec();
    let params = mock_params("creator", &coin("1000", "earth"), &[]);
    let res = init(&mut store, params, bad_msg);
    assert_eq!(true, res.is_err());
}

#[test]
fn proper_handle() {
    let mut store = mock_instance(WASM);

    // initialize the store
    let init_msg = to_vec(&InitMsg {
        verifier: String::from("verifies"),
        beneficiary: String::from("benefits"),
    })
    .unwrap();
    let init_params = mock_params("creator", &coin("1000", "earth"), &coin("1000", "earth"));
    let init_res = init(&mut store, init_params, init_msg).unwrap();
    assert_eq!(0, init_res.messages.len());

    // beneficiary can release it
    let handle_params = mock_params("verifies", &coin("15", "earth"), &coin("1015", "earth"));
    let handle_res = handle(&mut store, handle_params, Vec::new()).unwrap();
    assert_eq!(1, handle_res.messages.len());
    let msg = handle_res.messages.get(0).expect("no message");
    assert_eq!(
        msg,
        &CosmosMsg::Send {
            from_address: "cosmos2contract".to_string(),
            to_address: "benefits".to_string(),
            amount: coin("1015", "earth"),
        }
    );

    // it worked, let's check the state
    store.with_storage(|store| {
        let data = store.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_slice(&data).unwrap();
        assert_eq!(
            state,
            State {
                verifier: "verifies".to_string(),
                beneficiary: "benefits".to_string(),
                funder: "creator".to_string(),
            }
        );
    });
}

#[test]
fn failed_handle() {
    let mut store = mock_instance(WASM);

    // initialize the store
    let init_msg = to_vec(&InitMsg {
        verifier: String::from("verifies"),
        beneficiary: String::from("benefits"),
    })
    .unwrap();
    let init_params = mock_params("creator", &coin("1000", "earth"), &coin("1000", "earth"));
    let init_res = init(&mut store, init_params, init_msg).unwrap();
    assert_eq!(0, init_res.messages.len());

    // beneficiary can release it
    let handle_params = mock_params("benefits", &[], &coin("1000", "earth"));
    let handle_res = handle(&mut store, handle_params, Vec::new());
    assert!(handle_res.is_err());

    // state should not change
    store.with_storage(|store| {
        let data = store.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_slice(&data).unwrap();
        assert_eq!(
            state,
            State {
                verifier: "verifies".to_string(),
                beneficiary: "benefits".to_string(),
                funder: "creator".to_string(),
            }
        );
    });
}
