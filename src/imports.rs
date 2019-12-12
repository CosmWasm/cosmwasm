#![cfg(target_arch = "wasm32")]

use std::ffi::c_void;
use std::str::from_utf8;
use std::vec::Vec;

use snafu::ResultExt;

use crate::memory::{alloc, build_slice, consume_slice, Slice};
use crate::storage::Storage;
use crate::errors::{ContractErr, Result, Utf8Err};

// this is the buffer we pre-allocate in get - we should configure this somehow later
static MAX_READ: usize = 2000;

// this should be plenty for any address representation
static ADDR_BUFFER: usize = 72;

extern "C" {
    fn c_read(key: *const c_void, value: *mut c_void) -> i32;
    fn c_write(key: *const c_void, value: *mut c_void);

    // we define two more functions that must be available...
    // they take a string and return to a preallocated buffer
    // returns negative on error, length of returned data on success
    fn canonical_address(human: *const c_void, canonical: *mut c_void) -> i32;
    fn humanize_address(canonical: *const c_void, human: *mut c_void) -> i32;
}

#[derive(Clone)]
pub struct ExternalStorage {}

impl ExternalStorage {
    pub fn new() -> ExternalStorage {
        ExternalStorage {}
    }
}

impl Storage for ExternalStorage {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let key = build_slice(key);
        let key_ptr = &*key as *const Slice as *const c_void;
        let value = alloc(MAX_READ);

        let read = unsafe { c_read(key_ptr, value) };
        if read < 0 {
            // TODO: try to read again with larger amount
            panic!("needed to read more data")
        } else if read == 0 {
            return None;
        }

        unsafe { consume_slice(value).ok() }.map(|mut d| {
            d.truncate(read as usize);
            d
        })
    }

    fn set(&mut self, key: &[u8], value: &[u8]) {
        // keep the boxes in scope, so we free it at the end (don't cast to pointers same line as build_slice)
        let key = build_slice(key);
        let key_ptr = &*key as *const Slice as *const c_void;
        let mut value = build_slice(value);
        let value_ptr = &mut *value as *mut Slice as *mut c_void;
        unsafe {
            c_write(key_ptr, value_ptr);
        }
    }
}


#[derive(Clone)]
pub struct ExternalAddresser {}

impl ExternalAddresser {
    pub fn new() -> ExternalAddresser {
        ExternalAddresser {}
    }
}

impl Addresser for ExternalAddresser {
    fn canonicalize(&self, human: &str) -> Result<Vec<u8>> {
        let send = build_slice(human.as_bytes());
        let send_ptr = &*send as *const Slice as *const c_void;
        let canon = alloc(ADDR_BUFFER);

        let read = unsafe { canonical_address(send_ptr, canon) };
        if read < 0 {
            return ContractErr { msg: "canonical_address returned error" }.fail();
        }

        let mut out = unsafe { consume_slice(canon)? };
        out.truncate(read as usize);
        Ok(out)
    }

    fn humanize(&self, canonical: &[u8]) -> Result<String> {
        let send = build_slice(canonical);
        let send_ptr = &*send as *const Slice as *const c_void;
        let human = alloc(ADDR_BUFFER);

        let read = unsafe { humanize_address(send_ptr, human) };
        if read < 0 {
            return ContractErr { msg: "humanize_address returned error" }.fail();
        }

        let mut out = unsafe { consume_slice(canon)? };
        out.truncate(read as usize);
        from_utf8(&out).context(Utf8Err{})?.to_string()
    }

        fn set(&mut self, key: &[u8], value: &[u8]) {
        // keep the boxes in scope, so we free it at the end (don't cast to pointers same line as build_slice)
        let key = build_slice(key);
        let key_ptr = &*key as *const Slice as *const c_void;
        let mut value = build_slice(value);
        let value_ptr = &mut *value as *mut Slice as *mut c_void;
        unsafe {
            c_write(key_ptr, value_ptr);
        }
    }
}
