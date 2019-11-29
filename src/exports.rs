#![cfg(target_arch = "wasm32")]

//! exports exposes the public wasm API
//! allocate and deallocate should be re-exported as is
//! do_init and do_wrapper should be wrapped with a extern "C" entry point
//! including the contract-specific init/handle function pointer.
use snafu::ResultExt;
use std::fmt::Display;
use std::os::raw::c_void;
use std::vec::Vec;

use crate::errors::{Error, ParseErr, SerializeErr};
use crate::imports::ExternalStorage;
use crate::memory::{alloc, consume_slice, release_buffer};
use crate::serde::{from_slice, to_vec};
use crate::types::{ContractResult, Params, QueryResponse, QueryResult, Response};

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
    // auto-drop slice on function end
    let _ = unsafe { consume_slice(pointer) };
}

// do_init should be wrapped in an external "C" export, containing a contract-specific function as arg
pub fn do_init<T: Display + From<Error>>(
    init_fn: &dyn Fn(&mut ExternalStorage, Params, Vec<u8>) -> Result<Response, T>,
    params_ptr: *mut c_void,
    msg_ptr: *mut c_void,
) -> *mut c_void {
    match _do_init(init_fn, params_ptr, msg_ptr) {
        Ok(res) => res,
        Err(err) => make_error_c_string(err),
    }
}

// do_handle should be wrapped in an external "C" export, containing a contract-specific function as arg
pub fn do_handle<T: Display + From<Error>>(
    handle_fn: &dyn Fn(&mut ExternalStorage, Params, Vec<u8>) -> Result<Response, T>,
    params_ptr: *mut c_void,
    msg_ptr: *mut c_void,
) -> *mut c_void {
    match _do_handle(handle_fn, params_ptr, msg_ptr) {
        Ok(res) => res,
        Err(err) => make_error_c_string(err),
    }
}

// do_query should be wrapped in an external "C" export, containing a contract-specific function as arg
pub fn do_query<T: Display + From<Error>>(
    query_fn: &dyn Fn(&ExternalStorage, Vec<u8>) -> Result<QueryResponse, T>,
    msg_ptr: *mut c_void,
) -> *mut c_void {
    match _do_query(query_fn, msg_ptr) {
        Ok(res) => res,
        Err(err) => make_query_error_c_string(err),
    }
}

fn _do_init<T: Display + From<Error>>(
    init_fn: &dyn Fn(&mut ExternalStorage, Params, Vec<u8>) -> Result<Response, T>,
    params_ptr: *mut c_void,
    msg_ptr: *mut c_void,
) -> Result<*mut c_void, T> {
    let params: Vec<u8> = unsafe { consume_slice(params_ptr)? };
    let msg: Vec<u8> = unsafe { consume_slice(msg_ptr)? };

    let params: Params = from_slice(&params).context(ParseErr { kind: "Params" })?;
    let mut store = ExternalStorage::new();
    let res = init_fn(&mut store, params, msg)?;
    let json = to_vec(&ContractResult::Ok(res)).context(SerializeErr {
        kind: "ContractResult",
    })?;
    Ok(release_buffer(json))
}

fn _do_handle<T: Display + From<Error>>(
    handle_fn: &dyn Fn(&mut ExternalStorage, Params, Vec<u8>) -> Result<Response, T>,
    params_ptr: *mut c_void,
    msg_ptr: *mut c_void,
) -> Result<*mut c_void, T> {
    let params: Vec<u8> = unsafe { consume_slice(params_ptr)? };
    let msg: Vec<u8> = unsafe { consume_slice(msg_ptr)? };

    let params: Params = from_slice(&params).context(ParseErr { kind: "Params" })?;
    let mut store = ExternalStorage::new();
    let res = handle_fn(&mut store, params, msg)?;
    let json = to_vec(&ContractResult::Ok(res)).context(SerializeErr {
        kind: "ContractResult",
    })?;
    Ok(release_buffer(json))
}

fn _do_query<T: Display + From<Error>>(
    query_fn: &dyn Fn(&ExternalStorage, Vec<u8>) -> Result<QueryResponse, T>,
    msg_ptr: *mut c_void,
) -> Result<*mut c_void, T> {
    let msg: Vec<u8> = unsafe { consume_slice(msg_ptr)? };

    let store = ExternalStorage::new();
    let res = query_fn(&store, msg)?;
    let json = to_vec(&QueryResult::Ok(res)).context(SerializeErr {
        kind: "QueryResult",
    })?;
    Ok(release_buffer(json))
}

fn make_error_c_string<T: Display>(error: T) -> *mut c_void {
    let v = to_vec(&ContractResult::Err(error.to_string())).unwrap();
    release_buffer(v)
}

fn make_query_error_c_string<T: Display>(error: T) -> *mut c_void {
    let v = to_vec(&QueryResult::Err(error.to_string())).unwrap();
    release_buffer(v)
}
