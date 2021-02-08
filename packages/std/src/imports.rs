use std::vec::Vec;

use crate::addresses::{CanonicalAddr, HumanAddr};
use crate::binary::Binary;
use crate::errors::{StdError, StdResult, SystemError};
use crate::memory::{alloc, build_region, consume_region, Region};
use crate::results::SystemResult;
#[cfg(feature = "iterator")]
use crate::sections::decode_sections2;
use crate::serde::from_slice;
use crate::traits::{Api, Querier, QuerierResult, Storage};
#[cfg(feature = "iterator")]
use crate::{
    iterator::{Order, KV},
    memory::get_optional_region_address,
};

/// An upper bound for typical canonical address lengths (e.g. 20 in Cosmos SDK/Ethereum or 32 in Nano/Substrate)
const CANONICAL_ADDRESS_BUFFER_LENGTH: usize = 32;
/// An upper bound for typical human readable address formats (e.g. 42 for Ethereum hex addresses or 90 for bech32)
const HUMAN_ADDRESS_BUFFER_LENGTH: usize = 90;

// This interface will compile into required Wasm imports.
// A complete documentation those functions is available in the VM that provides them:
// https://github.com/confio/cosmwasm/blob/0.7/lib/vm/src/instance.rs#L43
extern "C" {
    fn db_read(key: u32) -> u32;
    fn db_write(key: u32, value: u32);
    fn db_remove(key: u32);

    // scan creates an iterator, which can be read by consecutive next() calls
    #[cfg(feature = "iterator")]
    fn db_scan(start_ptr: u32, end_ptr: u32, order: i32) -> u32;
    #[cfg(feature = "iterator")]
    fn db_next(iterator_id: u32) -> u32;

    fn canonicalize_address(source_ptr: u32, destination_ptr: u32) -> u32;
    fn humanize_address(source_ptr: u32, destination_ptr: u32) -> u32;

    fn verify_secp256k1(message_hash_ptr: u32, signature_ptr: u32, public_key_ptr: u32) -> u32;

    fn debug(source_ptr: u32);

    /// Executes a query on the chain (import). Not to be confused with the
    /// query export, which queries the state of the contract.
    fn query_chain(request: u32) -> u32;
}

/// A stateless convenience wrapper around database imports provided by the VM.
/// This cannot be cloned as it would not copy any data. If you need to clone this, it indicates a flaw in your logic.
pub struct ExternalStorage {}

impl ExternalStorage {
    pub fn new() -> ExternalStorage {
        ExternalStorage {}
    }
}

impl Storage for ExternalStorage {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let key = build_region(key);
        let key_ptr = &*key as *const Region as u32;

        let read = unsafe { db_read(key_ptr) };
        if read == 0 {
            // key does not exist in external storage
            return None;
        }

