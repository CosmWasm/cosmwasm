use std::mem;
use std::ffi::c_void;

use wasmer_runtime::{Ctx};

use cosmwasm::imports::{Storage};
use cosmwasm::mock::{MockStorage};

use crate::memory::{read_memory, write_memory};

/*** mocks to stub out actually db writes as extern "C" ***/

pub fn do_read(ctx: &mut Ctx, key_ptr: i32, val_ptr: i32) -> i32 {
    let key = read_memory(ctx, key_ptr);
    let mut value: Option<Vec<u8>> = None;
    with_storage_from_context(ctx, |store| value = store.get(&key));
    match value {
        Some(buf) => write_memory(ctx, val_ptr, &buf),
        None => 0,
    }
}

pub fn do_write(ctx: &mut Ctx, key: i32, value: i32) {
    let key = read_memory(ctx, key);
    let value = read_memory(ctx, value);
    with_storage_from_context(ctx, |store| store.set(&key, &value));
}


/*** context data ****/

pub fn setup_context() -> (*mut c_void, fn(*mut c_void)) {
    (create_unmanaged_storage(), destroy_unmanaged_storage)
}

fn create_unmanaged_storage() ->*mut c_void {
    let state = Box::new(MockStorage::new());
    Box::into_raw(state) as *mut c_void
}

fn destroy_unmanaged_storage(ptr: *mut c_void) {
    let b = unsafe { Box::from_raw(ptr as *mut MockStorage) };
    mem::drop(b);
}

fn with_storage_from_context<F: FnMut(&mut MockStorage)>(ctx: &mut Ctx, mut func: F) {
    let mut b = unsafe { Box::from_raw(ctx.data as *mut MockStorage) };
    func(b.as_mut());
    mem::forget(b); // we do this to avoid cleanup
}



