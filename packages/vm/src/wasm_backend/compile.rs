use wasmer::{Engine, Module};

use crate::errors::VmResult;

/// Compiles a given Wasm bytecode into a module.
pub fn compile(engine: &Engine, code: &[u8]) -> VmResult<Module> {
    let module = Module::new(&engine, code)?;
    Ok(module)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wasm_backend::make_compiling_engine;

    static CONTRACT: &[u8] = include_bytes!("../../testdata/floaty.wasm");

    #[test]
    fn contract_with_floats_fails_check() {
        let engine = make_compiling_engine(None);
        let err = compile(&engine, CONTRACT).unwrap_err();
        assert!(err.to_string().contains("Float operator detected:"));
    }
}
