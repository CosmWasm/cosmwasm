mod memory;
mod exports;
pub mod wasmer;

pub use crate::exports::{do_read, do_write, setup_context};
pub use crate::memory::{read_memory, write_memory, allocate};
