#![cfg(target_arch = "wasm32")]

use std::ffi::c_void;
use std::str::from_utf8;
use std::vec::Vec;

use snafu::ResultExt;

use crate::errors::{ContractErr, Result, Utf8Err};
use crate::memory::{alloc, build_slice, consume_slice, Slice};
use crate::traits::{Api, Extern, Storage};

// this is the buffer we pre-allocate in get - we should configure this somehow later
static MAX_READ: usize = 2000;

// this should be plenty for any address representation
static ADDR_BUFFER: usize = 72;

// TODO: use feature switches to enable precompile dependencies in the future,
// so contracts that need less
extern "C" {
    // these are needed for storage
    fn c_read(key: *const c_void, value: *mut c_void) -> i32;
    fn c_write(key: *const c_void, value: *mut c_void);

    // we define two more functions that must be available...
    // they take a string and return to a preallocated buffer
    // returns negative on error, length of returned data on success
    fn c_canonical_address(human: *const c_void, canonical: *mut c_void) -> i32;
    fn c_human_address(canonical: *const c_void, human: *mut c_void) -> i32;
}

// dependencies are all external requirements that can be injected in a real-wasm contract
pub fn dependencies() -> Extern<ExternalStorage, ExternalApi> {
    Extern {
        storage: ExternalStorage::new(),
        api: ExternalApi::new(),
    }
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

#[derive(Copy, Clone)]
pub struct ExternalApi {}

impl ExternalApi {
    pub fn new() -> ExternalApi {
        ExternalApi {}
    }
}

impl Api for ExternalApi {
    fn canonical_address(&self, human: &str) -> Result<Vec<u8>> {
        let send = build_slice(human.as_bytes());
        let send_ptr = &*send as *const Slice as *const c_void;
        let canon = alloc(ADDR_BUFFER);

        let read = unsafe { c_canonical_address(send_ptr, canon) };
        if read < 0 {
            return ContractErr {
                msg: "canonical_address returned error",
            }
            .fail();
        }

        let mut out = unsafe { consume_slice(canon)? };
        out.truncate(read as usize);
        Ok(out)
    }

    fn human_address(&self, canonical: &[u8]) -> Result<String> {
        let send = build_slice(canonical);
        let send_ptr = &*send as *const Slice as *const c_void;
        let human = alloc(ADDR_BUFFER);

        let read = unsafe { c_human_address(send_ptr, human) };
        if read < 0 {
            return ContractErr {
                msg: "humanize_address returned error",
            }
            .fail();
        }

        let mut out = unsafe { consume_slice(human)? };
        out.truncate(read as usize);
        let result = from_utf8(&out).context(Utf8Err {})?.to_string();
        Ok(result)
    }
}
