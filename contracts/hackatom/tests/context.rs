use std::mem;
use std::ffi::c_void;

use wasmer_runtime::{Ctx};

use cosmwasm::mock::{MockStorage};

/*** context data ****/

pub fn create_unmanaged_storage() ->*mut c_void {
    let state = Box::new(MockStorage::new());
    Box::into_raw(state) as *mut c_void
}

pub fn destroy_unmanaged_storage(ptr: *mut c_void) {
    let b = unsafe { Box::from_raw(ptr as *mut MockStorage) };
    mem::drop(b);
}

pub fn with_storage_from_context<F: FnMut(&mut MockStorage)>(ctx: &mut Ctx, mut func: F) {
    let mut b = unsafe { Box::from_raw(ctx.data as *mut MockStorage) };
    func(b.as_mut());
    mem::forget(b); // we do this to avoid cleanup
}
