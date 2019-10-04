use std::fs;

use serde_json::{from_slice, to_vec};

use hackatom::contract::{RegenInitMsg};
use cosmwasm::types::{coin, ContractResult, mock_params};
use cosmwasm_vm::{allocate, Func, instantiate, read_memory};

/**
This integration test tries to run and call the generated wasm.
It depends on a release build being available already. You can create that with:

cargo wasm && wasm-gc ./target/wasm32-unknown-unknown/release/hackatom.wasm

Then running `cargo test` will validate we can properly call into that generated data.
**/

#[test]
fn run_contract() {
    let wasm_file = "./target/wasm32-unknown-unknown/release/hackatom.wasm";
    let wasm = fs::read(wasm_file).unwrap();
    assert!(wasm.len() > 100000);

    // create the instance
    let mut instance = instantiate(&wasm);
//    let send: Func<(i32, i32), (i32)> = instance.func("send_wrapper").unwrap();

    // prepare arguments
    let i_params = to_vec(
        &mock_params("creator", &coin("1000", "earth"), &[])).unwrap();
    let i_msg = to_vec(&RegenInitMsg {
        verifier: String::from("verifies"),
        beneficiary: String::from("benefits"),
    }).unwrap();

    // call the instance
    let param_offset = allocate(&mut instance, &i_params);
    let msg_offset = allocate(&mut instance, &i_msg);
    let init: Func<(i32, i32), (i32)> = instance.func("init_wrapper").unwrap();
    let res_offset = init.call(param_offset, msg_offset).unwrap();
    assert!(res_offset > 1000);

    // read the return value
    let res: ContractResult = from_slice(&read_memory(instance.context(), res_offset)).unwrap();
    match res {
        ContractResult::Msgs(msgs) => {
            assert_eq!(msgs.len(), 0);
        },
        ContractResult::Error(err) => panic!("Unexpected error: {}", err),
    }
}

