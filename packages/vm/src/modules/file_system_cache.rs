// copied from https://github.com/wasmerio/wasmer/blob/0.8.0/lib/runtime/src/cache.rs
// with some minor modifications

use memmap::Mmap;
use std::{
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
};

use wasmer_runtime_core::{cache::Artifact, module::Module};

use crate::backends::{compiler_for_backend, BACKEND_NAME};
use crate::checksum::Checksum;
use crate::errors::{VmError, VmResult};

/// Representation of a directory that contains compiled Wasm artifacts.
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
            fs::create_dir_all(&path)?;
            Ok(Self { path })
        }
    }

    pub fn load(&self, checksum: &Checksum) -> VmResult<Module> {
        let backend = BACKEND_NAME;

        let filename = checksum.to_hex();
        let file_path = self.path.clone().join(backend).join(filename);
        let file = File::open(file_path)
            .map_err(|e| VmError::cache_err(format!("Error opening module file: {}", e)))?;
        let mmap = unsafe { Mmap::map(&file) }
            .map_err(|e| VmError::cache_err(format!("Mmap error: {}", e)))?;

        let serialized_cache = Artifact::deserialize(&mmap[..])?;
        let module = unsafe {
            wasmer_runtime_core::load_cache_with(
                serialized_cache,
                compiler_for_backend(backend)
                    .ok_or_else(|| VmError::cache_err(format!("Unsupported backend: {}", backend)))?
                    .as_ref(),
            )
        }?;
        Ok(module)
    }

    pub fn store(&mut self, checksum: &Checksum, module: Module) -> VmResult<()> {
        let backend_str = module.info().backend.to_string();
        let modules_dir = self.path.clone().join(backend_str);
        fs::create_dir_all(&modules_dir)
            .map_err(|e| VmError::cache_err(format!("Error creating direcory: {}", e)))?;

        let serialized_cache = module.cache()?;
        let buffer = serialized_cache.serialize()?;

        let filename = checksum.to_hex();
        let mut file = File::create(modules_dir.join(filename))
            .map_err(|e| VmError::cache_err(format!("Error creating module file: {}", e)))?;
        file.write_all(&buffer)
            .map_err(|e| VmError::cache_err(format!("Error writing module to disk: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::compile;
    use std::env;
    use wabt::wat2wasm;

    #[test]
    fn test_file_system_cache_run() {
        use wasmer_runtime_core::{imports, typed_func::Func};

        let wasm = wat2wasm(
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

        // assert we are using the proper backend
        assert_eq!(BACKEND_NAME.to_string(), module.info().backend.to_string());

        let cache_dir = env::temp_dir();
        let mut fs_cache = unsafe { FileSystemCache::new(cache_dir).unwrap() };

        // store module
        fs_cache.store(&checksum, module.clone()).unwrap();

        // load module
        let cached_result = fs_cache.load(&checksum);

        let cached_module = cached_result.unwrap();
        let import_object = imports! {};
        let instance = cached_module.instantiate(&import_object).unwrap();
        let add_one: Func<i32, i32> = instance.exports.get("add_one").unwrap();

        let value = add_one.call(42).unwrap();

        // verify it works
        assert_eq!(value, 43);
    }
}
