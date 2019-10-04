extern crate hackatom;

use std::fs;
use std::mem;
use std::str::from_utf8;
use std::ffi::c_void;

use wasmer_runtime::{compile_with, Ctx, Func, func, imports};
use wasmer_runtime_core::{Instance};
use wasmer_clif_backend::CraneliftCompiler;

use hackatom::contract::{RegenInitMsg};
use hackatom::imports::Storage;
use hackatom::memory::Slice;
use hackatom::mock::{MockStorage};
use hackatom::types::{coin, mock_params};

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

/****** read/write to wasm memory buffer ****/

// write_mem allocates memory in the instance and copies the given data in
// returns the memory offset, to be passed as an argument
// panics on any error (TODO, use result?)
fn allocate(instance: &mut Instance, data: &[u8]) -> i32 {
    // allocate
    let alloc: Func<(i32), (i32)> = instance.func("allocate").unwrap();
    let ptr = alloc.call(data.len() as i32).unwrap();
    write_memory(instance.context(), ptr, data);
    ptr
}

fn read_memory(ctx: &Ctx, ptr: i32) -> Vec<u8> {
    let slice = to_slice(ctx, ptr);
    let (start, end) = (slice.offset, slice.offset+slice.len);
    let memory = &ctx.memory(0).view::<u8>()[start..end];

    // TODO: there must be a faster way to copy memory
    let mut result = vec![0u8; slice.len];
    for i in 0..slice.len {
        result[i] = memory[i].get();
    }
    result
}

// write_memory returns how many bytes written on success
// negative result is how many bytes requested if too small
fn write_memory(ctx: &Ctx, ptr: i32, data: &[u8]) -> i32 {
    let slice = to_slice(ctx, ptr);
    if data.len() > slice.len {
        return -(data.len() as i32);
    }
    if data.len() == 0 {
        return 0;
    }

    let (start, end) = (slice.offset, slice.offset+slice.len);
    let memory = &ctx.memory(0).view::<u8>()[start..end];
    // TODO: there must be a faster way to copy memory
    for i in 0..data.len() {
        memory[i].set(data[i])
    }
    data.len() as i32
}

// to_slice reads in a ptr to slice in wasm memory and constructs the object we can use to access it
fn to_slice(ctx: &Ctx, ptr: i32) -> Slice {
    let buf_ptr = (ptr / 4) as usize;  // convert from u8 to i32 offset
    let memory = &ctx.memory(0).view::<i32>();
    Slice {
        offset: memory[buf_ptr].get() as usize,
        len: memory[buf_ptr+1].get() as usize,
    }
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