        let value_ptr = read as *mut Region;
        let data = unsafe { consume_region(value_ptr) };
        Some(data)
    }

    fn set(&mut self, key: &[u8], value: &[u8]) {
        if value.is_empty() {
            panic!("TL;DR: Value must not be empty in Storage::set but in most cases you can use Storage::remove instead. Long story: Getting empty values from storage is not well supported at the moment. Some of our internal interfaces cannot differentiate between a non-existent key and an empty value. Right now, you cannot rely on the behaviour of empty values. To protect you from trouble later on, we stop here. Sorry for the inconvenience! We highly welcome you to contribute to CosmWasm, making this more solid one way or the other.");
        }

        // keep the boxes in scope, so we free it at the end (don't cast to pointers same line as build_region)
        let key = build_region(key);
        let key_ptr = &*key as *const Region as u32;
        let mut value = build_region(value);
        let value_ptr = &mut *value as *mut Region as u32;
        unsafe { db_write(key_ptr, value_ptr) };
    }

    fn remove(&mut self, key: &[u8]) {
        // keep the boxes in scope, so we free it at the end (don't cast to pointers same line as build_region)
        let key = build_region(key);
        let key_ptr = &*key as *const Region as u32;
        unsafe { db_remove(key_ptr) };
    }

    #[cfg(feature = "iterator")]
    fn range(
        &self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = KV>> {
        // There is lots of gotchas on turning options into regions for FFI, thus this design
        // See: https://github.com/CosmWasm/cosmwasm/pull/509
        let start_region = start.map(build_region);
        let end_region = end.map(build_region);
        let start_region_addr = get_optional_region_address(&start_region.as_ref());
        let end_region_addr = get_optional_region_address(&end_region.as_ref());
        let iterator_id = unsafe { db_scan(start_region_addr, end_region_addr, order as i32) };
        let iter = ExternalIterator { iterator_id };
        Box::new(iter)
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
        let kv_region_ptr = next_result as *mut Region;
        let kv = unsafe { consume_region(kv_region_ptr) };
        let (key, value) = decode_sections2(kv);
        if key.len() == 0 {
            None
        } else {
            Some((key, value))
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
    fn canonical_address(&self, human: &HumanAddr) -> StdResult<CanonicalAddr> {
        let send = build_region(human.as_str().as_bytes());
        let send_ptr = &*send as *const Region as u32;
        let canon = alloc(CANONICAL_ADDRESS_BUFFER_LENGTH);

        let result = unsafe { canonicalize_address(send_ptr, canon as u32) };
        if result != 0 {
            let error = unsafe { consume_string_region_written_by_vm(result as *mut Region) };
            return Err(StdError::generic_err(format!(
                "canonicalize_address errored: {}",
                error
            )));
        }

        let out = unsafe { consume_region(canon) };
        Ok(CanonicalAddr(Binary(out)))
    }

    fn human_address(&self, canonical: &CanonicalAddr) -> StdResult<HumanAddr> {
        let send = build_region(&canonical);
        let send_ptr = &*send as *const Region as u32;
        let human = alloc(HUMAN_ADDRESS_BUFFER_LENGTH);

        let result = unsafe { humanize_address(send_ptr, human as u32) };
        if result != 0 {
            let error = unsafe { consume_string_region_written_by_vm(result as *mut Region) };
            return Err(StdError::generic_err(format!(
                "humanize_address errored: {}",
                error
            )));
        }

        let address = unsafe { consume_string_region_written_by_vm(human) };
        Ok(address.into())
    }

    fn secp256k1_verify(
        &self,
        message_hash: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> StdResult<bool> {
        let hash_send = build_region(message_hash);
        let hash_send_ptr = &*hash_send as *const Region as u32;
        let sig_send = build_region(signature);
        let sig_send_ptr = &*sig_send as *const Region as u32;
        let pubkey_send = build_region(public_key);
        let pubkey_send_ptr = &*pubkey_send as *const Region as u32;

        let result = unsafe { verify_secp256k1(hash_send_ptr, sig_send_ptr, pubkey_send_ptr) };
        match result {
            1 => Ok(true),
            0 => Ok(false),
            x => panic!(format!("unexpected verify_secp256k1 return value: {}", x)),
        }
    }

    fn debug(&self, message: &str) {
        // keep the boxes in scope, so we free it at the end (don't cast to pointers same line as build_region)
        let region = build_region(message.as_bytes());
        let region_ptr = region.as_ref() as *const Region as u32;
        unsafe { debug(region_ptr) };
    }
}

/// Takes a pointer to a Region and reads the data into a String.
/// This is for trusted string sources only.
unsafe fn consume_string_region_written_by_vm(from: *mut Region) -> String {
    let data = consume_region(from);
    // We trust the VM/chain to return correct UTF-8, so let's save some gas
    String::from_utf8_unchecked(data)
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
        let request_ptr = &*req as *const Region as u32;

        let response_ptr = unsafe { query_chain(request_ptr) };
        let response = unsafe { consume_region(response_ptr as *mut Region) };

        from_slice(&response).unwrap_or_else(|parsing_err| {
            SystemResult::Err(SystemError::InvalidResponse {
                error: parsing_err.to_string(),
                response: response.into(),
            })
        })
    }
}
