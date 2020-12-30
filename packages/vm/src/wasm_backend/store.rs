use std::convert::TryInto;
use std::sync::Arc;
#[cfg(feature = "cranelift")]
use wasmer::Cranelift;
#[cfg(not(feature = "cranelift"))]
use wasmer::Singlepass;
use wasmer::{
    wasmparser::Operator, BaseTunables, CompilerConfig, Engine, Pages, Store, Target, JIT,
    WASM_PAGE_SIZE,
};
use wasmer_middlewares::Metering;

use crate::middleware::Deterministic;
use crate::size::Size;

use super::limiting_tunables::LimitingTunables;

/// WebAssembly linear memory objects have sizes measured in pages. Each page
/// is 65536 (2^16) bytes. In WebAssembly version 1, a linear memory can have at
/// most 65536 pages, for a total of 2^32 bytes (4 gibibytes).
/// https://github.com/WebAssembly/memory64/blob/master/proposals/memory64/Overview.md
const MAX_WASM_MEMORY: usize = 4 * 1024 * 1024 * 1024;

fn cost(operator: &Operator) -> u64 {
    match operator {
        Operator::LocalGet { .. } | Operator::I32Const { .. } => 9,
        Operator::I32Add { .. } => 12,
        _ => 10,
    }
}

/// Created a store with the default compiler and the given memory limit (in bytes).
/// If memory_limit is None, no limit is applied.
pub fn make_compile_time_store(memory_limit: Option<Size>) -> Store {
    let gas_limit = 0;
    let deterministic = Arc::new(Deterministic::new());
    let metering = Arc::new(Metering::new(gas_limit, cost));

    #[cfg(feature = "cranelift")]
    {
        let mut config = Cranelift::default();
        config.push_middleware(deterministic);
        config.push_middleware(metering);
        let engine = JIT::new(config).engine();
        make_store_with_engine(&engine, memory_limit)
    }

    #[cfg(not(feature = "cranelift"))]
    {
        let mut config = Singlepass::default();
        config.push_middleware(deterministic);
        config.push_middleware(metering);
        let engine = JIT::new(config).engine();
        make_store_with_engine(&engine, memory_limit)
    }
}

/// Created a store with no compiler and the given memory limit (in bytes)
/// If memory_limit is None, no limit is applied.
pub fn make_runtime_store(memory_limit: Size) -> Store {
    let engine = JIT::headless().engine();
    make_store_with_engine(&engine, Some(memory_limit))
}

/// Creates a store from an engine and an optional memory limit.
/// If no limit is set, the no custom tunables will be used.
fn make_store_with_engine(engine: &dyn Engine, memory_limit: Option<Size>) -> Store {
    match memory_limit {
        Some(limit) => {
            let base = BaseTunables::for_target(&Target::default());
            let tunables = LimitingTunables::new(base, limit_to_pages(limit));
            Store::new_with_tunables(engine, tunables)
        }
        None => Store::new(engine),
    }
}

