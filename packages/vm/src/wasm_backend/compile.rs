use wasmer::{Engine, Module};

use crate::errors::VmResult;
use crate::internals::make_compiling_engine;
use crate::parsed_wasm::ParsedWasm;
use crate::Size;

/// Compiles a given Wasm bytecode into a module using custom engine.
pub fn compile(engine: &Engine, code: &[u8]) -> VmResult<Module> {
    let module = Module::new(&engine, code)?;
    Ok(module)
}

/// Compiles a given Wasm bytecode into a module using compiling engine.
pub fn compile_module(wasm: &[u8], memory_limit: Option<Size>) -> VmResult<(Module, Engine)> {
    let parsed_wasm = ParsedWasm::parse(wasm)?;
    let engine = make_compiling_engine(memory_limit, Some(parsed_wasm));
    let module = compile(&engine, wasm)?;
    Ok((module, engine))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsed_wasm::ParsedWasm;
    use crate::wasm_backend::make_compiling_engine;

    static CONTRACT: &[u8] = include_bytes!("../../testdata/floaty.wasm");

    #[test]
    fn contract_with_floats_passes_check() {
        let parsed_wasm = ParsedWasm::parse(CONTRACT).unwrap();
        let engine = make_compiling_engine(None, Some(parsed_wasm));
        assert!(compile(&engine, CONTRACT).is_ok());
    }
}
