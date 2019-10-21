mod cache;
mod calls;
mod exports;
mod memory;
mod wasm_store;
mod wasmer;

pub use crate::cache::Cache;
pub use crate::calls::{call_handle, call_handle_raw, call_init, call_init_raw};
pub use crate::memory::{allocate, read_memory};
pub use crate::wasmer::{instantiate, with_storage};
