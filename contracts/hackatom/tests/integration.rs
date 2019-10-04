extern crate hackatom;

use std::fs;
use std::mem;
use std::str::from_utf8;
use std::ffi::c_void;

use wasmer_runtime::{compile_with, Ctx, Func, func, imports};
use wasmer_runtime_core::{Instance};
use wasmer_clif_backend::CraneliftCompiler;

use hackatom::mock::{MockStorage};
use hackatom::types::{Buffer, coin, mock_params};
use hackatom::contract::{RegenInitMsg};
use hackatom::imports::Storage;

#[test]
fn test_coin() {
    let c = hackatom::types::coin("123", "tokens");
    assert_eq!(c.len(), 1);
    assert_eq!(c.get(0).unwrap().amount, "123");
}

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

    // what does alloc return
    let alloc: Func<(i32), (i32)> = instance.func("allocate").unwrap();
    let offset = alloc.call(5789).unwrap();
    println!("alloc {:?}", offset);

    // let's check if we can get and pass arguments
    let create: Func<(), (i32)> = instance.func("create_buffer").unwrap();
    let offset = create.call().unwrap();
    let buf_ptr = (offset / 4) as usize;  // convert from u8 to i32
    let memory = &instance.context().memory(0).view::<i32>();
    println!("buf_ptr {:?}", buf_ptr);
    println!("offset {:?}", offset);
    // TODO: add helpers to read in the buffer contents if we need them
    let buf_offset = memory[buf_ptr].get();
    let buf_len = memory[buf_ptr+1].get();
    assert_eq!(buf_offset, 12345);
    assert_eq!(buf_len, 100);

    // let's try to modify it for fun
    memory[buf_ptr].set(9876543);

    // and they can read it as well...
    let free: Func<(i32), (i32)> = instance.func("free_buffer").unwrap();
    let freed_offset = free.call(offset).unwrap();
    assert_eq!(freed_offset, 9876543);

    // prepare arguments
    let params = mock_params("creator", &coin("1000", "earth"), &[]);
    let mut json_params = serde_json::to_vec(&params).unwrap();
    // currently we need to 0 pad it
    json_params.push(0);

    let msg = &RegenInitMsg {
        verifier: String::from("verifies"),
        beneficiary: String::from("benefits"),
    };
    let mut json_msg = serde_json::to_vec(&msg).unwrap();
    json_msg.push(0);

    // place data in the instance memory
    let param_offset = allocate(&mut instance, &json_params);
    let msg_offset = allocate(&mut instance, &json_msg);

    // call the instance
    let init: Func<(i32, i32, i32), (i32)> = instance.func("init_wrapper").unwrap();
    let res_offset = init.call(15, param_offset, msg_offset).unwrap();
    assert!(res_offset > 1000);

    // read the return value
    let res = read_memory(instance.context(), res_offset);
    let str_res = from_utf8(&res).unwrap();
    assert_eq!(str_res , "{\"msgs\":[]}");
}

// write_mem allocates memory in the instance and copies the given data in
// returns the memory offset, to be passed as an argument
// panics on any error (TODO, use result?)
fn allocate(instance: &mut Instance, data: &[u8]) -> i32 {
    // allocate
    let alloc: Func<(i32), (i32)> = instance.func("allocate").unwrap();
    let offset = alloc.call(data.len() as i32).unwrap();
    write_memory(instance.context(), offset, data);
    offset
}
// TODO: free_mem

fn read_memory(ctx: &Ctx, offset: i32) -> Vec<u8> {
    // TODO: there must be a faster way to copy memory
    let start = offset as usize;
    let memory = &ctx.memory(0).view::<u8>()[start..];

    let mut result = Vec::new();
    let mut i = 0;
    while memory[i].get() != 0 {
        result.push(memory[i].get());
        i+=1;
    }
    result
}

fn write_memory(ctx: &Ctx, offset: i32, data: &[u8]) {
    let start = offset as usize;
    let end = start + data.len();
    // TODO: there must be a faster way to copy memory
    let memory = &ctx.memory(0).view::<u8>()[start..end];
    for i in 0..data.len() {
        memory[i].set(data[i])
    }
}

// TODO: this is so ugly, no clear idea how to make that callback to alloc in do_read
// There is support on Ctx for call_with_table_index: https://github.com/wasmerio/wasmer/pull/803
// But I cannot figure out how to get the table index for the function (allocate)
// Just guess it is 1???
//static mut INSTANCE: Option<Box<Instance> = None;

fn do_read(ctx: &mut Ctx, _dbref: i32, key: i32) -> i32 {
    let key = read_memory(ctx, key);
    let mut value: Option<Vec<u8>> = None;
    with_storage_from_context(ctx, |store| value = store.get(&key));
    match value {
        Some(_) => panic!("not implemented"),
        None => 0,
    }
}

fn do_write(ctx: &mut Ctx, _dbref: i32, key: i32, value: i32) {
    let key = read_memory(ctx, key);
    let value = read_memory(ctx, value);
    with_storage_from_context(ctx, |store| store.set(&key, &value));
}

//fn do_read(ctx: &mut Ctx, store: &mut MockStorage, key: i32) -> i32 {
//    let key = read_memory(ctx, key, 100);
//}
//
//fn do_write(ctx: &mut Ctx, store: &mut MockStorage, key: i32, value: i32) {
//}



/*** context data ****/

fn create_unmanaged_storage() ->*mut c_void {
    let state = Box::new(MockStorage::new());
    Box::into_raw(state) as *mut c_void
}

fn destroy_unmanaged_storage(ptr: *mut c_void) {
    let b = unsafe { Box::from_raw(ptr as *mut MockStorage) };
    mem::drop(b);
}

fn with_storage_from_context<F: FnMut(&mut MockStorage)>(ctx: &mut Ctx, mut func: F) {
    let mut b = unsafe { Box::from_raw(ctx.data as *mut MockStorage) };
    func(b.as_mut());
    mem::forget(b); // we do this to avoid cleanup
}
