mod memory;
mod exports;
mod wasmer;

pub use crate::memory::{read_memory, allocate};
pub use crate::wasmer::{Func, instantiate};
