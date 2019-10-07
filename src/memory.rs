use std::mem;
use std::os::raw::c_void;
use std::vec::Vec;

use failure::{bail, Error};

/// Slice refers to some heap allocated data in wasm.
/// A pointer to this can be returned over ffi boundaries.
#[repr(C)]
pub struct Slice {
    pub offset: u32,
    pub len: u32,
}

/// alloc is the same as external allocate, but designed to be called internally
pub fn alloc(size: usize) -> *mut c_void {
    // allocate the space in memory
    let buffer = vec![0u8; size];
    release_buffer(buffer)
}

/// release_buffer is like alloc, but instead of creating a new vector
/// it consumes an existing one and returns a pointer to the slice
/// (preventing the memory from being freed until explicitly called later)
pub fn release_buffer(buffer: Vec<u8>) -> *mut c_void {
    let slice = build_slice(&buffer);
    mem::forget(buffer);
    Box::into_raw(slice) as *mut c_void
}

/// consume_slice will return the data referenced by the slice and
/// deallocates the slice (and the vector when finished).
/// Warning: only use this when you are sure the caller will never use (or free) the slice later
pub unsafe fn consume_slice(ptr: *mut c_void) -> Result<Vec<u8>, Error> {
    if ptr.is_null() {
        bail!("cannot consume null pointer");
    }
    let slice = Box::from_raw(ptr as *mut Slice);
    let buffer = Vec::from_raw_parts(
        slice.offset as *mut u8,
        slice.len as usize,
        slice.len as usize,
    );
    Ok(buffer)
}

/// build_slice returns a box of a slice, which can be sent over a call to extern
/// note that this DOES NOT take ownership of the data, and we MUST NOT consume_slice
/// the resulting data.
/// The Box must be dropped (with scope), but not the data
pub fn build_slice(data: &[u8]) -> Box<Slice> {
    Box::new(Slice {
        offset: data.as_ptr() as u32,
        len: data.len() as u32,
    })
}
