// copied from https://github.com/wasmerio/wasmer/blob/0.8.0/lib/runtime/src/cache.rs
// with some minor modifications

use memmap::Mmap;
use std::{
    fs::{self, File},
    io::{self, ErrorKind, Write},
    path::PathBuf,
};

use wasmer::Module;

use crate::checksum::Checksum;
use crate::errors::{VmError, VmResult};
use crate::wasm_backend::make_store_headless;

/// Bump this version whenever the module system changes in a way
/// that old stored modules would be corrupt when loaded in the new system.
/// This needs to be done e.g. when switching between the jit/native engine.
///
/// The string is used as a folder and should be named in a way that is
/// easy to interprete for system admins. It should allow easy clearing
/// of old versions.
const MODULE_SERIALIZATION_VERSION: &str = "v1";

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

    pub fn load(&self, checksum: &Checksum, memory_limit: u32) -> VmResult<Option<Module>> {
        let filename = checksum.to_hex();
        let file_path = self
            .path
            .clone()
            .join(MODULE_SERIALIZATION_VERSION)
            .join(filename);

        let file = match File::open(file_path) {
            Ok(file) => file,
            Err(err) => match err.kind() {
                ErrorKind::NotFound => return Ok(None),
                _ => {
                    return Err(VmError::cache_err(format!(
                        "Error opening module file: {}",
                        err
                    )))
                }
            },
        };

        let mmap = unsafe { Mmap::map(&file) }
            .map_err(|e| VmError::cache_err(format!("Mmap error: {}", e)))?;

        let store = make_store_headless(memory_limit);
        let module = unsafe { Module::deserialize(&store, &mmap[..]) }?;
        Ok(Some(module))
    }

    pub fn store(&mut self, checksum: &Checksum, module: Module) -> VmResult<()> {
        let modules_dir = self.path.clone().join(MODULE_SERIALIZATION_VERSION);
        fs::create_dir_all(&modules_dir)
            .map_err(|e| VmError::cache_err(format!("Error creating direcory: {}", e)))?;

        let buffer = module.serialize()?;

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
    use crate::wasm_backend::compile;
    use tempfile::TempDir;
    use wabt::wat2wasm;
    use wasmer::{imports, Instance as WasmerInstance};

    const TESTING_MEMORY_LIMIT: u32 = 256; // 256 pages = 16 MiB

    #[test]
    fn test_file_system_cache_run() {
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

        let module = compile(&wasm, TESTING_MEMORY_LIMIT).unwrap();

        let tmp_dir = TempDir::new().unwrap();
        let mut fs_cache = unsafe { FileSystemCache::new(tmp_dir.path()).unwrap() };

        // Module does not exist
        let cached = fs_cache.load(&checksum, TESTING_MEMORY_LIMIT).unwrap();
        assert!(cached.is_none());

        // Store module
        fs_cache.store(&checksum, module.clone()).unwrap();

        // Load module
        let cached = fs_cache.load(&checksum, TESTING_MEMORY_LIMIT).unwrap();
        assert!(cached.is_some());

        let cached_module = cached.unwrap();
        let import_object = imports! {};
        let instance = WasmerInstance::new(&cached_module, &import_object).unwrap();
        let add_one = instance.exports.get_function("add_one").unwrap();

        let result = add_one.call(&[42.into()]).unwrap();

        // verify it works
        assert_eq!(result[0].unwrap_i32(), 43);
    }
}
