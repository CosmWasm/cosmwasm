//! Internal details to be used by instance.rs only
use std::ffi::c_void;
use std::mem;

use wasmer_runtime_core::vm::Ctx;

#[cfg(feature = "iterator")]
use cosmwasm_std::KV;
use cosmwasm_std::{
    Api, ApiQuerierResponse, ApiSystemError, Binary, CanonicalAddr, HumanAddr, Querier,
    QuerierResponse, QueryRequest, Storage,
};

#[cfg(feature = "iterator")]
pub use iter_support::{
    do_next, do_scan, ERROR_NEXT_INVALID_ITERATOR, ERROR_NO_STORAGE, ERROR_SCAN_INVALID_ORDER,
};

use crate::errors::Error;
use crate::memory::{read_region, write_region};
use crate::serde::{from_slice, to_vec};

static MAX_LENGTH_DB_KEY: usize = 100_000;
static MAX_LENGTH_DB_VALUE: usize = 100_000;
static MAX_LENGTH_ADDRESS: usize = 200;
static MAX_LENGTH_QUERY: usize = 100_000;

static SUCCESS: i32 = 0;
/// An unknown error occurred when writing to region
static ERROR_REGION_WRITE_UNKNOWN: i32 = -1_000_001;
/// Could not write to region because it is too small
static ERROR_REGION_WRITE_TOO_SMALL: i32 = -1_000_002;
/// An unknown error occurred when reading region
static ERROR_REGION_READ_UNKNOWN: i32 = -1_000_101;
/// The contract sent us a Region we're not willing to read because it is too big
static ERROR_REGION_READ_LENGTH_TOO_BIG: i32 = -1_000_102;
/// An unknonw error when canonicalizing address
static ERROR_CANONICALIZE_UNKNOWN: i32 = -1_000_201;
/// The input address (human address) was invalid
static ERROR_CANONICALIZE_INVALID_INPUT: i32 = -1_000_202;
/// An unknonw error when humanizing address
static ERROR_HUMANIZE_UNKNOWN: i32 = -1_000_301;
/// An unknonw error when querying the chain
// static ERROR_QUERY_CHAIN_UNKNOWN: i32 = -1_000_401;
/// Cannot serialize query response
static ERROR_QUERY_CHAIN_CANNOT_SERIALIZE_RESPONSE: i32 = -1_000_402;

/// Reads a storage entry from the VM's storage into Wasm memory
pub fn do_read<S: Storage, Q: Querier>(ctx: &Ctx, key_ptr: u32, value_ptr: u32) -> i32 {
    let key = match read_region(ctx, key_ptr, MAX_LENGTH_DB_KEY) {
        Ok(data) => data,
        Err(Error::RegionLengthTooBigErr { .. }) => return ERROR_REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return ERROR_REGION_READ_UNKNOWN,
    };
    let mut value: Option<Vec<u8>> = None;
    with_storage_from_context::<S, Q, _>(ctx, |store| value = store.get(&key));
    match value {
        Some(buf) => match write_region(ctx, value_ptr, &buf) {
            Ok(()) => SUCCESS,
            Err(Error::RegionTooSmallErr { .. }) => ERROR_REGION_WRITE_TOO_SMALL,
            Err(_) => ERROR_REGION_WRITE_UNKNOWN,
        },
        None => SUCCESS,
    }
}

/// Writes a storage entry from Wasm memory into the VM's storage
pub fn do_write<S: Storage, Q: Querier>(ctx: &Ctx, key_ptr: u32, value_ptr: u32) -> i32 {
    let key = match read_region(ctx, key_ptr, MAX_LENGTH_DB_KEY) {
        Ok(data) => data,
        Err(Error::RegionLengthTooBigErr { .. }) => return ERROR_REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return ERROR_REGION_READ_UNKNOWN,
    };
    let value = match read_region(ctx, value_ptr, MAX_LENGTH_DB_VALUE) {
        Ok(data) => data,
        Err(Error::RegionLengthTooBigErr { .. }) => return ERROR_REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return ERROR_REGION_READ_UNKNOWN,
    };
    with_storage_from_context::<S, Q, _>(ctx, |store| store.set(&key, &value));
    SUCCESS
}

