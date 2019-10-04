use wasmer_runtime::{Ctx, Func, Instance};

use cosmwasm::memory::Slice;

/****** read/write to wasm memory buffer ****/

// write_mem allocates memory in the instance and copies the given data in
// returns the memory offset, to be passed as an argument
// panics on any error (TODO, use result?)
pub fn allocate(instance: &mut Instance, data: &[u8]) -> i32 {
    // allocate
    let alloc: Func<(i32), (i32)> = instance.func("allocate").unwrap();
    let ptr = alloc.call(data.len() as i32).unwrap();
    write_memory(instance.context(), ptr, data);
    ptr
}

pub fn read_memory(ctx: &Ctx, ptr: i32) -> Vec<u8> {
    let slice = to_slice(ctx, ptr);
    let (start, end) = (slice.offset, slice.offset + slice.len);
    let memory = &ctx.memory(0).view::<u8>()[start..end];

    // TODO: there must be a faster way to copy memory
    let mut result = vec![0u8; slice.len];
    for i in 0..slice.len {
        result[i] = memory[i].get();
    }
    result
}

// write_memory returns how many bytes written on success
// negative result is how many bytes requested if too small
pub fn write_memory(ctx: &Ctx, ptr: i32, data: &[u8]) -> i32 {
    let slice = to_slice(ctx, ptr);
    if data.len() > slice.len {
        return -(data.len() as i32);
    }
    if data.len() == 0 {
        return 0;
    }

    let (start, end) = (slice.offset, slice.offset + slice.len);
    let memory = &ctx.memory(0).view::<u8>()[start..end];
    // TODO: there must be a faster way to copy memory
    for i in 0..data.len() {
        memory[i].set(data[i])
    }
    data.len() as i32
}

// to_slice reads in a ptr to slice in wasm memory and constructs the object we can use to access it
fn to_slice(ctx: &Ctx, ptr: i32) -> Slice {
    let buf_ptr = (ptr / 4) as usize; // convert from u8 to i32 offset
    let memory = &ctx.memory(0).view::<i32>();
    Slice {
        offset: memory[buf_ptr].get() as usize,
        len: memory[buf_ptr + 1].get() as usize,
    }
}
