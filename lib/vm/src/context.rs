/**
Internal details to be used by instance.rs only
**/
use std::ffi::c_void;
use std::mem;

use wasmer_runtime::Ctx;

use cosmwasm::storage::Storage;

use crate::memory::{read_memory, write_memory};

pub fn do_read<T: Storage>(ctx: &mut Ctx, key_ptr: u32, val_ptr: u32) -> i32 {
    let key = read_memory(ctx, key_ptr);
    let mut value: Option<Vec<u8>> = None;
    with_storage_from_context(ctx, |store: &mut T| value = store.get(&key));
    match value {
        Some(buf) => write_memory(ctx, val_ptr, &buf),
        None => 0,
    }
}

pub fn do_write<T: Storage>(ctx: &mut Ctx, key: u32, value: u32) {
    let key = read_memory(ctx, key);
    let value = read_memory(ctx, value);
    with_storage_from_context(ctx, |store: &mut T| store.set(&key, &value));
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
    let mut b = unsafe { get_data::<T>(ctx.data) };
    if let Some(store) = &mut b.data {
        func(store);
    }
    mem::forget(b); // we do this to avoid cleanup
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
