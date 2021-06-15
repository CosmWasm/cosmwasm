use wasmer::Module;

use crate::errors::VmResult;
use crate::size::Size;

use super::store::make_compile_time_store;

/// Compiles a given Wasm bytecode into a module.
/// The given memory limit (in bytes) is used when memories are created.
/// If no memory limit is passed, the resulting compiled module should
/// not be used for execution.
pub fn compile(code: &[u8], memory_limit: Option<Size>) -> VmResult<Module> {
    let store = make_compile_time_store(memory_limit);
    let module = Module::new(&store, code)?;
    Ok(module)
}