pub fn do_remove<S: Storage, Q: Querier>(ctx: &Ctx, key_ptr: u32) -> i32 {
    let key = match read_region(ctx, key_ptr, MAX_LENGTH_DB_KEY) {
        Ok(data) => data,
        Err(Error::RegionLengthTooBigErr { .. }) => return ERROR_REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return ERROR_REGION_READ_UNKNOWN,
    };
    with_storage_from_context::<S, Q, _>(ctx, |store| store.remove(&key));
    SUCCESS
}

pub fn do_canonicalize_address<A: Api>(
    api: A,
    ctx: &mut Ctx,
    human_ptr: u32,
    canonical_ptr: u32,
) -> i32 {
    let human_data = match read_region(ctx, human_ptr, MAX_LENGTH_ADDRESS) {
        Ok(data) => data,
        Err(Error::RegionLengthTooBigErr { .. }) => return ERROR_REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return ERROR_REGION_READ_UNKNOWN,
    };
    let human = match String::from_utf8(human_data) {
        Ok(human_str) => HumanAddr(human_str),
        Err(_) => return ERROR_CANONICALIZE_INVALID_INPUT,
    };
    match api.canonical_address(&human) {
        Ok(canon) => match write_region(ctx, canonical_ptr, canon.as_slice()) {
            Ok(()) => SUCCESS,
            Err(Error::RegionTooSmallErr { .. }) => ERROR_REGION_WRITE_TOO_SMALL,
            Err(_) => ERROR_REGION_WRITE_UNKNOWN,
        },
        Err(_) => ERROR_CANONICALIZE_UNKNOWN,
    }
}

pub fn do_humanize_address<A: Api>(
    api: A,
    ctx: &mut Ctx,
    canonical_ptr: u32,
    human_ptr: u32,
) -> i32 {
    let canonical = match read_region(ctx, canonical_ptr, MAX_LENGTH_ADDRESS) {
        Ok(data) => Binary(data),
        Err(Error::RegionLengthTooBigErr { .. }) => return ERROR_REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return ERROR_REGION_READ_UNKNOWN,
    };
    match api.human_address(&CanonicalAddr(canonical)) {
        Ok(human) => match write_region(ctx, human_ptr, human.as_str().as_bytes()) {
            Ok(()) => SUCCESS,
            Err(Error::RegionTooSmallErr { .. }) => ERROR_REGION_WRITE_TOO_SMALL,
            Err(_) => ERROR_REGION_WRITE_UNKNOWN,
        },
        Err(_) => ERROR_HUMANIZE_UNKNOWN,
    }
}

pub fn do_query_chain<A: Api, S: Storage, Q: Querier>(
    _api: A,
    ctx: &mut Ctx,
    request_ptr: u32,
    response_ptr: u32,
) -> i32 {
    let request = match read_region(ctx, request_ptr, MAX_LENGTH_QUERY) {
        Ok(data) => data,
        Err(Error::RegionLengthTooBigErr { .. }) => return ERROR_REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return ERROR_REGION_READ_UNKNOWN,
    };

    // default result, then try real querier callback
    let mut res: QuerierResponse = Err(ApiSystemError::InvalidRequest {
        error: "no querier registered".to_string(),
    });
    match from_slice::<QueryRequest>(&request) {
        // if we parse, try to execute the query
        Ok(parsed) => {
            with_querier_from_context::<S, Q, _>(ctx, |querier| res = querier.query(&parsed))
        }
        // otherwise, return the InvalidRequest error as ApiSystemError
        Err(err) => {
            res = Err(ApiSystemError::InvalidRequest {
                error: err.to_string(),
            })
        }
    };

    let api_res: ApiQuerierResponse = res.into();

    match to_vec(&api_res) {
        Ok(serialized) => match write_region(ctx, response_ptr, &serialized) {
            Ok(()) => SUCCESS,
            Err(Error::RegionTooSmallErr { .. }) => ERROR_REGION_WRITE_TOO_SMALL,
            Err(_) => ERROR_REGION_WRITE_UNKNOWN,
        },
        Err(_) => ERROR_QUERY_CHAIN_CANNOT_SERIALIZE_RESPONSE,
    }
}

