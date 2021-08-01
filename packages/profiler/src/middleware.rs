use std::collections::HashMap;
use std::mem;
use std::sync::{Arc, Mutex};

use loupe::{MemoryUsage, MemoryUsageTracker};
use wasmer::{FunctionMiddleware, ModuleMiddleware};

use crate::operators::OperatorSymbol;

#[non_exhaustive]
#[derive(Debug, MemoryUsage)]
pub struct Profiling {
    block_map: Arc<Mutex<Option<BlockStore>>>,
}

impl Profiling {
    pub fn new() -> Self {
        Self {
            block_map: Arc::new(Mutex::new(None)),
        }
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

use std::hash::Hash;

/// Stores the non-branching Wasm code blocks so that the exact
/// list of operators can be looked up by hash later.
#[derive(Debug)]
struct BlockStore {
    inner: HashMap<u64, Vec<OperatorSymbol>>,
}

impl BlockStore {
    fn register_block(&mut self, v: Vec<OperatorSymbol>) -> u64 {
        let hash = calculate_hash(&v);
        self.inner.insert(hash, v);
        hash
    }

    fn get_block(&self, hash: u64) -> Option<&Vec<OperatorSymbol>> {
        self.inner.get(&hash)
    }
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    use std::hash::Hasher as _;

    let mut s = std::collections::hash_map::DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

impl MemoryUsage for BlockStore {
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self)
            + self
                .inner
                .iter()
                .map(|(key, value)| key.size_of_val(tracker) + mem::size_of_val(value))
                .sum::<usize>()
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
