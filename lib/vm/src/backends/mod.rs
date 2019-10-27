pub mod cranelift;
pub mod singlepass;

#[cfg(feature = "default-cranelift")]
pub use cranelift::{backend, compile};

#[cfg(feature = "default-singlepass")]
pub use singlepass::{backend, compile};
