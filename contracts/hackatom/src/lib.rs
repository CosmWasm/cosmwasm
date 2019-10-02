extern crate failure;
extern crate heapless;
extern crate serde;
extern crate serde_json;

use failure::Error;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use serde_json::{from_slice, to_vec};
use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::{c_char, c_void};

mod contract;
use contract::{init, send};

#[derive(Serialize, Deserialize)]
pub struct SendParams<'a> {
    contract_address: String,
    sender: String,
    #[serde(borrow)]
    msg: &'a RawValue,
    sent_funds: u64,
}

#[derive(Serialize, Deserialize)]
struct RegenSendMsg {}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum CosmosMsg {
    #[serde(rename = "cosmos-sdk/MsgSend")]
    SendTx {
        from_address: String,
        to_address: String,
        amount: Vec<SendAmount>,
    },
}

#[derive(Serialize, Deserialize)]
pub struct SendAmount {
    denom: String,
    amount: String,
}

#[derive(Serialize, Deserialize)]
enum ContractResult {
    #[serde(rename = "msgs")]
    Msgs(Vec<CosmosMsg>),
    #[serde(rename = "error")]
    Error(String),
}

extern "C" {
    fn c_read() -> *mut c_char;
    fn c_write(string: *mut c_char);
}

pub fn get_state() -> std::vec::Vec<u8> {
    let state: std::vec::Vec<u8>;

    unsafe {
        state = CStr::from_ptr(c_read()).to_bytes().to_vec();
    }

    state
}

pub fn set_state(state: Vec<u8>) {
    unsafe {
        c_write(CString::new(state).unwrap().into_raw());
    }
}

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

#[derive(Serialize, Deserialize)]
pub struct InitParams<'a> {
    contract_address: String,
    sender: String,
    #[serde(borrow)]
    msg: &'a RawValue,
    sent_funds: u64,
}

fn make_error_c_string<E: Into<Error>>(error: E) -> *mut c_char {
    let error: Error = error.into();
    CString::new(to_vec(&ContractResult::Error(error.to_string())).unwrap())
        .unwrap()
        .into_raw()
}

#[no_mangle]
pub extern "C" fn init_wrapper(params_ptr: *mut c_char) -> *mut c_char {
    let params: std::vec::Vec<u8>;

    unsafe {
        params = CStr::from_ptr(params_ptr).to_bytes().to_vec();
    }

    // Catches and formats deserialization errors
    let params: InitParams = match from_slice(&params) {
        Ok(params) => params,
        Err(e) => return make_error_c_string(e),
    };

    // Catches and formats errors from the logic
    let res = match init(params) {
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

#[no_mangle]
pub extern "C" fn send_wrapper(params_ptr: *mut c_char) -> *mut c_char {
    let params: std::vec::Vec<u8>;

    unsafe {
        params = CStr::from_ptr(params_ptr).to_bytes().to_vec();
    }

    // Catches and formats deserialization errors
    let params: SendParams = match from_slice(&params) {
        Ok(params) => params,
        Err(e) => return make_error_c_string(e),
    };

    // Catches and formats errors from the logic
    let res = match send(params) {
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
