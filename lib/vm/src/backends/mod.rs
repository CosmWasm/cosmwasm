pub mod cranelift;
pub mod singlepass;
pub mod llvm;

#[cfg(feature = "default-cranelift")]
pub use cranelift::{backend, compile, get_gas, set_gas};

#[cfg(feature = "default-singlepass")]
pub use singlepass::{backend, compile, get_gas, set_gas};

#[cfg(feature = "default-llvm")]
pub use llvm::{backend, compile, get_gas, set_gas};
