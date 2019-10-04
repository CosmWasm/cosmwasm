use std::fs;

use serde_json::{to_vec};

use hackatom::contract::{RegenInitMsg, RegenSendMsg};
use cosmwasm::types::{coin, ContractResult, CosmosMsg, mock_params};
use cosmwasm_vm::{call_init, call_send, instantiate};

/**
This integration test tries to run and call the generated wasm.
It depends on a release build being available already. You can create that with:

cargo wasm && wasm-gc ./target/wasm32-unknown-unknown/release/hackatom.wasm

Then running `cargo test` will validate we can properly call into that generated data.
**/

// Note this is very similar in scope and size to proper_send in contracts.rs tests
// Making it as easy to write vm external integration tests as rust unit tests
#[test]
fn succeessful_init_and_send() {
    let wasm_file = "./target/wasm32-unknown-unknown/release/hackatom.wasm";
    let wasm = fs::read(wasm_file).unwrap();
    assert!(wasm.len() > 100000);

    // create the instance
    let mut instance = instantiate(&wasm);

    // prepare arguments
    let params = mock_params("creator", &coin("1000", "earth"), &[]);
    let msg = to_vec(&RegenInitMsg {
        verifier: String::from("verifies"),
        beneficiary: String::from("benefits"),
    }).unwrap();

    // call and check
    let res = call_init(&mut instance, &params, &msg).unwrap();
    match res {
        ContractResult::Msgs(msgs) => {
            assert_eq!(msgs.len(), 0);
        },
        ContractResult::Error(err) => panic!("Unexpected error: {}", err),
    }

    // now try to send this one
    let params = mock_params("verifies", &coin("15", "earth"), &coin("1015", "earth"));
    let msg = to_vec(&RegenSendMsg {}).unwrap();
    let res = call_send(&mut instance, &params, &msg).unwrap();
    match res {
        ContractResult::Msgs(msgs) => {
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
        },
        ContractResult::Error(err) => panic!("Unexpected error: {}", err),
    }
}