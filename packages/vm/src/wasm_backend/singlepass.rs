// use wasmer_middleware_common::metering;
use wasmer::{Module, Store};
use wasmer_compiler_singlepass::Singlepass;
use wasmer_engine_jit::JIT;

use crate::errors::VmResult;
// use crate::middleware::DeterministicMiddleware;

pub fn compile(code: &[u8]) -> VmResult<Module> {
    let compiler = Singlepass::default();
    let engine = JIT::new(&compiler).engine();
    let store = Store::new(&engine);
    let module = Module::new(&store, code)?;
    Ok(module)
}
