#![cfg(target_arch = "wasm32")]

use std::ffi::c_void;
use std::vec::Vec;

use crate::encoding::Binary;
use crate::errors::{ContractErr, Result};
use crate::memory::{alloc, build_region, consume_region, Region};
use crate::traits::{Api, ReadonlyStorage, Storage};
use crate::types::{CanonicalAddr, HumanAddr};

// this is the buffer we pre-allocate in get - we should configure this somehow later
static MAX_READ: usize = 2000;

// this should be plenty for any address representation
static ADDR_BUFFER: usize = 72;

// This interface will compile into required Wasm imports.
// A complete documentation those functions is available in the VM that provides them:
// https://github.com/confio/cosmwasm/blob/0.7/lib/vm/src/instance.rs#L43
//
// TODO: use feature switches to enable precompile dependencies in the future,
// so contracts that need less
extern "C" {
    fn read_db(key: *const c_void, value: *mut c_void) -> i32;
    fn write_db(key: *const c_void, value: *mut c_void);
    fn canonicalize_address(human: *const c_void, canonical: *mut c_void) -> i32;
    fn humanize_address(canonical: *const c_void, human: *mut c_void) -> i32;
}

/// A stateless convenience wrapper around database imports provided by the VM.
/// Clone with caution: this might not do what you expect, in particular no data is cloned.
#[derive(Clone)]
pub struct ExternalStorage {}

impl ExternalStorage {
    pub fn new() -> ExternalStorage {
        ExternalStorage {}
    }
}

impl ReadonlyStorage for ExternalStorage {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let key = build_region(key);
        let key_ptr = &*key as *const Region as *const c_void;
        let value = alloc(MAX_READ);

        let read = unsafe { read_db(key_ptr, value) };
        if read == -1000002 {
            panic!("Allocated memory too small to hold the database value for the given key. \
                If this is causing trouble for you, have a look at https://github.com/confio/cosmwasm/issues/126");
        } else if read < 0 {
            panic!("An unknown error occurred in the read_db call.")
        } else if read == 0 {
            return None;
        }

        unsafe { consume_region(value).ok() }.map(|mut d| {
            d.truncate(read as usize);
            d
        })
    }
}

impl Storage for ExternalStorage {
    fn set(&mut self, key: &[u8], value: &[u8]) {
        // keep the boxes in scope, so we free it at the end (don't cast to pointers same line as build_region)
        let key = build_region(key);
        let key_ptr = &*key as *const Region as *const c_void;
        let mut value = build_region(value);
        let value_ptr = &mut *value as *mut Region as *mut c_void;
        unsafe {
            write_db(key_ptr, value_ptr);
        }
    }
}

/// A stateless convenience wrapper around imports provided by the VM
#[derive(Copy, Clone)]
pub struct ExternalApi {}

impl ExternalApi {
    pub fn new() -> ExternalApi {
        ExternalApi {}
    }
}

impl Api for ExternalApi {
    fn canonical_address(&self, human: &HumanAddr) -> Result<CanonicalAddr> {
        let send = build_region(human.as_str().as_bytes());
        let send_ptr = &*send as *const Region as *const c_void;
        let canon = alloc(ADDR_BUFFER);

        let read = unsafe { canonicalize_address(send_ptr, canon) };
        if read < 0 {
            return ContractErr {
                msg: "canonicalize_address returned error",
            }
            .fail();
        }

        let mut out = unsafe { consume_region(canon)? };
        out.truncate(read as usize);
        Ok(CanonicalAddr(Binary(out)))
    }

    fn human_address(&self, canonical: &CanonicalAddr) -> Result<HumanAddr> {
        let send = build_region(canonical.as_slice());
        let send_ptr = &*send as *const Region as *const c_void;
        let human = alloc(ADDR_BUFFER);

        let read = unsafe { humanize_address(send_ptr, human) };
        if read < 0 {
            return ContractErr {
                msg: "humanize_address returned error",
            }
            .fail();
        }

        let mut out = unsafe { consume_region(human)? };
        out.truncate(read as usize);
        // we know input was correct when created, so let's save some bytes
        let result = unsafe { String::from_utf8_unchecked(out) };
        Ok(HumanAddr(result))
    }
}
