use std::ffi::c_void;
use std::vec::Vec;

use crate::encoding::Binary;
use crate::errors::{generic_err, StdResult};
#[cfg(feature = "iterator")]
use crate::iterator::{Order, KV};
use crate::memory::{alloc, build_region, consume_region, Region};
use crate::serde::from_slice;
use crate::traits::{Api, Querier, QuerierResult, ReadonlyStorage, Storage};
use crate::types::{CanonicalAddr, HumanAddr};

/// An upper bound for typical canonical address lengths (e.g. 20 in Cosmos SDK/Ethereum or 32 in Nano/Substrate)
const CANONICAL_ADDRESS_BUFFER_LENGTH: usize = 32;
/// An upper bound for typical human readable address formats (e.g. 42 for Ethereum hex addresses or 90 for bech32)
const HUMAN_ADDRESS_BUFFER_LENGTH: usize = 90;

// This interface will compile into required Wasm imports.
// A complete documentation those functions is available in the VM that provides them:
// https://github.com/confio/cosmwasm/blob/0.7/lib/vm/src/instance.rs#L43
extern "C" {
    fn db_read(key: *const c_void) -> u32;
    fn db_write(key: *const c_void, value: *mut c_void) -> i32;
    fn db_remove(key: *const c_void) -> i32;

    // scan creates an iterator, which can be read by consecutive next() calls
    #[cfg(feature = "iterator")]
    fn db_scan(start: *const c_void, end: *const c_void, order: i32) -> i32;
    #[cfg(feature = "iterator")]
    fn db_next(iterator_id: u32) -> u32;

    fn canonicalize_address(human: *const c_void, canonical: *mut c_void) -> i32;
    fn humanize_address(canonical: *const c_void, human: *mut c_void) -> i32;

    // query_chain will launch a query on the chain (import)
    // different than query which will query the state of the contract (export)
    fn query_chain(request: *const c_void) -> u32;
}

/// A stateless convenience wrapper around database imports provided by the VM.
/// This cannot be cloned as it would not copy any data. If you need to clone this, it indicates a flaw in your logic.
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

        let read = unsafe { db_read(key_ptr) };
        if read == 0 {
            // key does not exist in external storage
            return None;
        }

        let value_ptr = read as *mut c_void;
        let data = unsafe { consume_region(value_ptr) };
        Some(data)
    }

    #[cfg(feature = "iterator")]
    fn range(
        &self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> StdResult<Box<dyn Iterator<Item = KV>>> {
        // start and end (Regions) must remain in scope as long as the start_ptr / end_ptr do
        // thus they are not inside a block
        let start = start.map(|s| build_region(s));
        let start_ptr = match start {
            Some(reg) => &*reg as *const Region as *const c_void,
            None => std::ptr::null(),
        };
        let end = end.map(|e| build_region(e));
        let end_ptr = match end {
            Some(reg) => &*reg as *const Region as *const c_void,
            None => std::ptr::null(),
        };
        let order = order as i32;

        let scan_result = unsafe { db_scan(start_ptr, end_ptr, order) };
        if scan_result < 0 {
            return Err(generic_err(format!(
                "Error creating iterator (via db_scan). Error code: {}",
                scan_result
            )));
        }
        let iter = ExternalIterator {
            iterator_id: scan_result as u32, // Cast is safe since we tested for negative values above
        };
        Ok(Box::new(iter))
    }
}

impl Storage for ExternalStorage {
    fn set(&mut self, key: &[u8], value: &[u8]) -> StdResult<()> {
        // keep the boxes in scope, so we free it at the end (don't cast to pointers same line as build_region)
        let key = build_region(key);
        let key_ptr = &*key as *const Region as *const c_void;
        let mut value = build_region(value);
        let value_ptr = &mut *value as *mut Region as *mut c_void;
        let result = unsafe { db_write(key_ptr, value_ptr) };
        if result < 0 {
            return Err(generic_err(format!(
                "Error writing to database. Error code: {}",
                result
            )));
        }
        Ok(())
    }

    fn remove(&mut self, key: &[u8]) -> StdResult<()> {
        // keep the boxes in scope, so we free it at the end (don't cast to pointers same line as build_region)
        let key = build_region(key);
        let key_ptr = &*key as *const Region as *const c_void;
        let result = unsafe { db_remove(key_ptr) };
        if result < 0 {
            return Err(generic_err(format!(
                "Error deleting from database. Error code: {}",
                result
            )));
        }
        Ok(())
    }
}

#[cfg(feature = "iterator")]
/// ExternalIterator makes a call out to next.
/// We use the pointer to differentiate between multiple open iterators.
struct ExternalIterator {
    iterator_id: u32,
}

#[cfg(feature = "iterator")]
impl Iterator for ExternalIterator {
    type Item = KV;

    fn next(&mut self) -> Option<Self::Item> {
        let next_result = unsafe { db_next(self.iterator_id) };
        let kv_region_ptr = next_result as *mut c_void;
        let mut kv = unsafe { consume_region(kv_region_ptr) };

        // The KV region uses the format value || key || keylen, where keylen is a fixed size big endian u32 value
        let keylen = u32::from_be_bytes([
            kv[kv.len() - 4],
            kv[kv.len() - 3],
            kv[kv.len() - 2],
            kv[kv.len() - 1],
        ]) as usize;
        if keylen == 0 {
            return None;
        }

        kv.truncate(kv.len() - 4);
        let key = kv.split_off(kv.len() - keylen);
        let value = kv;
        Some((key, value))
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
    fn canonical_address(&self, human: &HumanAddr) -> StdResult<CanonicalAddr> {
        let send = build_region(human.as_str().as_bytes());
        let send_ptr = &*send as *const Region as *const c_void;
        let canon = alloc(CANONICAL_ADDRESS_BUFFER_LENGTH);

        let read = unsafe { canonicalize_address(send_ptr, canon) };
        if read < 0 {
            return Err(generic_err("canonicalize_address returned error"));
        }

        let out = unsafe { consume_region(canon) };
        Ok(CanonicalAddr(Binary(out)))
    }

    fn human_address(&self, canonical: &CanonicalAddr) -> StdResult<HumanAddr> {
        let send = build_region(canonical.as_slice());
        let send_ptr = &*send as *const Region as *const c_void;
        let human = alloc(HUMAN_ADDRESS_BUFFER_LENGTH);

        let read = unsafe { humanize_address(send_ptr, human) };
        if read < 0 {
            return Err(generic_err("humanize_address returned error"));
        }

        let out = unsafe { consume_region(human) };
        // we know input was correct when created, so let's save some bytes
        let result = unsafe { String::from_utf8_unchecked(out) };
        Ok(HumanAddr(result))
    }
}

/// A stateless convenience wrapper around imports provided by the VM
pub struct ExternalQuerier {}

impl ExternalQuerier {
    pub fn new() -> ExternalQuerier {
        ExternalQuerier {}
    }
}

impl Querier for ExternalQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let req = build_region(bin_request);
        let request_ptr = &*req as *const Region as *const c_void;

        let response_ptr = unsafe { query_chain(request_ptr) };

        let response = unsafe { consume_region(response_ptr as *mut c_void) };
        from_slice(&response).unwrap_or_else(|err| Ok(Err(err)))
    }
}
