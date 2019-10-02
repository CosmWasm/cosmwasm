pub mod contract;
pub mod storage;
pub mod types;

#[cfg(target_arch = "wasm32")]
mod memory;
#[cfg(target_arch = "wasm32")]
pub use crate::memory::{allocate, deallocate};

#[cfg(target_arch = "wasm32")]
mod api;
#[cfg(target_arch = "wasm32")]
pub use crate::api::{init_wrapper, send_wrapper};
