pub mod cranelift;
pub mod singlepass;

pub use wasmer_runtime_core::backend::Compiler;
use wasmer_runtime_core::vm::Ctx;

pub fn compiler_for_backend(backend: &str) -> Option<Box<dyn Compiler>> {
    match backend {
        #[cfg(any(feature = "cranelift", feature = "default-cranelift"))]
        "cranelift" => Some(cranelift::compiler()),

        #[cfg(any(feature = "singlepass", feature = "default-singlepass"))]
        "singlepass" => Some(singlepass::compiler()),

        _ => None,
    }
}

/// Decreases gas left by the given amount. If the amount exceeds the available gas,
/// the remaining gas is set to 0.
pub fn decrease_gas_left(ctx: &mut Ctx, amount: u64) {
    let remaining = get_gas_left(ctx).saturating_sub(amount);
    set_gas_left(ctx, remaining);
}

#[cfg(feature = "default-cranelift")]
pub use cranelift::{backend, compile, get_gas_left, set_gas_left};

#[cfg(feature = "default-singlepass")]
pub use singlepass::{backend, compile, get_gas_left, set_gas_left};