#[cfg(feature = "iterator")]
mod iter_support {
    use super::*;
    use crate::memory::maybe_read_region;
    use cosmwasm_std::{Order, KV};
    use std::convert::TryInto;

    /// Invalid Order enum value passed into scan
    pub static ERROR_SCAN_INVALID_ORDER: i32 = -2_000_001;
    // Iterator pointer not registered
    pub static ERROR_NEXT_INVALID_ITERATOR: i32 = -2_000_002;
    /// Generic error - using context with no Storage attached
    pub static ERROR_NO_STORAGE: i32 = -3_000_001;

    pub fn do_scan<S: Storage + 'static, Q: Querier>(
        ctx: &Ctx,
        start_ptr: u32,
        end_ptr: u32,
        order: i32,
    ) -> i32 {
        let start = match maybe_read_region(ctx, start_ptr, MAX_LENGTH_DB_KEY) {
            Ok(data) => data,
            Err(Error::RegionLengthTooBigErr { .. }) => return ERROR_REGION_READ_LENGTH_TOO_BIG,
            Err(_) => return ERROR_REGION_READ_UNKNOWN,
        };
        let end = match maybe_read_region(ctx, end_ptr, MAX_LENGTH_DB_KEY) {
            Ok(data) => data,
            Err(Error::RegionLengthTooBigErr { .. }) => return ERROR_REGION_READ_LENGTH_TOO_BIG,
            Err(_) => return ERROR_REGION_READ_UNKNOWN,
        };
        let order: Order = match order.try_into() {
            Ok(o) => o,
            Err(_) => return ERROR_SCAN_INVALID_ORDER,
        };
        let mut res = ERROR_NO_STORAGE;
        with_storage_from_context::<S, Q, _>(ctx, |store| {
            let iter = store.range(start.as_deref(), end.as_deref(), order);
            // Unsafe: I know the iterator will be deallocated before the storage as I control the lifetime below
            // But there is no way for the compiler to know. So... let's just lie to the compiler a little bit.
            let live_forever: Box<dyn Iterator<Item = KV> + 'static> =
                unsafe { mem::transmute(iter) };
            leave_iterator::<S, Q>(ctx, live_forever);
            res = SUCCESS;
        });
        res
    }

    pub fn do_next<S: Storage, Q: Querier>(ctx: &Ctx, key_ptr: u32, value_ptr: u32) -> i32 {
        let mut iter = match take_iterator::<S, Q>(ctx) {
            Some(i) => i,
            None => return ERROR_NEXT_INVALID_ITERATOR,
        };
        // get next item and return iterator
        let item = iter.next();
        leave_iterator::<S, Q>(ctx, iter);

        // prepare return values
        let (key, value) = match item {
            Some(item) => item,
            None => return SUCCESS, // Return early without writing key. Empty key will later be treated as _no more element_.
        };

        match write_region(ctx, key_ptr, &key) {
            Ok(()) => (),
            Err(Error::RegionTooSmallErr { .. }) => return ERROR_REGION_WRITE_TOO_SMALL,
            Err(_) => return ERROR_REGION_WRITE_UNKNOWN,
        };
        match write_region(ctx, value_ptr, &value) {
            Ok(()) => (),
            Err(Error::RegionTooSmallErr { .. }) => return ERROR_REGION_WRITE_TOO_SMALL,
            Err(_) => return ERROR_REGION_WRITE_UNKNOWN,
        };
        SUCCESS
    }

    // set the iterator, overwriting any possible iterator previously set
    fn leave_iterator<S: Storage, Q: Querier>(ctx: &Ctx, iter: Box<dyn Iterator<Item = KV>>) {
        let b = unsafe { get_data::<S, Q>(ctx.data) };
        let mut b = mem::ManuallyDrop::new(b); // we do this to avoid cleanup
        b.iter = Some(iter);
    }

    fn take_iterator<S: Storage, Q: Querier>(ctx: &Ctx) -> Option<Box<dyn Iterator<Item = KV>>> {
        let b = unsafe { get_data::<S, Q>(ctx.data) };
        let mut b = mem::ManuallyDrop::new(b); // we do this to avoid cleanup
        b.iter.take()
    }
}

