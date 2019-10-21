use wasmer_runtime::{Ctx, Func, Instance};
use wasmer_runtime_core::memory::ptr::{Array, WasmPtr};
use wasmer_runtime_core::types::ValueType;

/****** read/write to wasm memory buffer ****/

/// Slice refers to some heap allocated data in wasm.
/// A pointer to this can be returned over ffi boundaries.
///
/// This is the same as cosmwasm::memory::Slice
/// but defined here to allow wasm impl
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct Slice {
    pub offset: u32,
    pub len: u32,
}

unsafe impl ValueType for Slice {}

// write_mem allocates memory in the instance and copies the given data in
// returns the memory offset, to be passed as an argument
// panics on any error (TODO, use result?)
pub fn allocate(instance: &mut Instance, data: &[u8]) -> u32 {
    // allocate
    let alloc: Func<(u32), (u32)> = instance.func("allocate").unwrap();
    let ptr = alloc.call(data.len() as u32).unwrap();
    write_memory(instance.context(), ptr, data);
    ptr
}

pub fn read_memory(ctx: &Ctx, ptr: u32) -> Vec<u8> {
    let slice = to_slice(ctx, ptr);
    let memory = ctx.memory(0);
    let len = slice.len as usize;
    let mut result = vec![0u8; len];

    // TODO: there must be a faster way to copy memory
    let buffer = WasmPtr::<u8, Array>::new(slice.offset)
        .deref(memory, 0, slice.len)
        .unwrap();
    for i in 0..len {
        // result[i] = unsafe { buffer.get_unchecked(i).get() }
        // resolved to memcpy, but only if we really start copying huge arrays
        result[i] = buffer[i].get();
    }
    result
}

// write_memory returns how many bytes written on success
// negative result is how many bytes requested if too small
pub fn write_memory(ctx: &Ctx, ptr: u32, data: &[u8]) -> i32 {
    let slice = to_slice(ctx, ptr);
    if data.len() > (slice.len as usize) {
        return -(data.len() as i32);
    }
    if data.is_empty() {
        return 0;
    }

    let memory = ctx.memory(0);
    // TODO: there must be a faster way to copy memory
    let buffer = unsafe {
        WasmPtr::<u8, Array>::new(slice.offset)
            .deref_mut(memory, 0, slice.len)
            .unwrap()
    };
    for i in 0..data.len() {
        buffer[i].set(data[i])
    }
    data.len() as i32
}

// to_slice reads in a ptr to slice in wasm memory and constructs the object we can use to access it
fn to_slice(ctx: &Ctx, ptr: u32) -> Slice {
    let memory = &ctx.memory(0);
    let wptr = WasmPtr::<Slice>::new(ptr);
    wptr.deref(memory).map(|x| x.get()).unwrap_or_default()
}
