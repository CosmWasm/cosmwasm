use alloc::vec::Vec;
use core::{any::TypeId, marker::PhantomData, mem, ops::Deref, slice};

/// Element that can be used to construct a new `Region`
///
/// # Safety
///
/// The following invariant must be upheld:
///
/// - full allocated capacity == value returned by capacity
///
/// This is important to uphold the safety invariant of the `dealloc` method, which requires us to pass the same Layout
/// into it as was used to allocate a memory region.
/// And since `size` is one of the parameters, it is important to pass in the exact same capacity.
///
/// See: <https://doc.rust-lang.org/stable/alloc/alloc/trait.GlobalAlloc.html#safety-2>
pub unsafe trait RegionSource {
    type Ownership: Ownership;

    fn ptr(&self) -> *const u8;
    fn len(&self) -> usize;
    fn capacity(&self) -> usize;
}

unsafe impl RegionSource for &[u8] {
    type Ownership = Borrowed;

    fn ptr(&self) -> *const u8 {
        self.as_ptr()
    }

    fn len(&self) -> usize {
        (*self).len()
    }

    fn capacity(&self) -> usize {
        self.len()
    }
}

unsafe impl RegionSource for Vec<u8> {
    type Ownership = Owned;

    fn ptr(&self) -> *const u8 {
        self.as_ptr()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn capacity(&self) -> usize {
        self.capacity()
    }
}

mod sealed {
    pub trait Sealed: 'static {}

    impl Sealed for super::Owned {}

    impl Sealed for super::Borrowed {}
}

pub trait Ownership: sealed::Sealed + 'static {}

impl<T> Ownership for T where T: sealed::Sealed {}

pub struct Owned {}

pub struct Borrowed {}

/// Describes some data allocated in Wasm's linear memory.
/// A pointer to an instance of this can be returned over FFI boundaries.
///
/// This struct is crate internal since the cosmwasm-vm defines the same type independently.
#[repr(C)]
pub struct Region<T: Ownership> {
    /// The beginning of the region expressed as bytes from the beginning of the linear memory
    pub offset: u32,
    /// The number of bytes available in this region
    pub capacity: u32,
    /// The number of bytes used in this region
    pub length: u32,

    _marker: PhantomData<T>,
}

impl Region<Owned> {
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
        let region = Self::from_data(data);
        region
    }

    pub fn into_inner(self) -> Vec<u8> {
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
    /// Construct a new region from any kind of data that can be turned into a region
    pub fn from_data<S>(data: S) -> Self
    where
        S: RegionSource<Ownership = O>,
    {
        // Well, this technically violates pointer provenance rules.
        // But there isn't a stable API for it, so that's the best we can do, I guess.
        let region = Region {
            offset: u32::try_from(data.ptr() as usize).expect("pointer doesn't fit in u32"),
            capacity: u32::try_from(data.capacity()).expect("capacity doesn't fit in u32"),
            length: u32::try_from(data.len()).expect("length doesn't fit in u32"),

            _marker: PhantomData,
        };

        // We gonna forget this.. as a safety measure..
        // If we didn't do this and the `RegionSource` was a `Vec` we would deallocate it and that's BAD
        mem::forget(data);

        region
    }
}

impl<T> Region<T>
where
    T: Ownership,
{
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

impl<T> Deref for Region<T>
where
    T: Ownership,
{
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

impl<T> Drop for Region<T>
where
    T: Ownership,
{
    fn drop(&mut self) {
        // Since we can't specialize the drop impl we need to perform a runtime check
        if TypeId::of::<T>() == TypeId::of::<Owned>() {
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
