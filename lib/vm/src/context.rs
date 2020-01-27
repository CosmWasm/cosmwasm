/**
Internal details to be used by instance.rs only
**/
use std::convert::TryInto;
use std::ffi::c_void;
use std::mem;

use wasmer_runtime_core::vm::Ctx;

use cosmwasm::traits::{Api, Storage};

use crate::errors::Error;
use crate::memory::{read_region, write_region};
use cosmwasm::types::{CanonicalAddr, HumanAddr};

/// An unknown error occurred when writing to region
static ERROR_WRITE_TO_REGION_UNKNONW: i32 = -1000001;
/// Could not write to region because it is too small
static ERROR_WRITE_TO_REGION_TOO_SMALL: i32 = -1000002;

static ERROR_CANONICAL_ADDRESS_UNKNOWN: i32 = -1000101;
static ERROR_CANONICAL_ADDRESS_INVALID_UTF8: i32 = -1000102;

static ERROR_HUMAN_ADDRESS_UNKNOWN: i32 = -1000201;
static ERROR_HUMAN_ADDRESS_INVALID_UTF8: i32 = -1000202;

pub fn do_read<T: Storage>(ctx: &Ctx, key_ptr: u32, value_ptr: u32) -> i32 {
    let key = read_region(ctx, key_ptr);
    let mut value: Option<Vec<u8>> = None;
    with_storage_from_context(ctx, |store: &mut T| value = store.get(&key));
    match value {
        Some(buf) => match write_region(ctx, value_ptr, &buf) {
            Ok(bytes_written) => bytes_written.try_into().unwrap(),
            Err(Error::RegionTooSmallErr { .. }) => ERROR_WRITE_TO_REGION_TOO_SMALL,
            Err(_) => ERROR_WRITE_TO_REGION_UNKNONW,
        },
        None => 0,
    }
}

pub fn do_write<T: Storage>(ctx: &Ctx, key_ptr: u32, value_ptr: u32) {
    let key = read_region(ctx, key_ptr);
    let value = read_region(ctx, value_ptr);
    with_storage_from_context(ctx, |store: &mut T| store.set(&key, &value));
}

pub fn do_canonical_address<A: Api>(
    api: A,
    ctx: &mut Ctx,
    human_ptr: u32,
    canonical_ptr: u32,
) -> i32 {
    let human = read_region(ctx, human_ptr);
    let human = match String::from_utf8(human) {
        Ok(human_str) => HumanAddr(human_str),
        Err(_) => return ERROR_HUMAN_ADDRESS_INVALID_UTF8,
    };
    match api.canonical_address(&human) {
        Ok(canon) => match write_region(ctx, canonical_ptr, canon.as_bytes()) {
            Ok(bytes_written) => bytes_written.try_into().unwrap(),
            Err(Error::RegionTooSmallErr { .. }) => ERROR_WRITE_TO_REGION_TOO_SMALL,
            Err(_) => ERROR_WRITE_TO_REGION_UNKNONW,
        },
        Err(_) => ERROR_CANONICAL_ADDRESS_UNKNOWN,
    }
}

pub fn do_human_address<A: Api>(api: A, ctx: &mut Ctx, canonical_ptr: u32, human_ptr: u32) -> i32 {
    let canon = match CanonicalAddr::from_external_base64(read_region(ctx, canonical_ptr)) {
        Ok(v) => v,
        Err(_) => return ERROR_CANONICAL_ADDRESS_INVALID_UTF8,
    };
    match api.human_address(&canon) {
        Ok(human) => match write_region(ctx, human_ptr, human.as_str().as_bytes()) {
            Ok(bytes_written) => bytes_written.try_into().unwrap(),
            Err(Error::RegionTooSmallErr { .. }) => ERROR_WRITE_TO_REGION_TOO_SMALL,
            Err(_) => ERROR_WRITE_TO_REGION_UNKNONW,
        },
        Err(_) => ERROR_HUMAN_ADDRESS_UNKNOWN,
    }
}

/** context data **/

struct ContextData<T: Storage> {
    data: Option<T>,
}

pub fn setup_context<T: Storage>() -> (*mut c_void, fn(*mut c_void)) {
    (
        create_unmanaged_storage::<T>(),
        destroy_unmanaged_storage::<T>,
    )
}

fn create_unmanaged_storage<T: Storage>() -> *mut c_void {
    let data = ContextData::<T> { data: None };
    let state = Box::new(data);
    Box::into_raw(state) as *mut c_void
}

unsafe fn get_data<T: Storage>(ptr: *mut c_void) -> Box<ContextData<T>> {
    Box::from_raw(ptr as *mut ContextData<T>)
}

fn destroy_unmanaged_storage<T: Storage>(ptr: *mut c_void) {
    if !ptr.is_null() {
        // auto-dropped with scope
        let _ = unsafe { get_data::<T>(ptr) };
    }
}

pub fn with_storage_from_context<T: Storage, F: FnMut(&mut T)>(ctx: &Ctx, mut func: F) {
    let mut storage: Option<T> = take_storage(ctx);
    if let Some(data) = &mut storage {
        func(data);
    }
    leave_storage(ctx, storage);
}

pub fn take_storage<T: Storage>(ctx: &Ctx) -> Option<T> {
    let mut b = unsafe { get_data(ctx.data) };
    let res = b.data.take();
    mem::forget(b); // we do this to avoid cleanup
    res
}

pub fn leave_storage<T: Storage>(ctx: &Ctx, storage: Option<T>) {
    let mut b = unsafe { get_data(ctx.data) };
    // clean-up if needed
    let _ = b.data.take();
    b.data = storage;
    mem::forget(b); // we do this to avoid cleanup
}
