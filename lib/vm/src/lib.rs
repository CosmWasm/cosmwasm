mod calls;
mod exports;
mod memory;
mod wasm_store;
mod wasmer;

pub use crate::calls::{call_handle, call_init};
pub use crate::memory::{allocate, read_memory};
pub use crate::wasmer::{instantiate, with_storage};
