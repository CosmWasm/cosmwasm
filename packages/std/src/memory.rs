use std::mem;
use std::os::raw::c_void;
use std::vec::Vec;

use crate::errors::{Error, NullPointer};

/// Refers to some heap allocated data in Wasm.
/// A pointer to an instance of this can be returned over FFI boundaries.
///
/// This struct is crate internal since the VM defined the same type independently.
#[repr(C)]
pub struct Region {
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
/// it consumes an existing one and returns a pointer to the Region
/// (preventing the memory from being freed until explicitly called later)
pub fn release_buffer(buffer: Vec<u8>) -> *mut c_void {
    let region = build_region(&buffer);
    mem::forget(buffer);
    Box::into_raw(region) as *mut c_void
}

/// Return the data referenced by the Region and
/// deallocates the Region (and the vector when finished).
/// Warning: only use this when you are sure the caller will never use (or free) the Region later
///
/// # Safety
///
/// If ptr is non-nil, it must refer to a valid Region, which was previously returned by alloc,
/// and not yet deallocated. This call will deallocate the Region and return an owner vector
/// to the caller containing the referenced data.
///
/// Naturally, calling this function twice on the same pointer will double deallocate data
/// and lead to a crash. Make sure to call it exactly once (either consuming the input in
/// the wasm code OR deallocating the buffer from the caller).
pub unsafe fn consume_region(ptr: *mut c_void) -> Result<Vec<u8>, Error> {
    if ptr.is_null() {
        return NullPointer {}.fail();
    }
    let region = Box::from_raw(ptr as *mut Region);
    let buffer = Vec::from_raw_parts(
        region.offset as *mut u8,
        region.len as usize,
        region.len as usize,
    );
    Ok(buffer)
}

/// Returns a box of a Region, which can be sent over a call to extern
/// note that this DOES NOT take ownership of the data, and we MUST NOT consume_region
/// the resulting data.
/// The Box must be dropped (with scope), but not the data
pub fn build_region(data: &[u8]) -> Box<Region> {
    Box::new(Region {
        offset: data.as_ptr() as u32,
        len: data.len() as u32,
    })
}
