// copied from https://github.com/wasmerio/wasmer/blob/0.8.0/lib/runtime/src/cache.rs
// with some minor modifications

use memmap::Mmap;
use std::{
    fs::{create_dir_all, File},
    io::{self, Write},
    path::PathBuf,
};

pub use wasmer_runtime_core::{
    backend::{Backend, Compiler},
    cache::{Artifact, Cache, WasmHash},
};
use wasmer_runtime_core::{cache::Error as CacheError, module::Module};

use crate::backends::{backend, compiler_for_backend};

/// Representation of a directory that contains compiled wasm artifacts.
///
/// The `FileSystemCache` type implements the [`Cache`] trait, which allows it to be used
/// generically when some sort of cache is required.
///
/// [`Cache`]: trait.Cache.html
///
/// # Usage:
///
/// ```rust
/// use cosmwasm_vm::FileSystemCache;
/// use wasmer_runtime_core::cache::{Cache, Error as CacheError, WasmHash};
/// use wasmer_runtime_core::module::Module;
///
/// fn store_module(module: Module) -> Result<Module, CacheError> {
///     // Create a new file system cache.
///     // This is unsafe because we can't ensure that the artifact wasn't
///     // corrupted or tampered with.
///     let mut fs_cache = unsafe { FileSystemCache::new("some/directory/goes/here")? };
///     // Compute a key for a given WebAssembly binary
///     let key = WasmHash::generate(&[]);
///     // Store a module into the cache given a key
///     fs_cache.store(key, module.clone())?;
///     Ok(module)
/// }
/// ```
pub struct FileSystemCache {
    path: PathBuf,
}

impl FileSystemCache {
    /// Construct a new `FileSystemCache` around the specified directory.
    /// The contents of the cache are stored in sub-versioned directories.
    ///
    /// # Safety
    ///
    /// This method is unsafe because there's no way to ensure the artifacts
    /// stored in this cache haven't been corrupted or tampered with.
    pub unsafe fn new<P: Into<PathBuf>>(path: P) -> io::Result<Self> {
        let path: PathBuf = path.into();
        if path.exists() {
            let metadata = path.metadata()?;
            if metadata.is_dir() {
                if !metadata.permissions().readonly() {
                    Ok(Self { path })
                } else {
                    // This directory is readonly.
                    Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        format!("the supplied path is readonly: {}", path.display()),
                    ))
                }
            } else {
                // This path points to a file.
                Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!(
                        "the supplied path already points to a file: {}",
                        path.display()
                    ),
                ))
            }
        } else {
            // Create the directory and any parent directories if they don't yet exist.
            create_dir_all(&path)?;
            Ok(Self { path })
        }
    }
}

impl Cache for FileSystemCache {
    type LoadError = CacheError;
    type StoreError = CacheError;

    fn load(&self, key: WasmHash) -> Result<Module, CacheError> {
        self.load_with_backend(key, backend())
    }

    fn load_with_backend(&self, key: WasmHash, backend: Backend) -> Result<Module, CacheError> {
        let filename = key.encode();
        let mut new_path_buf = self.path.clone();
        new_path_buf.push(backend.to_string());
        new_path_buf.push(filename);
        let file = File::open(new_path_buf)?;
        let mmap = unsafe { Mmap::map(&file)? };

        let serialized_cache = Artifact::deserialize(&mmap[..])?;
        unsafe {
            wasmer_runtime_core::load_cache_with(
                serialized_cache,
                compiler_for_backend(backend)
                    .ok_or_else(|| CacheError::UnsupportedBackend(backend))?
                    .as_ref(),
            )
        }
    }

    fn store(&mut self, key: WasmHash, module: Module) -> Result<(), CacheError> {
        let filename = key.encode();
        let backend_str = module.info().backend.to_string();
        let mut new_path_buf = self.path.clone();
        new_path_buf.push(backend_str);

        let serialized_cache = module.cache()?;
        let buffer = serialized_cache.serialize()?;

        std::fs::create_dir_all(&new_path_buf)?;
        new_path_buf.push(filename);
        let mut file = File::create(new_path_buf)?;
        file.write_all(&buffer)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::backends::compile;
    use std::env;

    #[test]
    fn test_file_system_cache_run() {
        use wabt::wat2wasm;
        use wasmer_runtime_core::{imports, typed_func::Func};

        static WAT: &'static str = r#"
            (module
              (type $t0 (func (param i32) (result i32)))
              (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
                get_local $p0
                i32.const 1
                i32.add))
        "#;

        let wasm = wat2wasm(WAT).unwrap();

        let module = compile(&wasm).unwrap();

        // assert we are using the proper backend
        assert_eq!(backend().to_string(), module.info().backend.to_string());

        let cache_dir = env::temp_dir();

        let mut fs_cache = unsafe {
            FileSystemCache::new(cache_dir)
                .map_err(|e| format!("Cache error: {:?}", e))
                .unwrap()
        };
        // store module
        let key = WasmHash::generate(&wasm);
        fs_cache.store(key, module.clone()).unwrap();

        // load module
        let cached_result = fs_cache.load(key);

        let cached_module = cached_result.unwrap();
        let import_object = imports! {};
        let instance = cached_module.instantiate(&import_object).unwrap();
        let add_one: Func<i32, i32> = instance.func("add_one").unwrap();

        let value = add_one.call(42).unwrap();

        // verify it works
        assert_eq!(value, 43);
    }
}
