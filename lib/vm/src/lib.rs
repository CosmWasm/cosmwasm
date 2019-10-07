mod calls;
mod exports;
mod memory;
mod wasmer;

pub use crate::calls::{call_init, call_handle};
pub use crate::memory::{allocate, read_memory};
pub use crate::wasmer::{instantiate, with_storage};
