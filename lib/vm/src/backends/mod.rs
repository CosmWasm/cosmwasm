pub mod cranelift;
pub mod singlepass;

pub use wasmer_runtime_core::backend::Compiler;

pub fn compiler_for_backend(backend: &str) -> Option<Box<dyn Compiler>> {
    match backend {
        #[cfg(any(feature = "cranelift", feature = "default-cranelift"))]
        "cranelift" => Some(cranelift::compiler()),

        #[cfg(any(feature = "singlepass", feature = "default-singlepass"))]
        "singlepass" => Some(singlepass::compiler()),

        _ => None,
    }
}

#[cfg(feature = "default-cranelift")]
pub use cranelift::{backend, compile, get_gas, set_gas};

#[cfg(feature = "default-singlepass")]
pub use singlepass::{backend, compile, get_gas, set_gas};