fn limit_to_pages(limit: Size) -> Pages {
    let capped = std::cmp::min(limit.0, MAX_WASM_MEMORY);
    // round down to ensure the limit is less than or equal to the config
    let pages: u32 = (capped / WASM_PAGE_SIZE)
        .try_into()
        .expect("Value must be <= 4 GiB/64KiB, i.e. fit in uint32. This is a bug.");
    Pages(pages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmer::{imports, Artifact, ImportObject, Instance, Memory, Module};
    use wasmer_middlewares::metering::set_remaining_points;

    const TESTING_MEMORY_LIMIT: Size = Size::mebi(16);
    const TESTING_GAS_LIMIT: u64 = 5_000;

    /// A Wasm module with an exported memory (min: 4 pages, max: none)
    const EXPORTED_MEMORY_WAT: &str = r#"(module
        (memory 4)
        (export "memory" (memory 0))
    )"#;

    #[test]
    fn limit_to_pages_works() {
        // rounds down
        assert_eq!(limit_to_pages(Size(0)), Pages(0));
        assert_eq!(limit_to_pages(Size(1)), Pages(0));
        assert_eq!(limit_to_pages(Size::kibi(63)), Pages(0));
        assert_eq!(limit_to_pages(Size::kibi(64)), Pages(1));
        assert_eq!(limit_to_pages(Size::kibi(65)), Pages(1));
        // caps at 4 GiB
        assert_eq!(limit_to_pages(Size::gibi(3)), Pages(49152));
        assert_eq!(limit_to_pages(Size::gibi(4)), Pages(65536));
        assert_eq!(limit_to_pages(Size::gibi(5)), Pages(65536));
    }

    #[test]
    fn make_compile_time_store_applies_memory_limit() {
        let wasm = wat::parse_str(EXPORTED_MEMORY_WAT).unwrap();

        // No limit
        let store = make_compile_time_store(None);
        let module = Module::new(&store, &wasm).unwrap();
        let module_memory = module.info().memories.last().unwrap();
        assert_eq!(module_memory.minimum, Pages(4));
        assert_eq!(module_memory.maximum, None);
        let instance = Instance::new(&module, &ImportObject::new()).unwrap();
        let instance_memory: Memory = instance
            .exports
            .iter()
            .memories()
            .map(|pair| pair.1.clone())
            .next()
            .unwrap();
        assert_eq!(instance_memory.ty().minimum, Pages(4));
        assert_eq!(instance_memory.ty().maximum, None);

        // Set limit
        let store = make_compile_time_store(Some(Size::kibi(23 * 64)));
        let module = Module::new(&store, &wasm).unwrap();
        let module_memory = module.info().memories.last().unwrap();
        assert_eq!(module_memory.minimum, Pages(4));
        assert_eq!(module_memory.maximum, None);
        let instance = Instance::new(&module, &ImportObject::new()).unwrap();
        let instance_memory: Memory = instance
            .exports
            .iter()
            .memories()
            .map(|pair| pair.1.clone())
            .next()
            .unwrap();
        assert_eq!(instance_memory.ty().minimum, Pages(4));
        assert_eq!(instance_memory.ty().maximum, Some(Pages(23)));
    }

    #[test]
    fn make_runtime_store_applies_memory_limit() {
        // Compile
        let serialized = {
            let wasm = wat::parse_str(EXPORTED_MEMORY_WAT).unwrap();
            let store = make_compile_time_store(None);
            let module = Module::new(&store, &wasm).unwrap();
            module.serialize().unwrap()
        };

        // Instantiate with limit
        let store = make_runtime_store(Size::kibi(23 * 64));
        let module = unsafe { Module::deserialize(&store, &serialized) }.unwrap();
        let module_memory = module.info().memories.last().unwrap();
        assert_eq!(module_memory.minimum, Pages(4));
        assert_eq!(module_memory.maximum, None);
        let instance = Instance::new(&module, &ImportObject::new()).unwrap();
        let instance_memory: Memory = instance
            .exports
            .iter()
            .memories()
            .map(|pair| pair.1.clone())
            .next()
            .unwrap();
        assert_eq!(instance_memory.ty().minimum, Pages(4));
        assert_eq!(instance_memory.ty().maximum, Some(Pages(23)));
    }

    #[test]
    fn can_compile_ahead_of_time_and_run_from_artifact() {
        let wasm = wat::parse_str(
            r#"(module
            (type $t0 (func (param i32) (result i32)))
            (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
                get_local $p0
                i32.const 1
                i32.add)
            )"#,
        )
        .unwrap();

        // Compile module
        let compile_time_store = make_compile_time_store(None);
        let original = Module::new(&compile_time_store, &wasm).unwrap();

        // Ensure original module can be executed
        {
            let instance = Instance::new(&original, &imports! {}).unwrap();
            set_remaining_points(&instance, TESTING_GAS_LIMIT);
            let add_one = instance.exports.get_function("add_one").unwrap();
            let result = add_one.call(&[42.into()]).unwrap();
            assert_eq!(result[0].unwrap_i32(), 43);
        }

        // Create second module via artifact
        let artifact: Arc<dyn Artifact> = Arc::clone(original.artifact());
        let run_time_store = make_runtime_store(TESTING_MEMORY_LIMIT);
        let restored = Module::from_artifact(&run_time_store, artifact);

        // Ensure restored module can be executed
        {
            let instance = Instance::new(&restored, &imports! {}).unwrap();
            set_remaining_points(&instance, TESTING_GAS_LIMIT);
            let add_one = instance.exports.get_function("add_one").unwrap();
            let result = add_one.call(&[42.into()]).unwrap();
            assert_eq!(result[0].unwrap_i32(), 43);
        }
    }
}
