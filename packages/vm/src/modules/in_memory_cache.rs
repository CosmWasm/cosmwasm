use clru::CLruCache;
use wasmer::Module;

use crate::{Checksum, Size, VmError, VmResult};

// Minimum module size.
// Based on `examples/module_size.sh`, and the cosmwasm-plus modules.
// We use an estimated *minimum* module size in order to compute a cache capacity
// big enough to handle a size-limited cache without hitting the capacity (number of entries) limit.
// This will incurr an extra memory cost for the unused entries, but it's negligible:
// Assuming the cost per entry is 48 bytes, 10000 entries will have an extra cost of just ~500 kB.
// Which is a very small percentage (~0.03%) of our typical cache memory budget (2 GB).
const MINIMUM_MODULE_SIZE: Size = Size::kibi(250);

/// An in-memory module cache
pub struct InMemoryCache {
    modules: CLruCache<Checksum, Module>,
}

impl InMemoryCache {
    /// Creates a new cache with the given size (in bytes)
    /// and estimated number of entries
    pub fn new(size: Size) -> Self {
        let max_entries = size.0 / MINIMUM_MODULE_SIZE.0;
        InMemoryCache {
            modules: CLruCache::with_weight(max_entries, size.0),
        }
    }

    pub fn store(&mut self, checksum: &Checksum, module: Module, size: usize) -> VmResult<()> {
        self.modules
            .put_with_weight(*checksum, module, size)
            .map_err(|e| VmError::cache_err(format!("{:?}", e)))?;
        Ok(())
    }

    /// Looks up a module in the cache and creates a new module
    pub fn load(&mut self, checksum: &Checksum) -> VmResult<Option<Module>> {
        match self.modules.get(checksum) {
            Some(module) => Ok(Some(module.clone())),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::size::Size;
    use crate::wasm_backend::compile_only;
    use wasmer::{imports, Instance as WasmerInstance};
    use wasmer_middlewares::metering::set_remaining_points;

    const TESTING_GAS_LIMIT: u64 = 5_000;
    // Based on `examples/module_size.sh`
    const TESTING_WASM_SIZE_FACTOR: usize = 18;

    #[test]
    fn in_memory_cache_run() {
        let mut cache = InMemoryCache::new(Size::mebi(200));

        // Create module
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
        let checksum = Checksum::generate(&wasm);

        // Module does not exist
        let cache_entry = cache.load(&checksum).unwrap();
        assert!(cache_entry.is_none());

        // Compile module
        let original = compile_only(&wasm).unwrap();

        // Ensure original module can be executed
        {
            let instance = WasmerInstance::new(&original, &imports! {}).unwrap();
            set_remaining_points(&instance, TESTING_GAS_LIMIT);
            let add_one = instance.exports.get_function("add_one").unwrap();
            let result = add_one.call(&[42.into()]).unwrap();
            assert_eq!(result[0].unwrap_i32(), 43);
        }

        // Store module
        let size = wasm.len() * TESTING_WASM_SIZE_FACTOR;
        cache.store(&checksum, original, size).unwrap();

        // Load module
        let cached = cache.load(&checksum).unwrap().unwrap();

        // Ensure cached module can be executed
        {
            let instance = WasmerInstance::new(&cached, &imports! {}).unwrap();
            set_remaining_points(&instance, TESTING_GAS_LIMIT);
            let add_one = instance.exports.get_function("add_one").unwrap();
            let result = add_one.call(&[42.into()]).unwrap();
            assert_eq!(result[0].unwrap_i32(), 43);
        }
    }
}
