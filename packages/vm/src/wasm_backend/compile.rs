// use wasmer_middleware_common::metering;
use wasmer::Module;

use crate::errors::VmResult;
use crate::size::Size;

use super::store::make_compile_time_store;
// use crate::middleware::DeterministicMiddleware;

/// Compiles a given Wasm bytecode into a module.
/// The resulting module has no memory limit. This
/// should only be used to compile for caching.
pub fn compile_only(code: &[u8]) -> VmResult<Module> {
    let store = make_compile_time_store(None);
    let module = Module::new(&store, code)?;
    Ok(module)
}

/// Compiles a given Wasm bytecode into a module.
/// The given memory limit (in bytes) is used when memories are created.
pub fn compile_and_use(code: &[u8], memory_limit: Option<Size>) -> VmResult<Module> {
    let store = make_compile_time_store(memory_limit);
    let module = Module::new(&store, code)?;
    Ok(module)
}