/** context data **/

struct ContextData<S: Storage, Q: Querier> {
    storage: Option<S>,
    querier: Option<Q>,
    #[cfg(feature = "iterator")]
    iter: Option<Box<dyn Iterator<Item = KV>>>,
}

pub fn setup_context<S: Storage, Q: Querier>() -> (*mut c_void, fn(*mut c_void)) {
    (
        create_unmanaged_storage::<S, Q>(),
        destroy_unmanaged_storage::<S, Q>,
    )
}

fn create_unmanaged_storage<S: Storage, Q: Querier>() -> *mut c_void {
    let data = ContextData::<S, Q> {
        storage: None,
        querier: None,
        #[cfg(feature = "iterator")]
        iter: None,
    };
    let state = Box::new(data);
    Box::into_raw(state) as *mut c_void
}

fn destroy_unmanaged_storage<S: Storage, Q: Querier>(ptr: *mut c_void) {
    if !ptr.is_null() {
        let mut dead = unsafe { get_data::<S, Q>(ptr) };
        // ensure the iterator (if any) is dropped before the storage
        free_iterator(&mut dead);
    }
}

unsafe fn get_data<S: Storage, Q: Querier>(ptr: *mut c_void) -> Box<ContextData<S, Q>> {
    Box::from_raw(ptr as *mut ContextData<S, Q>)
}

#[cfg(feature = "iterator")]
fn free_iterator<S: Storage, Q: Querier>(context: &mut Box<ContextData<S, Q>>) {
    let _ = context.iter.take();
}

#[cfg(not(feature = "iterator"))]
fn free_iterator<S: Storage, Q: Querier>(_context: &mut Box<ContextData<S, Q>>) {}

pub fn with_storage_from_context<S: Storage, Q: Querier, F: FnMut(&mut S)>(ctx: &Ctx, mut func: F) {
    let b = unsafe { get_data::<S, Q>(ctx.data) };
    let mut b = mem::ManuallyDrop::new(b);
    let mut storage = b.storage.take();
    if let Some(data) = &mut storage {
        func(data);
    }
    b.storage = storage;
}

pub fn with_querier_from_context<S: Storage, Q: Querier, F: FnMut(&Q)>(ctx: &Ctx, mut func: F) {
    let b = unsafe { get_data::<S, Q>(ctx.data) };
    // we do this to avoid cleanup
    let mut b = mem::ManuallyDrop::new(b);
    let querier = b.querier.take();
    if let Some(q) = &querier {
        func(q);
    }
    b.querier = querier;
}

/// take_context_data will return the original storage and querier, and closes any remaining
/// iterators. This is meant to be called when recycling the instance
pub(crate) fn take_context_data<S: Storage, Q: Querier>(ctx: &Ctx) -> (Option<S>, Option<Q>) {
    let b = unsafe { get_data::<S, Q>(ctx.data) };
    let mut b = mem::ManuallyDrop::new(b);
    // free out the iterator as this finalizes the instance
    free_iterator(&mut b);
    (b.storage.take(), b.querier.take())
}

/// leave_context_data sets the original storage and querier. These must both be set.
/// Should be followed by exactly one call to take_context_data when the instance is finished.
pub(crate) fn leave_context_data<S: Storage, Q: Querier>(ctx: &Ctx, storage: S, querier: Q) {
    let b = unsafe { get_data::<S, Q>(ctx.data) };
    let mut b = mem::ManuallyDrop::new(b); // we do this to avoid cleanup
    b.storage = Some(storage);
    b.querier = Some(querier);
}
