use std::sync::Arc;

use wasmer::{Engine, Module, ModuleMiddleware};

use crate::errors::VmResult;
use crate::wasm_backend::make_engine;

/// Compiles a given Wasm bytecode into a module.
pub fn compile(
    code: &[u8],
    middlewares: &[Arc<dyn ModuleMiddleware>],
) -> VmResult<(Engine, Module)> {
    let engine = make_engine(middlewares);
    let module = Module::new(&engine, code)?;
    Ok((engine, module))
}

#[cfg(test)]
mod tests {
    use super::*;

    static CONTRACT: &[u8] = include_bytes!("../../testdata/floaty.wasm");

    #[test]
    fn contract_with_floats_fails_check() {
        let err = compile(CONTRACT, &[]).unwrap_err();
        assert!(err.to_string().contains("Float operator detected:"));
    }
}
