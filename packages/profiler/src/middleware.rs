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

/// Stores non-branching Wasm code blocks so that the exact
/// list of operators can be looked up by hash later.
#[derive(Debug)]
struct BlockStore {
    inner: HashMap<u64, Vec<OperatorSymbol>>,
}

impl BlockStore {
    fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    fn register_block<'b, Op>(&mut self, block: &'b [Op]) -> u64
    where
        &'b Op: Into<OperatorSymbol>,
    {
        let v: Vec<OperatorSymbol> = block.iter().map(|item| item.into()).collect();

        let hash = calculate_hash(&v);
        self.inner.insert(hash, v);
        hash
    }

    fn get_block(&self, hash: u64) -> Option<&[OperatorSymbol]> {
        self.inner.get(&hash).map(|x| x.as_slice())
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
    use wasmer::wasmparser::{Operator, Type, TypeOrFuncType};
    use wasmer::{
        imports, wat2wasm, CompilerConfig, Cranelift, Instance, Module, Store, Universal,
    };

    const WAT: &[u8] = br#"
(module)
"#;

    #[test]
    fn block_store() {
        let mut store = BlockStore::new();

        let code_block1 = [
            Operator::GlobalGet { global_index: 333 },
            Operator::I64Const { value: 555 as i64 },
            Operator::I64LtU,
            Operator::If {
                ty: TypeOrFuncType::Type(Type::EmptyBlockType),
            },
            Operator::I32Const { value: 1 },
            Operator::GlobalSet { global_index: 222 },
            Operator::Unreachable,
            Operator::End,
        ];
        let code_block2 = [
            Operator::GlobalGet { global_index: 333 },
            Operator::I64Const { value: 222 },
            Operator::I64Sub,
            Operator::GlobalSet { global_index: 333 },
        ];

        let code_block1_hash = store.register_block(&code_block1);
        let code_block2_hash = store.register_block(&code_block2);
        let code_block1_another_hash = store.register_block(&code_block1);

        assert_eq!(code_block1_hash, code_block1_another_hash);
        assert_ne!(code_block1_hash, code_block2_hash);

        let cb1_expected = [
            OperatorSymbol::GlobalGet,
            OperatorSymbol::I64Const,
            OperatorSymbol::I64LtU,
            OperatorSymbol::If,
            OperatorSymbol::I32Const,
            OperatorSymbol::GlobalSet,
            OperatorSymbol::Unreachable,
            OperatorSymbol::End,
        ];

        let cb2_expected = [
            OperatorSymbol::GlobalGet,
            OperatorSymbol::I64Const,
            OperatorSymbol::I64Sub,
            OperatorSymbol::GlobalSet,
        ];

        assert_eq!(store.get_block(code_block1_hash), Some(&cb1_expected[..]));
        assert_eq!(store.get_block(code_block2_hash), Some(&cb2_expected[..]));
        assert_eq!(store.get_block(234), None);
    }

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
