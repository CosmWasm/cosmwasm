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

// do_init should be wrapped in an external "C" export, containing a contract-specific function as arg
pub fn do_init(
    init_fn: &dyn Fn(&mut ExternalStorage, Params, Vec<u8>) -> Result<Vec<CosmosMsg>, Error>,
    params_ptr: *mut c_void,
    msg_ptr: *mut c_void,
) -> *mut c_void {
    match _do_init(init_fn, params_ptr, msg_ptr) {
        Ok(res) => res,
        Err(err) => make_error_c_string(err),
    }
}

// do_handle should be wrapped in an external "C" export, containing a contract-specific function as arg
pub fn do_handle(
    handle_fn: &dyn Fn(&mut ExternalStorage, Params, Vec<u8>) -> Result<Vec<CosmosMsg>, Error>,
    params_ptr: *mut c_void,
    msg_ptr: *mut c_void,
) -> *mut c_void {
    match _do_handle(handle_fn, params_ptr, msg_ptr) {
        Ok(res) => res,
        Err(err) => make_error_c_string(err),
    }
}
fn _do_init(
    init_fn: &dyn Fn(&mut ExternalStorage, Params, Vec<u8>) -> Result<Vec<CosmosMsg>, Error>,
    params_ptr: *mut c_void,
    msg_ptr: *mut c_void,
) -> Result<*mut c_void, Error> {
    let params: Vec<u8> = consume_slice(params_ptr);
    let msg: Vec<u8> = consume_slice(msg_ptr);

    let params: Params = from_slice(&params)?;
    let mut store = ExternalStorage::new();
    let msgs = init_fn(&mut store, params, msg)?;
    let json = to_vec(&ContractResult::Msgs(msgs))?;
    Ok(release_buffer(json))
}

fn _do_handle(
    handle_fn: &dyn Fn(&mut ExternalStorage, Params, Vec<u8>) -> Result<Vec<CosmosMsg>, Error>,
    params_ptr: *mut c_void,
    msg_ptr: *mut c_void,
) -> Result<*mut c_void, Error> {
    let params: Vec<u8> = consume_slice(params_ptr);
    let msg: Vec<u8> = consume_slice(msg_ptr);

    let params: Params = from_slice(&params)?;
    let mut store = ExternalStorage::new();
    let msgs = handle_fn(&mut store, params, msg)?;
    let json = to_vec(&ContractResult::Msgs(msgs))?;
    Ok(release_buffer(json))
}

fn make_error_c_string<E: Into<Error>>(error: E) -> *mut c_void {
    let error: Error = error.into();
    let v = to_vec(&ContractResult::Error(error.to_string())).unwrap();
    release_buffer(v)
}
