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
///
/// See https://github.com/wasmerio/wasmer/issues/2781 for more information
/// on Wasmer's module stability concept.
///
/// ## Version history:
/// - **v1**:<br>
///   cosmwasm_vm < 1.0.0-beta5. This is working well up to Wasmer 2.0.0 as
///   [in wasmvm 1.0.0-beta2](https://github.com/CosmWasm/wasmvm/blob/v1.0.0-beta2/libwasmvm/Cargo.lock#L1412-L1413)
///   and [wasmvm 0.16.3](https://github.com/CosmWasm/wasmvm/blob/v0.16.3/libwasmvm/Cargo.lock#L1408-L1409).
///   Versions that ship with Wasmer 2.1.x such [as wasmvm 1.0.0-beta3](https://github.com/CosmWasm/wasmvm/blob/v1.0.0-beta3/libwasmvm/Cargo.lock#L1534-L1535)
///   to [wasmvm 1.0.0-beta5](https://github.com/CosmWasm/wasmvm/blob/v1.0.0-beta5/libwasmvm/Cargo.lock#L1530-L1531)
///   are broken, i.e. they will crash when reading older v1 modules.
/// - **v2**:<br>
///   Version for cosmwasm_vm 1.0.0-beta5 / wasmvm 1.0.0-beta6 that ships with Wasmer 2.1.1.
/// - **v3**:<br>
///   Version for Wasmer 2.2.0 which contains a [module breaking change to 2.1.x](https://github.com/wasmerio/wasmer/pull/2747).
const MODULE_SERIALIZATION_VERSION: &str = "v2";

/// Representation of a directory that contains compiled Wasm artifacts.
pub struct FileSystemCache {
    /// The base path this cache operates in. Within this path, versioned directories are created.
    /// A sophisticated version of this cache might be able to read multiple input versions in the future.
    base_path: PathBuf,
}

impl FileSystemCache {
    /// Construct a new `FileSystemCache` around the specified directory.
    /// The contents of the cache are stored in sub-versioned directories.
    ///
    /// # Safety
    ///
    /// This method is unsafe because there's no way to ensure the artifacts
    /// stored in this cache haven't been corrupted or tampered with.
    pub unsafe fn new(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path: PathBuf = path.into();
        if path.exists() {
            let metadata = path.metadata()?;
            if metadata.is_dir() {
                if !metadata.permissions().readonly() {
                    Ok(Self { base_path: path })
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
            Ok(Self { base_path: path })
        }
    }

    /// Loads a serialized module from the file system and returns a module (i.e. artifact + store),
    /// along with the size of the serialized module.
    pub fn load(&self, checksum: &Checksum, store: &Store) -> VmResult<Option<Module>> {
        let filename = checksum.to_hex();
        let file_path = self.latest_modules_path().join(filename);

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

    /// Stores a serialized module to the file system. Returns the size of the serialized module.
    pub fn store(&mut self, checksum: &Checksum, module: &Module) -> VmResult<()> {
        let modules_dir = self.latest_modules_path();
        fs::create_dir_all(&modules_dir)
            .map_err(|e| VmError::cache_err(format!("Error creating directory: {}", e)))?;
        let filename = checksum.to_hex();
        let path = modules_dir.join(filename);
        module
            .serialize_to_file(path)
            .map_err(|e| VmError::cache_err(format!("Error writing module to disk: {}", e)))?;
        Ok(())
    }

    /// The path to the latest version of the modules.
    fn latest_modules_path(&self) -> PathBuf {
        self.base_path.join(MODULE_SERIALIZATION_VERSION)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::size::Size;
    use crate::wasm_backend::{compile, make_runtime_store};
    use tempfile::TempDir;
    use wasmer::{imports, Instance as WasmerInstance};
    use wasmer_middlewares::metering::set_remaining_points;

    const TESTING_MEMORY_LIMIT: Option<Size> = Some(Size::mebi(16));
    const TESTING_GAS_LIMIT: u64 = 500_000_000;

    const SOME_WAT: &str = r#"(module
        (type $t0 (func (param i32) (result i32)))
        (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
            get_local $p0
            i32.const 1
            i32.add))
    "#;

    #[test]
    fn file_system_cache_run() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { FileSystemCache::new(tmp_dir.path()).unwrap() };

        // Create module
        let wasm = wat::parse_str(SOME_WAT).unwrap();
        let checksum = Checksum::generate(&wasm);

        // Module does not exist
        let store = make_runtime_store(TESTING_MEMORY_LIMIT);
        let cached = cache.load(&checksum, &store).unwrap();
        assert!(cached.is_none());

        // Store module
        let module = compile(&wasm, None, &[]).unwrap();
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
            set_remaining_points(&instance, TESTING_GAS_LIMIT);
            let add_one = instance.exports.get_function("add_one").unwrap();
            let result = add_one.call(&[42.into()]).unwrap();
            assert_eq!(result[0].unwrap_i32(), 43);
        }
    }

    #[test]
    fn file_system_cache_store_uses_expected_path() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { FileSystemCache::new(tmp_dir.path()).unwrap() };

        // Create module
        let wasm = wat::parse_str(SOME_WAT).unwrap();
        let checksum = Checksum::generate(&wasm);

        // Store module
        let module = compile(&wasm, None, &[]).unwrap();
        cache.store(&checksum, &module).unwrap();

        let file_path = format!("{}/v2/{}", tmp_dir.path().to_string_lossy(), checksum);
        let serialized_module = fs::read(file_path).unwrap();
        assert_eq!(serialized_module.len(), 1040);
    }
}
