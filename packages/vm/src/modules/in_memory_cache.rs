use clru::CLruCache;
use wasmer::Module;

use crate::wasm_backend::make_store_headless;
use crate::{Checksum, Size, VmResult};

const ESTIMATED_MODULE_SIZE: Size = Size::mebi(10);

/// An in-memory module cache
pub struct InMemoryCache {
    artifacts: CLruCache<Checksum, Vec<u8>>,
}

impl InMemoryCache {
    /// Creates a new cache with the given size (in bytes)
    pub fn new(size: Size) -> Self {
        let max_entries = size.0 / ESTIMATED_MODULE_SIZE.0;
        InMemoryCache {
            artifacts: CLruCache::new(max_entries),
        }
    }

    pub fn store(&mut self, checksum: &Checksum, module: Module) -> VmResult<()> {
        let serialized_artifact = module.serialize()?;
        self.artifacts.put(*checksum, serialized_artifact);
        Ok(())
    }

    /// Looks up a module in the cache and takes its artifact, creates a new store and
    /// creates a new module from store and artifact.
    pub fn load(&mut self, checksum: &Checksum, memory_limit: Size) -> VmResult<Option<Module>> {
        match self.artifacts.get(checksum) {
            Some(serialized_artifact) => {
                // Swap out store. Looks complicated because a lot of artifact
                // APIs are private.
                let store = make_store_headless(Some(memory_limit));
                let new_module = unsafe { Module::deserialize(&store, &serialized_artifact) }?;
                Ok(Some(new_module))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wasm_backend::compile;
    use wasmer::{imports, Instance as WasmerInstance};

    const TESTING_MEMORY_LIMIT: Size = Size::mebi(16);

    #[test]
    fn test_in_memory_cache_run() {
        let mut cache = InMemoryCache::new(Size::mebi(200));

        // Create module
        let wasm = wat::parse_str(
            r#"(module
            (type $t0 (func (param i32) (result i32)))
            (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
                get_local $p0
                i32.const 1
                i32.add))
            "#,
        )
        .unwrap();
        let checksum = Checksum::generate(&wasm);
        let module = compile(&wasm, Some(TESTING_MEMORY_LIMIT)).unwrap();

        // Module does not exist
        let cached = cache.load(&checksum, TESTING_MEMORY_LIMIT).unwrap();
        assert!(cached.is_none());

        // Store module
        cache.store(&checksum, module.clone()).unwrap();

        // Load module
        let cached = cache.load(&checksum, TESTING_MEMORY_LIMIT).unwrap();
        assert!(cached.is_some());

        // Check the returned module is functional.
        // This is not really testing the cache API but better safe than sorry.
        {
            let cached_module = cached.unwrap();
            let import_object = imports! {};
            let instance = WasmerInstance::new(&cached_module, &import_object).unwrap();
            let add_one = instance.exports.get_function("add_one").unwrap();
            let result = add_one.call(&[42.into()]).unwrap();
            assert_eq!(result[0].unwrap_i32(), 43);
        }
    }
}
