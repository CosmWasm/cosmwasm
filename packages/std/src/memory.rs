use alloc::vec::Vec;
use core::{any::TypeId, marker::PhantomData, mem, ops::Deref, slice};

mod sealed {
    pub trait Sealed: 'static {}

    impl Sealed for super::Owned {}

    impl Sealed for super::Borrowed {}
}

pub trait Ownership: sealed::Sealed + 'static {}

impl<T> Ownership for T where T: sealed::Sealed {}

pub struct Owned;

pub struct Borrowed;

/// Describes some data allocated in Wasm's linear memory.
/// A pointer to an instance of this can be returned over FFI boundaries.
///
/// This struct is crate internal since the cosmwasm-vm defines the same type independently.
#[repr(C)]
pub struct Region<O: Ownership> {
    /// The beginning of the region expressed as bytes from the beginning of the linear memory
    pub offset: u32,
    /// The number of bytes available in this region
    pub capacity: u32,
    /// The number of bytes used in this region
    pub length: u32,

    _marker: PhantomData<O>,
}

const _: () = {
    assert!(mem::size_of::<Region<Borrowed>>() == 12);
    assert!(mem::size_of::<Region<Owned>>() == 12);
};

impl Region<Borrowed> {
    pub fn from_slice(slice: &[u8]) -> Self {
        unsafe { Self::from_parts(slice.as_ptr(), slice.len(), slice.len()) }
    }
}

impl Region<Owned> {
    /// Construct a region from an existing vector
    pub fn from_vec(vec: Vec<u8>) -> Self {
        let region = unsafe { Self::from_parts(vec.as_ptr(), vec.capacity(), vec.len()) };
        mem::forget(vec);
        region
    }

    /// Reconstruct a region from a raw pointer pointing to a `Box<Region>`.
    /// You'll want to use this when you received a region from the VM and want to dereference its contents.
    ///
    /// # Safety
    ///
    /// - The pointer must not be null
    /// - The pointer must be heap allocated
    /// - This region must point to a valid memory region
    /// - The memory region this region points to must be heap allocated as well
    pub unsafe fn from_heap_ptr(ptr: *mut Self) -> Box<Self> {
        assert!(!ptr.is_null(), "Region pointer is null");
        Box::from_raw(ptr)
    }

    /// Construct a new empty region with *at least* a capacity of what you passed in and a length of 0
    pub fn with_capacity(cap: usize) -> Self {
        let data = Vec::with_capacity(cap);
        let region = Self::from_vec(data);
        region
    }

    pub fn into_vec(self) -> Vec<u8> {
        let vector = unsafe {
            Vec::from_raw_parts(
                self.offset as *mut u8,
                self.length as usize,
                self.capacity as usize,
            )
        };
        mem::forget(self);
        vector
    }
}

impl<O> Region<O>
where
    O: Ownership,
{
    unsafe fn from_parts(ptr: *const u8, capacity: usize, length: usize) -> Self {
        // Well, this technically violates pointer provenance rules.
        // But there isn't a stable API for it, so that's the best we can do, I guess.
        Region {
            offset: u32::try_from(ptr as usize).expect("pointer doesn't fit in u32"),
            capacity: u32::try_from(capacity).expect("capacity doesn't fit in u32"),
            length: u32::try_from(length).expect("length doesn't fit in u32"),

            _marker: PhantomData,
        }
    }

    /// Access the memory region this region points to in form of a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.offset as *const u8, self.length as usize) }
    }

    /// Obtain the pointer to the region
    ///
    /// This is nothing but `&self as *const Region<T>` but makes sure the correct generic parameter is used
    pub fn as_ptr(&self) -> *const Self {
        self
    }

    /// Transform the region into an unmanaged mutable pointer
    ///
    /// This means we move this regions onto the heap (note, only the *structure* of the region, not the *contents of the pointer* we manage internally).
    /// To then deallocate this structure, you'll have to reconstruct the region via [`Region::from_heap_ptr`] and drop it.
    pub fn to_heap_ptr(self) -> *mut Self {
        let boxed = Box::new(self);
        Box::into_raw(boxed)
    }
}

impl<O> Deref for Region<O>
where
    O: Ownership,
{
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

impl<O> Drop for Region<O>
where
    O: Ownership,
{
    fn drop(&mut self) {
        // Since we can't specialize the drop impl we need to perform a runtime check
        if TypeId::of::<O>() == TypeId::of::<Owned>() {
            let region_start = self.offset as *mut u8;

            // This case is explicitely disallowed by Vec
            // "The pointer will never be null, so this type is null-pointer-optimized."
            assert!(!region_start.is_null(), "Region starts at null pointer");

            unsafe {
                let data =
                    Vec::from_raw_parts(region_start, self.length as usize, self.capacity as usize);

                drop(data);
            }
        }
    }
}

/// Returns the address of the optional Region as an offset in linear memory,
/// or zero if not present
#[cfg(feature = "iterator")]
pub fn get_optional_region_address<O: Ownership>(region: &Option<&Region<O>>) -> u32 {
    /// Returns the address of the Region as an offset in linear memory
    fn get_region_address<O: Ownership>(region: &Region<O>) -> u32 {
        region.as_ptr() as u32
    }

    region.map(get_region_address).unwrap_or(0)
}
