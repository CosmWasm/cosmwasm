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

    static FLOATY: &[u8] = include_bytes!("../../testdata/floaty.wasm");

    #[test]
    fn contract_with_floats_passes_check() {
        let engine = make_compiling_engine(None);
        assert!(compile(&engine, FLOATY).is_ok());
    }

    #[test]
    fn reference_types_dont_panic() {
        const WASM: &str = r#"(module
            (type $t0 (func (param funcref externref)))
            (import "" "" (func $hello (type $t0)))
        )"#;

        let wasm = wat::parse_str(WASM).unwrap();
        let engine = make_compiling_engine(None);
        let error = compile(&engine, &wasm).unwrap_err();
        assert!(error.to_string().contains("FuncRef"));
    }
}
