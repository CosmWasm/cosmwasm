#![cfg(target_arch = "wasm32")]

use std::ffi::c_void;
use std::vec::Vec;

use crate::memory::{alloc, build_slice, consume_slice, Slice};
use crate::storage::Storage;

// this is the buffer we pre-allocate in get - we should configure this somehow later
static MAX_READ: usize = 2000;

extern "C" {
    fn c_read(key: *const c_void, value: *mut c_void) -> i32;
    fn c_write(key: *const c_void, value: *mut c_void);
}

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
