// api.rs includes the public wasm API
// when included, this whole file should be wrapped by
// #[cfg(target_arch = "wasm32")]
use failure::Error;
use serde_json::{from_slice, to_vec};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char};
use std::vec::Vec;

use crate::{contract};
use crate::imports::{ExternalStorage};
use crate::types::{ContractResult, CosmosMsg, InitParams, SendParams};

fn make_error_c_string<E: Into<Error>>(error: E) -> *mut c_char {
    let error: Error = error.into();
    CString::new(to_vec(&ContractResult::Error(error.to_string())).unwrap())
        .unwrap()
        .into_raw()
}

pub fn init(init_fn: &dyn Fn(ExternalStorage, InitParams, Vec<u8>) -> Result<Vec<CosmosMsg>, Error>, params_ptr: *mut c_char, msg_ptr: *mut c_char) -> *mut c_char {
    let params: Vec<u8>;
    let msg: Vec<u8>;

    unsafe {
        params = CStr::from_ptr(params_ptr).to_bytes().to_vec();
        msg = CStr::from_ptr(msg_ptr).to_bytes().to_vec();
    }

    // Catches and formats deserialization errors
    let params: InitParams = match from_slice(&params) {
        Ok(params) => params,
        Err(e) => return make_error_c_string(e),
    };

    // Catches and formats errors from the logic
    let store = ExternalStorage{};
    let init_res = init_fn(store, params, msg);
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

pub fn send(params_ptr: *mut c_char, msg_ptr: *mut c_char) -> *mut c_char {
    let params: Vec<u8>;
    let msg: Vec<u8>;

    unsafe {
        params = CStr::from_ptr(params_ptr).to_bytes().to_vec();
        msg = CStr::from_ptr(msg_ptr).to_bytes().to_vec();
    }

    // Catches and formats deserialization errors
    let params: SendParams = match from_slice(&params) {
        Ok(params) => params,
        Err(e) => return make_error_c_string(e),
    };

    // Catches and formats errors from the logic
    let store = ExternalStorage{};
    let res = match contract::send(store, params, msg) {
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
