use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use cosmwasm_vm::{
    testing::{MockApi, MockQuerier, MockStorage},
    Backend, Instance,
};
use loupe::MemoryUsage;
use wasmer::{
    internals::WithEnv, wasmparser::Operator, Exports, Function, FunctionMiddleware, HostFunction,
    LocalFunctionIndex, ModuleMiddleware, WasmerEnv,
};
use wasmer_types::{FunctionIndex, ImportIndex};

use crate::{code_blocks::BlockStore, operators::OperatorSymbol};

pub enum Module<'d> {
    Path(&'d Path),
    #[cfg(test)]
    Bytes(&'d [u8]),
}

impl<'d> Module<'d> {
    pub fn from_path<P: AsRef<Path> + ?Sized>(path: &'d P) -> Self {
        Self::Path(path.as_ref())
    }

    #[cfg(test)]
    pub fn from_bytes(bytes: &'d [u8]) -> Self {
        Self::Bytes(bytes)
    }

    pub fn instrument<Env, F1, F2>(
        &self,
        block_store: Arc<Mutex<BlockStore>>,
        env: Env,
        start_measurement_fn: F1,
        take_measurement_fn: F2,
    ) -> InstrumentedInstance
    where
        Env: WasmerEnv + 'static,
        F1: HostFunction<(u32, u32), (), WithEnv, Env>,
        F2: HostFunction<(u32, u32, u64), (), WithEnv, Env>,
    {
        let profiling = Arc::new(Profiling::new(block_store));

        // Create the module with our middleware.
        // let mut compiler_config = Cranelift::default();
        // compiler_config.push_middleware(profiling.clone());
        // let store = Store::new(&Universal::new(compiler_config).engine());
        let mut walrus_module = match self {
            Module::Path(path) => walrus::Module::from_file(path).unwrap(),
            #[cfg(test)]
            Module::Bytes(bytes) => walrus::Module::from_buffer(bytes).unwrap(),
        };
        add_imports(&mut walrus_module);
        let wasm = walrus_module.emit_wasm();
        //let wasmer_module = wasmer::Module::new(&store, wasm).unwrap();

        let wasmer_module =
            cosmwasm_vm::internals::compile(&wasm, None, &[profiling.clone()]).unwrap();
        let store = wasmer_module.store();

        // Mock imports that do nothing.
        let mut fns_to_import = Exports::new();
        fns_to_import.insert(
            "start_measurement",
            Function::new_native_with_env(store, env.clone(), start_measurement_fn),
        );
        fns_to_import.insert(
            "take_measurement",
            Function::new_native_with_env(store, env, take_measurement_fn),
        );

        let backend = Backend {
            api: MockApi::default(),
            storage: MockStorage::default(),
            querier: MockQuerier::new(&[]),
        };
        let instance = cosmwasm_vm::internals::instance_from_module(
            &wasmer_module,
            backend,
            999999999,
            false,
            Some(vec![("profiling", fns_to_import)].into_iter().collect()),
        )
        .unwrap();

        InstrumentedInstance {
            profiling,
            instance,
        }
    }
}

type MockInstance = Instance<MockApi, MockStorage, MockQuerier>;

pub struct InstrumentedInstance {
    #[allow(dead_code)]
    profiling: Arc<Profiling>,
    instance: MockInstance,
}

impl InstrumentedInstance {
    pub fn vm_instance(&mut self) -> &mut MockInstance {
        &mut self.instance
    }
}

/// Add the imports we need to make instrumentation work.
/// Returns the ids for both fns.
fn add_imports(module: &mut walrus::Module) -> (usize, usize) {
    use walrus::ValType::*;

    let start_type = module.types.add(&[I32, I32], &[]);
    let take_type = module.types.add(&[I32, I32, I64], &[]);

    let (fn1, _) = module.add_import_func("profiling", "start_measurement", start_type);
    let (fn2, _) = module.add_import_func("profiling", "take_measurement", take_type);

    (fn1.index(), fn2.index())
}

#[non_exhaustive]
#[derive(Debug, MemoryUsage)]
pub struct Profiling {
    block_store: Arc<Mutex<BlockStore>>,
    indexes: Mutex<Option<ProfilingIndexes>>,
}

impl Profiling {
    pub fn new(block_store: Arc<Mutex<BlockStore>>) -> Self {
        Self {
            block_store,
            indexes: Mutex::new(None),
        }
    }
}

impl ModuleMiddleware for Profiling {
    fn generate_function_middleware(
        &self,
        local_function_index: wasmer::LocalFunctionIndex,
    ) -> Box<dyn wasmer::FunctionMiddleware> {
        Box::new(FunctionProfiling::new(
            self.block_store.clone(),
            self.indexes.lock().unwrap().clone().unwrap(),
            local_function_index,
        ))
    }

    fn transform_module_info(&self, module_info: &mut wasmer_vm::ModuleInfo) {
        let mut indexes = self.indexes.lock().unwrap();

        if indexes.is_some() {
            panic!("Profiling::transform_module_info: Attempting to use a `Profiling` middleware from multiple modules.");
        }

        let fn1 = module_info
            .imports
            .iter()
            .find_map(|((module, field, _), index)| {
                if (module.as_str(), field.as_str()) == ("profiling", "start_measurement") {
                    if let ImportIndex::Function(fn_index) = index {
                        return Some(fn_index);
                    }
                }
                None
            })
            .unwrap();

        let fn2 = module_info
            .imports
            .iter()
            .find_map(|((module, field, _), index)| {
                if (module.as_str(), field.as_str()) == ("profiling", "take_measurement") {
                    if let ImportIndex::Function(fn_index) = index {
                        return Some(fn_index);
                    }
                }
                None
            })
            .unwrap();

        *indexes = Some(ProfilingIndexes {
            start_measurement: *fn1,
            take_measurement: *fn2,
        });
    }
}

#[derive(Debug)]
struct FunctionProfiling {
    block_store: Arc<Mutex<BlockStore>>,
    accumulated_ops: Vec<OperatorSymbol>,
    indexes: ProfilingIndexes,
    block_count: u32,
    fn_index: LocalFunctionIndex,
}

impl FunctionProfiling {
    fn new(
        block_store: Arc<Mutex<BlockStore>>,
        indexes: ProfilingIndexes,
        fn_index: LocalFunctionIndex,
    ) -> Self {
        Self {
            block_store,
            accumulated_ops: Vec::new(),
            indexes,
            block_count: 0,
            fn_index,
        }
    }
}

impl FunctionMiddleware for FunctionProfiling {
    fn feed<'a>(
        &mut self,
        operator: wasmer::wasmparser::Operator<'a>,
        state: &mut wasmer::MiddlewareReaderState<'a>,
    ) -> Result<(), wasmer::MiddlewareError> {
        // Possible sources and targets of a branch. Finalize the cost of the previous basic block and perform necessary checks.
        match operator {
            Operator::Loop { .. } // loop headers are branch targets
            | Operator::End // block ends are branch targets
            | Operator::Else // "else" is the "end" of an if branch
            | Operator::Br { .. } // branch source
            | Operator::BrTable { .. } // branch source
            | Operator::BrIf { .. } // branch source
            | Operator::Call { .. } // function call - branch source
            | Operator::CallIndirect { .. } // function call - branch source
            | Operator::Return // end of function - branch source
            => {
                if !self.accumulated_ops.is_empty() {
                    let mut store = self.block_store.lock().unwrap();
                    let block_id = store.register_block(std::mem::take(&mut self.accumulated_ops));

                    // We're at the end of a code block. Finalize the measurement.
                    state.extend(&[
                        Operator::I32Const { value: self.fn_index.as_u32() as i32 },
                        Operator::I32Const { value: self.block_count as i32 },
                        Operator::I64Const { value: block_id.as_u64() as i64 },
                        Operator::Call{ function_index: self.indexes.take_measurement.as_u32() },
                    ]);
                }
            }
            _ => {
                if self.accumulated_ops.is_empty() {
                    // We know we're at the beginning of a code block.
                    // Call start_measurement before executing it.
                    state.extend(&[
                        Operator::I32Const { value: self.fn_index.as_u32() as i32 },
                        Operator::I32Const { value: self.block_count as i32 },
                        Operator::Call{ function_index: self.indexes.start_measurement.as_u32() },
                    ]);
                }
                self.accumulated_ops.push(OperatorSymbol::from(&operator));
            }
        }

        state.push_operator(operator);
        Ok(())
    }
}

