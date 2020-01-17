use wasmer_runtime_core::{
    memory::ptr::{Array, WasmPtr},
    types::ValueType,
    vm::Ctx,
};

/****** read/write to wasm memory buffer ****/

/// Slice refers to some heap allocated data in wasm.
/// A pointer to this can be returned over ffi boundaries.
///
/// This is the same as cosmwasm::memory::Slice
/// but defined here to allow wasm impl
#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct Slice {
    pub offset: u32,
    pub len: u32,
}

unsafe impl ValueType for Slice {}

// Expects a (fixed size) Slice struct at ptr, which is read. This links to the
// memory region, which is read in the second step.
pub fn read_memory(ctx: &Ctx, ptr: u32) -> Vec<u8> {
    let slice = to_slice(ctx, ptr);
    let memory = ctx.memory(0);

    // TODO: there must be a faster way to copy memory
    match WasmPtr::<u8, Array>::new(slice.offset).deref(memory, 0, slice.len) {
        Some(cells) => {
            let len = slice.len as usize;
            let mut result = vec![0u8; len];
            for i in 0..len {
                // result[i] = unsafe { cells.get_unchecked(i).get() }
                // resolved to memcpy, but only if we really start copying huge arrays
                result[i] = cells[i].get();
            }
            result
        }
        None => panic!(
            "Error dereferencing slice {:?} in wasm memory of size {}. This typically happens when the given pointer does not point to a Slice struct.",
            slice,
            memory.size().bytes().0
        ),
    }
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
    let memory = ctx.memory(0);
    let wptr = WasmPtr::<Slice>::new(ptr);
    let cell = wptr.deref(memory).unwrap();
    cell.get()
}
