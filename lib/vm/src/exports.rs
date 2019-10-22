use std::ffi::c_void;
use std::mem;

use wasmer_runtime::Ctx;

use cosmwasm::storage::Storage;

use crate::memory::{read_memory, write_memory};

/*** mocks to stub out actually db writes as extern "C" ***/

pub fn do_read<T: Storage>(ctx: &mut Ctx, key_ptr: u32, val_ptr: u32) -> i32 {
    let key = read_memory(ctx, key_ptr);
    let mut value: Option<Vec<u8>> = None;
    with_storage_from_context::<T>(ctx, |store| value = store.get(&key));
    match value {
        Some(buf) => write_memory(ctx, val_ptr, &buf),
        None => 0,
    }
}

pub fn do_write<T: Storage>(ctx: &mut Ctx, key: u32, value: u32) {
    let key = read_memory(ctx, key);
    let value = read_memory(ctx, value);
    with_storage_from_context::<T>(ctx, |store| store.set(&key, &value));
}

/*** context data ****/

pub fn setup_context<T: Storage>(storage: T) -> (*mut c_void, fn(*mut c_void)) {
    (create_unmanaged_storage(storage), destroy_unmanaged_storage::<T>)
}

fn create_unmanaged_storage<T: Storage>(storage: T) -> *mut c_void {
    let state = Box::new(storage);
    Box::into_raw(state) as *mut c_void
}

fn destroy_unmanaged_storage<T: Storage>(ptr: *mut c_void) {
    // auto-dropped with scope
    let _ = unsafe { Box::from_raw(ptr as *mut T) };
}

pub fn with_storage_from_context<T: Storage, F: FnMut(&mut T)>(ctx: &Ctx, mut func: F) {
    let mut b = unsafe { Box::from_raw(ctx.data as *mut T) };
    func(b.as_mut());
    mem::forget(b); // we do this to avoid cleanup
}