#[derive(Debug, MemoryUsage, Clone)]
struct ProfilingIndexes {
    start_measurement: FunctionIndex,
    take_measurement: FunctionIndex,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::code_blocks::CodeBlock;

    use std::sync::Arc;
    use wasmer::{wat2wasm, WasmerEnv};

    const WAT: &[u8] = br#"
    (module
    (type $t0 (func (param i32) (result i32)))
    (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
        get_local $p0
        i32.const 1
        i32.add)
    (func $multisub (export "multisub") (type $t0) (param $p0 i32) (result i32)
        get_local $p0
        i32.const 2
        i32.mul
        call $sub_one
        i32.const 1
        i32.sub)
    (func $sub_one (type $t0) (param $p0 i32) (result i32)
        get_local $p0
        i32.const 1
        i32.sub))
    "#;

    struct Fixture {
        instance: InstrumentedInstance,
    }

    #[derive(Debug, Clone, WasmerEnv)]
    struct FixtureEnv {
        start_calls: Arc<Mutex<Vec<(u32, u32)>>>,
        end_calls: Arc<Mutex<Vec<(u32, u32, u64)>>>,
    }

    impl FixtureEnv {
        fn new() -> Self {
            Self {
                start_calls: Arc::new(Mutex::new(Vec::new())),
                end_calls: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    impl Fixture {
        fn new() -> Self {
            let wasm = wat2wasm(WAT).unwrap();
            let module = Module::from_bytes(&wasm);

            let env = FixtureEnv::new();
            let start_measurement_fn = |env: &FixtureEnv, fun: u32, block: u32| {
                env.start_calls.lock().unwrap().push((fun, block));
            };
            let take_measurement_fn = |env: &FixtureEnv, fun: u32, block: u32, hash: u64| {
                env.end_calls.lock().unwrap().push((fun, block, hash));
            };

            let block_store = Arc::new(Mutex::new(BlockStore::new()));

            Self {
                instance: module.instrument(
                    block_store,
                    env,
                    start_measurement_fn,
                    take_measurement_fn,
                ),
            }
        }
    }

    // #[test]
    // fn instrumentation_does_not_mess_up_local_fns() {
    //     let fixture = Fixture::new();

    //     let result = fixture.add_one().call(&[Value::I32(42)]).unwrap();
    //     assert_eq!(result[0], Value::I32(43));

    //     let result = fixture.multisub().call(&[Value::I32(4)]).unwrap();
    //     assert_eq!(result[0], Value::I32(6));
    // }

    #[test]
    fn instrumentation_registers_code_blocks() {
        let fixture = Fixture::new();

        let block_store = fixture.instance.profiling.block_store.lock().unwrap();
        assert_eq!(block_store.len(), 4);
        println!("{:?}", block_store);

        // The body of $add_one.
        let expected_block = CodeBlock::from(vec![
            OperatorSymbol::LocalGet,
            OperatorSymbol::I32Const,
            OperatorSymbol::I32Add,
        ]);
        let block = block_store.get_block(expected_block.get_hash());
        assert_eq!(block, Some(&expected_block));

        // The body of $sub_one
        let expected_block = CodeBlock::from(vec![
            OperatorSymbol::LocalGet,
            OperatorSymbol::I32Const,
            OperatorSymbol::I32Sub,
        ]);
        let block = block_store.get_block(expected_block.get_hash());
        assert_eq!(block, Some(&expected_block));

        // The body of $multisub until the `call` instruction.
        let expected_block = CodeBlock::from(vec![
            OperatorSymbol::LocalGet,
            OperatorSymbol::I32Const,
            OperatorSymbol::I32Mul,
        ]);
        let block = block_store.get_block(expected_block.get_hash());
        assert_eq!(block, Some(&expected_block));

        // The body of $multisub after the `call` instruction.
        let expected_block =
            CodeBlock::from(vec![OperatorSymbol::I32Const, OperatorSymbol::I32Sub]);
        let block = block_store.get_block(expected_block.get_hash());
        assert_eq!(block, Some(&expected_block));
    }

    // #[test]
    // fn instrumentation_works() {
    //     let fixture = Fixture::new();

    //     fixture.add_one().call(&[Value::I32(42)]).unwrap();
    //     fixture.multisub().call(&[Value::I32(4)]).unwrap();

    //     let start_measurement_calls = fixture.instance.env.start_calls.lock().unwrap();
    //     let take_measurement_calls = fixture.instance.env.end_calls.lock().unwrap();

    //     assert_eq!(*start_measurement_calls, [(1, 0), (0, 0), (2, 0), (0, 0)]);
    //     assert_eq!(
    //         *take_measurement_calls,
    //         [
    //             (1, 0, 8893795678467789947),
    //             (0, 0, 14205319683222620312),
    //             (2, 0, 10205745157157101990),
    //             (0, 0, 13601349546502136404)
    //         ]
    //     );
    // }
}
