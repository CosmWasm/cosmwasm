use std::fs;
use std::str::from_utf8;

use hackatom::contract::{RegenInitMsg};
use cosmwasm::imports::Storage;
use cosmwasm::types::{coin, mock_params};

mod memory;
mod exports;
mod wasmer;

use crate::exports::{do_read, do_write, setup_context};
use crate::memory::{read_memory, write_memory, allocate};
use crate::wasmer::{Func, func, imports, wasm_instance};

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

    // TODO: set up proper callback for read and write here
    // TODO: figure out passing state
    let import_object = imports! {
        || { setup_context() },
        "env" => {
            "c_read" => func!(do_read),
            "c_write" => func!(do_write),
        },
    };

    // create the instance
    let mut instance = wasm_instance(wasm, import_object);

    // prepare arguments
    let params = mock_params("creator", &coin("1000", "earth"), &[]);
    let json_params = serde_json::to_vec(&params).unwrap();
    // currently we need to 0 pad it

    let msg = &RegenInitMsg {
        verifier: String::from("verifies"),
        beneficiary: String::from("benefits"),
    };
    let json_msg = serde_json::to_vec(&msg).unwrap();

    // place data in the instance memory
    let param_offset = allocate(&mut instance, &json_params);
    let msg_offset = allocate(&mut instance, &json_msg);

    // call the instance
    let init: Func<(i32, i32), (i32)> = instance.func("init_wrapper").unwrap();
    let res_offset = init.call(param_offset, msg_offset).unwrap();
    assert!(res_offset > 1000);

    // read the return value
    let res = read_memory(instance.context(), res_offset);
    let str_res = from_utf8(&res).unwrap();
    assert_eq!(str_res , "{\"msgs\":[]}");
}

