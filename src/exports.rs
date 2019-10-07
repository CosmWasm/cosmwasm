// api.rs includes the public wasm API
// when included, this whole file should be wrapped by
// #[cfg(target_arch = "wasm32")]
use failure::Error;
use serde_json::{from_slice, to_vec};
use std::mem;
use std::os::raw::c_void;
use std::vec::Vec;

use crate::imports::ExternalStorage;
use crate::memory::{alloc, consume_slice, release_buffer};
use crate::types::{ContractResult, CosmosMsg, Params};

// allocate reserves the given number of bytes in wasm memory and returns a pointer
// to a slice defining this data. This space is managed by the calling process
// and should be accompanied by a corresponding deallocate
#[no_mangle]
pub extern "C" fn allocate(size: usize) -> *mut c_void {
    alloc(size)
}

// deallocate expects a pointer to a Slice created with allocate.
// It will free both the Slice and the memory referenced by the slice.
#[no_mangle]
pub extern "C" fn deallocate(pointer: *mut c_void) {
    mem::drop(consume_slice(pointer));
}

// init should be wrapped in an external "C" export, containing a contract-specific function as arg
pub fn init(
    init_fn: &dyn Fn(&mut ExternalStorage, Params, Vec<u8>) -> Result<Vec<CosmosMsg>, Error>,
    params_ptr: *mut c_void,
    msg_ptr: *mut c_void,
) -> *mut c_void {
    let params: Vec<u8> = consume_slice(params_ptr);
    let msg: Vec<u8> = consume_slice(msg_ptr);

    // Catches and formats deserialization errors
    let params: Params = match from_slice(&params) {
        Ok(params) => params,
        Err(e) => return make_error_c_string(e),
    };

    // Catches and formats errors from the logic
    let mut store = ExternalStorage::new();
    let init_res = init_fn(&mut store, params, msg);
    let res = match init_res {
        Ok(msgs) => ContractResult::Msgs(msgs),
        Err(e) => return make_error_c_string(e),
    };

    // Catches and formats serialization errors
    let res = match to_vec(&res) {
        Ok(res) => res,
        Err(e) => return make_error_c_string(e),
    };

    release_buffer(res)
}

// handle should be wrapped in an external "C" export, containing a contract-specific function as arg
pub fn handle(
    handle_fn: &dyn Fn(&mut ExternalStorage, Params, Vec<u8>) -> Result<Vec<CosmosMsg>, Error>,
    params_ptr: *mut c_void,
    msg_ptr: *mut c_void,
) -> *mut c_void {
    let params: Vec<u8> = consume_slice(params_ptr);
    let msg: Vec<u8> = consume_slice(msg_ptr);

    // Catches and formats deserialization errors
    let params: Params = match from_slice(&params) {
        Ok(params) => params,
        Err(e) => return make_error_c_string(e),
    };

    // Catches and formats errors from the logic
    let mut store = ExternalStorage::new();
    let res = match handle_fn(&mut store, params, msg) {
        Ok(msgs) => ContractResult::Msgs(msgs),
        Err(e) => return make_error_c_string(e),
    };

    // Catches and formats serialization errors
    let res = match to_vec(&res) {
        Ok(res) => res,
        Err(e) => return make_error_c_string(e),
    };

    release_buffer(res)
}

fn make_error_c_string<E: Into<Error>>(error: E) -> *mut c_void {
    let error: Error = error.into();
    let v = to_vec(&ContractResult::Error(error.to_string())).unwrap();
    release_buffer(v)
}
