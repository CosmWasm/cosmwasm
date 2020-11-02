use clru::CLruCache;
use wasmer_runtime_core::module::Module;

use crate::{Checksum, Size, VmResult};

const ESTIMATED_MODULE_SIZE: Size = Size::mebi(10);

/// An in-memory module cache
pub struct InMemoryCache {
    lru: CLruCache<Checksum, Module>,
}

impl InMemoryCache {
    /// Creates a new cache with the given size (in bytes)
    pub fn new(size: Size) -> Self {
        let max_entries = size.0 / ESTIMATED_MODULE_SIZE.0;
        InMemoryCache {
            lru: CLruCache::new(max_entries),
        }
    }

    pub fn store(&mut self, checksum: &Checksum, module: Module) -> VmResult<()> {
        self.lru.put(*checksum, module);
        Ok(())
    }

    pub fn load(&mut self, checksum: &Checksum) -> VmResult<Option<&Module>> {
        let optional = self.lru.get(checksum);
        Ok(optional)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::{compile, BACKEND_NAME};

    #[test]
    fn test_in_memory_cache_run() {
        use wasmer_runtime_core::{imports, typed_func::Func};

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
        let module = compile(&wasm).unwrap();

        // Module does not exist
        let cached = cache.load(&checksum).unwrap();
        assert!(cached.is_none());

        // Store module
        cache.store(&checksum, module.clone()).unwrap();

        // Load module
        let cached = cache.load(&checksum).unwrap();
        assert!(cached.is_some());

        // Check the returned module is functional.
        // This is not really testing the cache API but better safe than sorry.
        {
            assert_eq!(module.info().backend.to_string(), BACKEND_NAME.to_string());
            let cached_module = cached.unwrap();
            let import_object = imports! {};
            let instance = cached_module.instantiate(&import_object).unwrap();
            let add_one: Func<i32, i32> = instance.exports.get("add_one").unwrap();
            let value = add_one.call(42).unwrap();
            assert_eq!(value, 43);
        }
    }
}
