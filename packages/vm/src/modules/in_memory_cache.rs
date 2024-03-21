use clru::{CLruCache, CLruCacheConfig, WeightScale};
use std::collections::hash_map::RandomState;
use std::num::NonZeroUsize;

use cosmwasm_std::Checksum;

use super::cached_module::CachedModule;
use crate::{Size, VmError, VmResult};

// Minimum module size.
// Based on `examples/module_size.sh`, and the cosmwasm-plus contracts.
// We use an estimated *minimum* module size in order to compute a number of pre-allocated entries
// that are enough to handle a size-limited cache without requiring re-allocation / resizing.
// This will incurr an extra memory cost for the unused entries, but it's negligible:
// Assuming the cost per entry is 48 bytes, 10000 entries will have an extra cost of just ~500 kB.
// Which is a very small percentage (~0.03%) of our typical cache memory budget (2 GB).
const MINIMUM_MODULE_SIZE: Size = Size::kibi(250);

#[derive(Debug)]
struct SizeScale;

impl WeightScale<Checksum, CachedModule> for SizeScale {
    #[inline]
    fn weight(&self, key: &Checksum, value: &CachedModule) -> usize {
        std::mem::size_of_val(key) + value.size_estimate
    }
}

/// An in-memory module cache
pub struct InMemoryCache {
    modules: Option<CLruCache<Checksum, CachedModule, RandomState, SizeScale>>,
}

impl InMemoryCache {
    /// Creates a new cache with the given size (in bytes)
    /// and pre-allocated entries.
    pub fn new(size: Size) -> Self {
        let preallocated_entries = size.0 / MINIMUM_MODULE_SIZE.0;

        InMemoryCache {
            modules: if size.0 > 0 {
                Some(CLruCache::with_config(
                    CLruCacheConfig::new(NonZeroUsize::new(size.0).unwrap())
                        .with_memory(preallocated_entries)
                        .with_scale(SizeScale),
                ))
            } else {
                None
            },
        }
    }

    pub fn store(&mut self, checksum: &Checksum, cached_module: CachedModule) -> VmResult<()> {
        if let Some(modules) = &mut self.modules {
            modules
                .put_with_weight(*checksum, cached_module)
                .map_err(|e| VmError::cache_err(format!("{e:?}")))?;
        }
        Ok(())
    }

