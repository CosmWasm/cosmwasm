use std::convert::TryFrom;
use std::mem;
use std::os::raw::c_void;
use std::vec::Vec;

/// Refers to some heap allocated data in Wasm.
/// A pointer to an instance of this can be returned over FFI boundaries.
///
/// This struct is crate internal since the VM defined the same type independently.
#[repr(C)]
pub struct Region {
    pub offset: u32,
    /// The number of bytes available in this region
    pub capacity: u32,
    /// The number of bytes used in this region
    pub length: u32,
}

/// Creates a memory region of capacity `size` and length 0. Returns a pointer to the Region.
/// This is the same as the `allocate` export, but designed to be called internally.
pub fn alloc(size: usize) -> *mut c_void {
    let data: Vec<u8> = Vec::with_capacity(size);
    let data_ptr = data.as_ptr() as usize;

    let region = build_region_from_components(
        u32::try_from(data_ptr).expect("pointer doesn't fit in u32"),
        u32::try_from(data.capacity()).expect("capacity doesn't fit in u32"),
        0,
    );
    mem::forget(data);
    Box::into_raw(region) as *mut c_void
}

/// Similar to alloc, but instead of creating a new vector it consumes an existing one and returns
/// a pointer to the Region (preventing the memory from being freed until explicitly called later).
///
/// The resulting Region has capacity = length, i.e. the buffer's capacity is ignored.
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
/// The ptr must refer to a valid Region, which was previously returned by alloc,
/// and not yet deallocated. This call will deallocate the Region and return an owner vector
/// to the caller containing the referenced data.
///
/// Naturally, calling this function twice on the same pointer will double deallocate data
/// and lead to a crash. Make sure to call it exactly once (either consuming the input in
/// the wasm code OR deallocating the buffer from the caller).
pub unsafe fn consume_region(ptr: *mut c_void) -> Vec<u8> {
    if ptr.is_null() {
        panic!("Region pointer is null");
    }
    let region = Box::from_raw(ptr as *mut Region);
    let region_start = region.offset as *mut u8;
    if region_start.is_null() {
        // This case is explicitely disallowed by Vec
        // "The pointer will never be null, so this type is null-pointer-optimized."
        panic!("Region starts at null pointer");
    }
    Vec::from_raw_parts(
        region_start,
        region.length as usize,
        region.capacity as usize,
    )
}

/// Returns a box of a Region, which can be sent over a call to extern
/// note that this DOES NOT take ownership of the data, and we MUST NOT consume_region
/// the resulting data.
/// The Box must be dropped (with scope), but not the data
pub fn build_region(data: &[u8]) -> Box<Region> {
    let data_ptr = data.as_ptr() as usize;
    build_region_from_components(
        u32::try_from(data_ptr).expect("pointer doesn't fit in u32"),
        u32::try_from(data.len()).expect("length doesn't fit in u32"),
        u32::try_from(data.len()).expect("length doesn't fit in u32"),
    )
}

fn build_region_from_components(offset: u32, capacity: u32, length: u32) -> Box<Region> {
    Box::new(Region {
        offset: offset,
        capacity: capacity,
        length: length,
    })
}
