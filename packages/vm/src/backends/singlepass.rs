#![cfg(any(feature = "singlepass", feature = "default-singlepass"))]
use wasmer_middleware_common::metering;
use wasmer_runtime_core::{
    backend::Compiler,
    codegen::{MiddlewareChain, StreamingCompiler},
    compile_with,
    module::Module,
    Instance as WasmerInstance,
};
use wasmer_singlepass_backend::ModuleCodeGenerator as SinglePassMCG;

use crate::errors::VmResult;
use crate::middleware::DeterministicMiddleware;

/// In Wasmer, The gas limit on instances is set during compile time and is included in the compiled binaries.
/// This causes issues when trying to reuse the same precompiled binaries for another instance with a different
/// gas limit.
/// https://github.com/wasmerio/wasmer/pull/996
/// To work around that, we set the gas limit of all Wasmer instances to this very-high gas limit value, under
/// the assumption that users won't request more than this amount of gas. Then to set a gas limit below that figure,
/// we pretend to consume the difference between the two in `set_gas_limit`, so the amount of units left is equal to
/// the requested gas limit.
pub const GAS_LIMIT: u64 = 10_000_000_000;

pub fn compile(code: &[u8]) -> VmResult<Module> {
    let module = compile_with(code, compiler().as_ref())?;
    Ok(module)
}

pub fn compiler() -> Box<dyn Compiler> {
    let c: StreamingCompiler<SinglePassMCG, _, _, _, _> = StreamingCompiler::new(move || {
        let mut chain = MiddlewareChain::new();
        chain.push(DeterministicMiddleware::new());
        chain.push(metering::Metering::new(GAS_LIMIT));
        chain
    });
    Box::new(c)
}

pub fn backend() -> &'static str {
    "singlepass"
}

/// Set the amount of gas units that can be used in the instance.
pub fn set_gas_limit(instance: &mut WasmerInstance, limit: u64) {
    let used = GAS_LIMIT.saturating_sub(limit);
    metering::set_points_used(instance, used)
}

/// Get how many more gas units can be used in the instance.
pub fn get_gas_left(instance: &WasmerInstance) -> u64 {
    let used = metering::get_points_used(instance);
    // when running out of gas, get_points_used can exceed GAS_LIMIT
    GAS_LIMIT.saturating_sub(used)
}