    /// Looks up a module in the cache and creates a new module
    pub fn load(&mut self, checksum: &Checksum) -> VmResult<Option<CachedModule>> {
        if let Some(modules) = &mut self.modules {
            match modules.get(checksum) {
                Some(cached) => Ok(Some(cached.clone())),
                None => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Returns the number of elements in the cache.
    pub fn len(&self) -> usize {
        self.modules
            .as_ref()
            .map(|modules| modules.len())
            .unwrap_or_default()
    }

    /// Returns cumulative size of all elements in the cache.
    ///
    /// This is based on the values provided with `store`. No actual
    /// memory size is measured here.
    pub fn size(&self) -> usize {
        self.modules
            .as_ref()
            .map(|modules| modules.weight())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wasm_backend::{compile, make_compiling_engine, make_runtime_engine};
    use std::mem;
    use wasmer::{imports, Instance as WasmerInstance, Module, Store};
    use wasmer_middlewares::metering::set_remaining_points;

    const TESTING_MEMORY_LIMIT: Option<Size> = Some(Size::mebi(16));
    const TESTING_GAS_LIMIT: u64 = 500_000;
    // Based on `examples/module_size.sh`
    const TESTING_WASM_SIZE_FACTOR: usize = 18;

    const WAT1: &str = r#"(module
        (type $t0 (func (param i32) (result i32)))
        (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
            local.get $p0
            i32.const 1
            i32.add)
        )"#;
    const WAT2: &str = r#"(module
        (type $t0 (func (param i32) (result i32)))
        (func $add_one (export "add_two") (type $t0) (param $p0 i32) (result i32)
            local.get $p0
            i32.const 2
            i32.add)
        )"#;
    const WAT3: &str = r#"(module
        (type $t0 (func (param i32) (result i32)))
        (func $add_one (export "add_three") (type $t0) (param $p0 i32) (result i32)
            local.get $p0
            i32.const 3
            i32.add)
        )"#;

    #[test]
    fn check_element_sizes() {
        let key_size = mem::size_of::<Checksum>();
        assert_eq!(key_size, 32);

        let value_size = mem::size_of::<Module>();
        assert_eq!(value_size, 8);

        // Just in case we want to go that route
        let boxed_value_size = mem::size_of::<Box<Module>>();
        assert_eq!(boxed_value_size, 8);
    }

    #[test]
    fn in_memory_cache_run() {
        let mut cache = InMemoryCache::new(Size::mebi(200));

        // Create module
        let wasm = wat::parse_str(WAT1).unwrap();
        let checksum = Checksum::generate(&wasm);

        // Module does not exist
        let cache_entry = cache.load(&checksum).unwrap();
        assert!(cache_entry.is_none());

        // Compile module
        let engine = make_compiling_engine(TESTING_MEMORY_LIMIT);
        let original = compile(&engine, &wasm).unwrap();

        // Ensure original module can be executed
        {
            let mut store = Store::new(engine.clone());
            let instance = WasmerInstance::new(&mut store, &original, &imports! {}).unwrap();
            set_remaining_points(&mut store, &instance, TESTING_GAS_LIMIT);
            let add_one = instance.exports.get_function("add_one").unwrap();
            let result = add_one.call(&mut store, &[42.into()]).unwrap();
            assert_eq!(result[0].unwrap_i32(), 43);
        }

        // Store module
        let module = CachedModule {
            module: original,
            engine: make_runtime_engine(TESTING_MEMORY_LIMIT),
            size_estimate: wasm.len() * TESTING_WASM_SIZE_FACTOR,
        };
        cache.store(&checksum, module).unwrap();

        // Load module
        let cached = cache.load(&checksum).unwrap().unwrap();

        // Ensure cached module can be executed
        {
            let mut store = Store::new(engine);
            let instance = WasmerInstance::new(&mut store, &cached.module, &imports! {}).unwrap();
            set_remaining_points(&mut store, &instance, TESTING_GAS_LIMIT);
            let add_one = instance.exports.get_function("add_one").unwrap();
            let result = add_one.call(&mut store, &[42.into()]).unwrap();
            assert_eq!(result[0].unwrap_i32(), 43);
        }
    }

    #[test]
    fn len_works() {
        let mut cache = InMemoryCache::new(Size::mebi(2));

        // Create module
        let wasm1 = wat::parse_str(WAT1).unwrap();
        let checksum1 = Checksum::generate(&wasm1);
        let wasm2 = wat::parse_str(WAT2).unwrap();
        let checksum2 = Checksum::generate(&wasm2);
        let wasm3 = wat::parse_str(WAT3).unwrap();
        let checksum3 = Checksum::generate(&wasm3);

        assert_eq!(cache.len(), 0);

        // Add 1
        let engine1 = make_compiling_engine(TESTING_MEMORY_LIMIT);
        let module = CachedModule {
            module: compile(&engine1, &wasm1).unwrap(),
            engine: make_runtime_engine(TESTING_MEMORY_LIMIT),
            size_estimate: 900_000,
        };
        cache.store(&checksum1, module).unwrap();
        assert_eq!(cache.len(), 1);

        // Add 2
        let engine2 = make_compiling_engine(TESTING_MEMORY_LIMIT);
        let module = CachedModule {
            module: compile(&engine2, &wasm2).unwrap(),
            engine: make_runtime_engine(TESTING_MEMORY_LIMIT),
            size_estimate: 900_000,
        };
        cache.store(&checksum2, module).unwrap();
        assert_eq!(cache.len(), 2);

        // Add 3 (pushes out the previous two)
        let engine3 = make_compiling_engine(TESTING_MEMORY_LIMIT);
        let module = CachedModule {
            module: compile(&engine3, &wasm3).unwrap(),
            engine: make_runtime_engine(TESTING_MEMORY_LIMIT),
            size_estimate: 1_500_000,
        };
        cache.store(&checksum3, module).unwrap();
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn size_works() {
        let mut cache = InMemoryCache::new(Size::mebi(2));

        // Create module
        let wasm1 = wat::parse_str(WAT1).unwrap();
        let checksum1 = Checksum::generate(&wasm1);
        let wasm2 = wat::parse_str(WAT2).unwrap();
        let checksum2 = Checksum::generate(&wasm2);
        let wasm3 = wat::parse_str(WAT3).unwrap();
        let checksum3 = Checksum::generate(&wasm3);

        assert_eq!(cache.size(), 0);

        // Add 1
        let engine1 = make_compiling_engine(TESTING_MEMORY_LIMIT);
        let module = CachedModule {
            module: compile(&engine1, &wasm1).unwrap(),
            engine: make_runtime_engine(TESTING_MEMORY_LIMIT),
            size_estimate: 900_000,
        };
        cache.store(&checksum1, module).unwrap();
        assert_eq!(cache.size(), 900_032);

        // Add 2
        let engine2 = make_compiling_engine(TESTING_MEMORY_LIMIT);
        let module = CachedModule {
            module: compile(&engine2, &wasm2).unwrap(),
            engine: make_runtime_engine(TESTING_MEMORY_LIMIT),
            size_estimate: 800_000,
        };
        cache.store(&checksum2, module).unwrap();
        assert_eq!(cache.size(), 900_032 + 800_032);

        // Add 3 (pushes out the previous two)
        let engine3 = make_compiling_engine(TESTING_MEMORY_LIMIT);
        let module = CachedModule {
            module: compile(&engine3, &wasm3).unwrap(),
            engine: make_runtime_engine(TESTING_MEMORY_LIMIT),
            size_estimate: 1_500_000,
        };
        cache.store(&checksum3, module).unwrap();
        assert_eq!(cache.size(), 1_500_032);
    }

    #[test]
    fn in_memory_cache_works_for_zero_size() {
        // A cache size of 0 practically disabled the cache. It must work
        // like any cache with insufficient space.
        // We test all common methods here.

        let mut cache = InMemoryCache::new(Size::mebi(0));

        // Create module
        let wasm = wat::parse_str(WAT1).unwrap();
        let checksum = Checksum::generate(&wasm);

        // Module does not exist
        let cache_entry = cache.load(&checksum).unwrap();
        assert!(cache_entry.is_none());
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.size(), 0);

        // Compile module
        let engine = make_compiling_engine(TESTING_MEMORY_LIMIT);
        let original = compile(&engine, &wasm).unwrap();

        // Store module
        let module = CachedModule {
            module: original,
            engine: make_runtime_engine(TESTING_MEMORY_LIMIT),
            size_estimate: wasm.len() * TESTING_WASM_SIZE_FACTOR,
        };
        cache.store(&checksum, module).unwrap();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.size(), 0);

        // Load module
        let cached = cache.load(&checksum).unwrap();
        assert!(cached.is_none());
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.size(), 0);
    }
}
