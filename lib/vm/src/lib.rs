mod memory;
mod exports;
mod wasmer;
mod calls;

pub use crate::memory::{read_memory, allocate};
pub use crate::wasmer::instantiate;
pub use crate::calls::{call_init, call_send};
