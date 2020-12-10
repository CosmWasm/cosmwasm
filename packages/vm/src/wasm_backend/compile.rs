// use wasmer_middleware_common::metering;
use wasmer::Module;

use crate::errors::VmResult;
use crate::size::Size;

use super::store::make_store;
// use crate::middleware::DeterministicMiddleware;

/// Compiles a given Wasm bytecode into a module.
/// The given memory limit (in bytes) is used when memories are created.
pub fn compile(code: &[u8], memory_limit: Option<Size>) -> VmResult<Module> {
    let store = make_store(memory_limit);
    let module = Module::new(&store, code)?;
    Ok(module)
}
