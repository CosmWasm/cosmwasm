pub mod cranelift;
pub mod singlepass;

#[cfg(feature = "default-cranelift")]
pub use cranelift::{backend, compile, get_gas, set_gas};

#[cfg(feature = "default-singlepass")]
pub use singlepass::{backend, compile, get_gas, set_gas};
