pub mod cranelift;
pub mod singlepass;

pub use wasmer_runtime_core::backend::{Backend, Compiler};

pub fn compiler_for_backend(backend: Backend) -> Option<Box<dyn Compiler>> {
    match backend {
        #[cfg(any(feature = "cranelift", feature = "default-cranelift"))]
        Backend::Cranelift => Some(cranelift::compiler()),

        #[cfg(any(feature = "singlepass", feature = "default-singlepass"))]
        Backend::Singlepass => Some(singlepass::compiler()),

        _ => None,
    }
}

#[cfg(feature = "default-cranelift")]
pub use cranelift::{backend, compile, get_gas, set_gas};

#[cfg(feature = "default-singlepass")]
pub use singlepass::{backend, compile, get_gas, set_gas};
