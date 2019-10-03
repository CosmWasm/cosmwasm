// api.rs includes the public wasm API
// when included, this whole file should be wrapped by
// #[cfg(target_arch = "wasm32")]
use failure::Error;
use serde_json::{from_slice, to_vec};
use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::{c_char, c_void};
use std::vec::Vec;

use crate::imports::{ExternalStorage};
use crate::types::{ContractResult, CosmosMsg, Param, Params};

#[no_mangle]
pub extern "C" fn allocate(size: usize) -> *mut c_void {
    let mut buffer = Vec::with_capacity(size);
    let pointer = buffer.as_mut_ptr();
    mem::forget(buffer);

    pointer as *mut c_void
}

#[no_mangle]
pub extern "C" fn deallocate(pointer: *mut c_void, capacity: usize) {
    unsafe {
        let _ = Vec::from_raw_parts(pointer, 0, capacity);
    }
}

// init should be wrapped in an external "C" export, containing a contract-specific function as arg
pub fn init(init_fn: &dyn Fn(&mut ExternalStorage, Params, Vec<u8>) -> Result<Vec<CosmosMsg>, Error>, dbref: i32, params_ptr: *mut c_char, msg_ptr: *mut c_char) -> *mut c_char {
    let params: Vec<u8>;
    let msg: Vec<u8>;

    unsafe {
        params = CStr::from_ptr(params_ptr).to_bytes().to_vec();
        msg = CStr::from_ptr(msg_ptr).to_bytes().to_vec();
    }

    // Catches and formats deserialization errors
    let params: Params = match from_slice(&params) {
        Ok(params) => params,
        Err(e) => return make_error_c_string(e),
    };

    // Catches and formats errors from the logic
    let mut store = ExternalStorage::new(dbref);
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

    // Catches and formats CString errors
    let res = match CString::new(res) {
        Ok(res) => res,
        Err(e) => return make_error_c_string(e),
    };

    res.into_raw()
}

// send should be wrapped in an external "C" export, containing a contract-specific function as arg
pub fn send(send_fn: &dyn Fn(&mut ExternalStorage, Params, Vec<u8>) -> Result<Vec<CosmosMsg>, Error>, dbref: i32, params_ptr: *mut c_char, msg_ptr: *mut c_char) -> *mut c_char {
    let params: Vec<u8>;
    let msg: Vec<u8>;

    unsafe {
        params = CStr::from_ptr(params_ptr).to_bytes().to_vec();
        msg = CStr::from_ptr(msg_ptr).to_bytes().to_vec();
    }

    // Catches and formats deserialization errors
    let params: Params = match from_slice(&params) {
        Ok(params) => params,
        Err(e) => return make_error_c_string(e),
    };

    // Catches and formats errors from the logic
    let mut store = ExternalStorage::new(dbref);
    let res = match send_fn(&mut store, params, msg) {
        Ok(msgs) => ContractResult::Msgs(msgs),
        Err(e) => return make_error_c_string(e),
    };

    // Catches and formats serialization errors
    let res = match to_vec(&res) {
        Ok(res) => res,
        Err(e) => return make_error_c_string(e),
    };

    // Catches and formats CString errors
    let res = match CString::new(res) {
        Ok(res) => res,
        Err(e) => return make_error_c_string(e),
    };

    res.into_raw()
}

fn make_error_c_string<E: Into<Error>>(error: E) -> *mut c_char {
    let error: Error = error.into();
    CString::new(to_vec(&ContractResult::Error(error.to_string())).unwrap())
        .unwrap()
        .into_raw()
}

