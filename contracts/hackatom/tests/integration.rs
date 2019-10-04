use std::fs;
use std::str::from_utf8;

use wasmer_runtime::{compile_with, Ctx, Func, func, imports};
use wasmer_runtime_core::{Instance};
use wasmer_clif_backend::CraneliftCompiler;

use hackatom::contract::{RegenInitMsg};
use cosmwasm::imports::Storage;
use cosmwasm::types::{coin, mock_params};

mod memory;
mod context;

use crate::context::{create_unmanaged_storage, destroy_unmanaged_storage, with_storage_from_context};
use crate::memory::{read_memory, write_memory, allocate};

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
        || (create_unmanaged_storage(), destroy_unmanaged_storage),
        "env" => {
            "c_read" => func!(do_read),
            "c_write" => func!(do_write),
        },
    };

    // create the instance
    let module = compile_with(&wasm, &CraneliftCompiler::new()).unwrap();
    let mut instance = module.instantiate (&import_object).unwrap();

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

/*** mocks to stub out actually db writes as extern "C" ***/

fn do_read(ctx: &mut Ctx, key_ptr: i32, val_ptr: i32) -> i32 {
    let key = read_memory(ctx, key_ptr);
    let mut value: Option<Vec<u8>> = None;
    with_storage_from_context(ctx, |store| value = store.get(&key));
    match value {
        Some(buf) => write_memory(ctx, val_ptr, &buf),
        None => 0,
    }
}

fn do_write(ctx: &mut Ctx, key: i32, value: i32) {
    let key = read_memory(ctx, key);
    let value = read_memory(ctx, value);
    with_storage_from_context(ctx, |store| store.set(&key, &value));
}


