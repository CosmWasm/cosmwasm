use loupe::MemoryUsage;
use wasmer::{FunctionMiddleware, ModuleMiddleware};

#[non_exhaustive]
#[derive(Debug, MemoryUsage)]
pub struct Profiling;

impl Profiling {
    pub fn new() -> Profiling {
        Profiling
    }
}

impl ModuleMiddleware for Profiling {
    fn generate_function_middleware(
        &self,
        _local_function_index: wasmer::LocalFunctionIndex,
    ) -> Box<dyn wasmer::FunctionMiddleware> {
        todo!()
    }
}

#[derive(Debug)]
struct FunctionProfiling;

impl FunctionMiddleware for FunctionProfiling {
    fn feed<'a>(
        &mut self,
        operator: wasmer::wasmparser::Operator<'a>,
        state: &mut wasmer::MiddlewareReaderState<'a>,
    ) -> Result<(), wasmer::MiddlewareError> {
        state.push_operator(operator);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;
    use wasmer::{
        imports, wat2wasm, CompilerConfig, Cranelift, Instance, Module, Store, Universal,
    };

    const WAT: &[u8] = br#"
(module)
"#;

    #[test]
    fn middleware_runs() {
        let profiling = Arc::new(Profiling::new());

        // Create the module with our middleware.
        let mut compiler_config = Cranelift::default();
        compiler_config.push_middleware(profiling.clone());
        let store = Store::new(&Universal::new(compiler_config).engine());
        let wasm = wat2wasm(WAT).unwrap();
        let module = Module::new(&store, wasm).unwrap();

        // Instantiate the module with our imports.
        let imports = imports! {};
        let _instance = Instance::new(&module, &imports).unwrap();
    }
}
