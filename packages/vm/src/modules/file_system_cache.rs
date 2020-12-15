use std::fs;
use std::io;
use std::path::PathBuf;

use wasmer::{DeserializeError, Module, Store};

use crate::checksum::Checksum;
use crate::errors::{VmError, VmResult};

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

    /// Loads an artifact from the file system and returns a module (i.e. artifact + store).
    pub fn load(&self, checksum: &Checksum, store: &Store) -> VmResult<Option<Module>> {
        let filename = checksum.to_hex();
        let file_path = self
            .path
            .clone()
            .join(MODULE_SERIALIZATION_VERSION)
            .join(filename);

        let result = unsafe { Module::deserialize_from_file(store, &file_path) };
        match result {
            Ok(module) => Ok(Some(module)),
            Err(DeserializeError::Io(err)) => match err.kind() {
                io::ErrorKind::NotFound => Ok(None),
                _ => Err(VmError::cache_err(format!(
                    "Error opening module file: {}",
                    err
                ))),
            },
            Err(err) => Err(VmError::cache_err(format!(
                "Error deserializing module: {}",
                err
            ))),
        }
    }

    pub fn store(&mut self, checksum: &Checksum, module: &Module) -> VmResult<()> {
        let modules_dir = self.path.clone().join(MODULE_SERIALIZATION_VERSION);
        fs::create_dir_all(&modules_dir)
            .map_err(|e| VmError::cache_err(format!("Error creating direcory: {}", e)))?;
        let filename = checksum.to_hex();
        let path = modules_dir.join(filename);
        module
            .serialize_to_file(path)
            .map_err(|e| VmError::cache_err(format!("Error writing module to disk: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::size::Size;
    use crate::wasm_backend::{compile_only, make_runtime_store, set_gas_left_to_wasmer_instance};
    use tempfile::TempDir;
    use wasmer::{imports, Instance as WasmerInstance};

    const TESTING_MEMORY_LIMIT: Size = Size::mebi(16);
    const TESTING_GAS_LIMIT: u64 = 5_000;

    #[test]
    fn test_file_system_cache_run() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { FileSystemCache::new(tmp_dir.path()).unwrap() };

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

        // Module does not exist
        let store = make_runtime_store(TESTING_MEMORY_LIMIT);
        let cached = cache.load(&checksum, &store).unwrap();
        assert!(cached.is_none());

        // Store module
        let module = compile_only(&wasm).unwrap();
        cache.store(&checksum, &module).unwrap();

        // Load module
        let store = make_runtime_store(TESTING_MEMORY_LIMIT);
        let cached = cache.load(&checksum, &store).unwrap();
        assert!(cached.is_some());

        // Check the returned module is functional.
        // This is not really testing the cache API but better safe than sorry.
        {
            let cached_module = cached.unwrap();
            let import_object = imports! {};
            let instance = WasmerInstance::new(&cached_module, &import_object).unwrap();
            set_gas_left_to_wasmer_instance(&instance, TESTING_GAS_LIMIT);
            let add_one = instance.exports.get_function("add_one").unwrap();
            let result = add_one.call(&[42.into()]).unwrap();
            assert_eq!(result[0].unwrap_i32(), 43);
        }
    }
}
