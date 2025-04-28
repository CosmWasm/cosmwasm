use blake2::{digest::consts::U5, Blake2b, Digest};
use std::fs;
use std::hash::Hash;
use std::io;
use std::panic::catch_unwind;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::OnceLock;
use thiserror::Error;

use wasmer::{sys::Target, DeserializeError, Module};

use cosmwasm_std::Checksum;

use crate::errors::{VmError, VmResult};
use crate::filesystem::mkdir_p;
use crate::modules::current_wasmer_module_version;
use crate::wasm_backend::make_runtime_engine;
use crate::wasm_backend::COST_FUNCTION_HASH;
use crate::Size;

use super::cached_module::engine_size_estimate;
use super::CachedModule;

/// This is a value you can manually modify to the cache.
/// You normally _do not_ need to change this value yourself.
///
/// Cases where you might need to update it yourself, is things like when the memory layout of some types in Rust [std] changes.
///
/// ---
///
/// Now follows the legacy documentation of this value:
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
/// - **v4**:<br>
///   Version for Wasmer 2.3.0 which contains a module breaking change to 2.2.0 that was not reflected in
///   the module header version (<https://github.com/wasmerio/wasmer/issues/3193>). In cosmwasm-vm 1.1.0-1.1.1
///   the old value "v3" is still used along with Wasmer 2.3.0 (bug). From cosmwasm 1.1.2 onwards, this is
///   fixed by bumping to "v4".
/// - **v5**:<br>
///   A change in memory layout of some types in Rust [std] caused
///   [issues with module deserialization](https://github.com/CosmWasm/wasmvm/issues/426).
///   To work around this, the version was bumped to "v5" here to invalidate these corrupt caches.
/// - **v6**:<br>
///   Version for cosmwasm_vm 1.3+ which adds a sub-folder with the target identifier for the modules.
/// - **v7**:<br>
///   New version because of Wasmer 2.3.0 -> 4 upgrade.
///   This internally changes how rkyv is used for module serialization, making compatibility unlikely.
/// - **v8**:<br>
///   New version because of Wasmer 4.1.2 -> 4.2.2 upgrade.
///   Module compatibility between Wasmer versions is not guaranteed.
/// - **v9**:<br>
///   New version because of Wasmer 4.2.2 -> 4.2.6 upgrade.
///   Module compatibility between Wasmer versions is not guaranteed.
/// - **v10**:<br>
///   New version because of Metering middleware change.
/// - **v20**:<br>
///   New version because of Wasmer 4.3.3 -> 4.3.7 upgrade.
///   Module compatibility between Wasmer versions is not guaranteed.
const MODULE_SERIALIZATION_VERSION: &str = "v20";

/// Function that actually does the heavy lifting of creating the module version discriminator.
///
/// Separated for sanity tests because otherwise the `OnceLock` would cache the result.
#[inline]
fn raw_module_version_discriminator() -> String {
    let hashes = [COST_FUNCTION_HASH];

    let mut hasher = Blake2b::<U5>::new();

    hasher.update(MODULE_SERIALIZATION_VERSION.as_bytes());
    hasher.update(wasmer_types::VERSION.as_bytes());

    for hash in hashes {
        hasher.update(hash);
    }

    hex::encode(hasher.finalize())
}

/// This version __MUST__ change whenever the module system changes in a way
/// that old stored modules would be corrupt when loaded in the new system.
/// This needs to be done e.g. when switching between the jit/native engine.
///
/// By default, this derived by performing the following operation:
///
/// ```ignore
/// BLAKE2(
///   manual module version,
///   wasmer version requirement,
///   BLAKE2_512(cost_fn)
/// )
/// ```
///
/// If anything else changes, you must change the manual module version.
///
/// See https://github.com/wasmerio/wasmer/issues/2781 for more information
/// on Wasmer's module stability concept.
#[inline]
fn module_version_discriminator() -> &'static str {
    static DISCRIMINATOR: OnceLock<String> = OnceLock::new();

    DISCRIMINATOR.get_or_init(raw_module_version_discriminator)
}

/// Representation of a directory that contains compiled Wasm artifacts.
pub struct FileSystemCache {
    modules_path: PathBuf,
    /// If true, the cache uses the `*_unchecked` wasmer functions for loading modules from disk.
    unchecked_modules: bool,
}

/// An error type that hides system specific error information
/// to ensure deterministic errors across operating systems.
#[derive(Error, Debug)]
pub enum NewFileSystemCacheError {
    #[error("Could not get metadata of cache path")]
    CouldntGetMetadata,
    #[error("The supplied path is readonly")]
    ReadonlyPath,
    #[error("The supplied path already exists but is no directory")]
    ExistsButNoDirectory,
    #[error("Could not create cache path")]
    CouldntCreatePath,
}

impl FileSystemCache {
    /// Construct a new `FileSystemCache` around the specified directory.
    /// The contents of the cache are stored in sub-versioned directories.
    /// If `unchecked_modules` is set to true, it uses the `*_unchecked`
    /// wasmer functions for loading modules from disk (no validity checks).
    ///
    /// # Safety
    ///
    /// This method is unsafe because there's no way to ensure the artifacts
    /// stored in this cache haven't been corrupted or tampered with.
    pub unsafe fn new(
        base_path: impl Into<PathBuf>,
        unchecked_modules: bool,
    ) -> Result<Self, NewFileSystemCacheError> {
        let base_path: PathBuf = base_path.into();
        if base_path.exists() {
            let metadata = base_path
                .metadata()
                .map_err(|_e| NewFileSystemCacheError::CouldntGetMetadata)?;
            if !metadata.is_dir() {
                return Err(NewFileSystemCacheError::ExistsButNoDirectory);
            }
            if metadata.permissions().readonly() {
                return Err(NewFileSystemCacheError::ReadonlyPath);
            }
        } else {
            // Create the directory and any parent directories if they don't yet exist.
            mkdir_p(&base_path).map_err(|_e| NewFileSystemCacheError::CouldntCreatePath)?;
        }

        Ok(Self {
            modules_path: modules_path(
                &base_path,
                current_wasmer_module_version(),
                &Target::default(),
            ),
            unchecked_modules,
        })
    }

    /// If `unchecked` is true, the cache will use the `*_unchecked` wasmer functions for
    /// loading modules from disk.
    pub fn set_module_unchecked(&mut self, unchecked: bool) {
        self.unchecked_modules = unchecked;
    }

    /// Returns the path to the serialized module with the given checksum.
    fn module_file(&self, checksum: &Checksum) -> PathBuf {
        let mut path = self.modules_path.clone();
        path.push(checksum.to_hex());
        path.set_extension("module");
        path
    }

    /// Loads a serialized module from the file system and returns a Module + Engine,
    /// along with a size estimation for the pair.
    pub fn load(
        &self,
        checksum: &Checksum,
        memory_limit: Option<Size>,
    ) -> VmResult<Option<CachedModule>> {
        let file_path = self.module_file(checksum);

        let engine = make_runtime_engine(memory_limit);
        let result = if self.unchecked_modules {
            unsafe { Module::deserialize_from_file_unchecked(&engine, &file_path) }
        } else {
            unsafe { Module::deserialize_from_file(&engine, &file_path) }
        };
        match result {
            Ok(module) => {
                let module_size = module_size(&file_path)?;
                Ok(Some(CachedModule {
                    module,
                    engine,
                    size_estimate: module_size + engine_size_estimate(),
                }))
            }
            Err(DeserializeError::Io(err)) => match err.kind() {
                io::ErrorKind::NotFound => Ok(None),
                _ => Err(VmError::cache_err(format!(
                    "Error opening module file: {err}"
                ))),
            },
            Err(err) => Err(VmError::cache_err(format!(
                "Error deserializing module: {err}"
            ))),
        }
    }

    /// Stores a serialized module to the file system. Returns the size of the serialized module.
    pub fn store(&mut self, checksum: &Checksum, module: &Module) -> VmResult<usize> {
        mkdir_p(&self.modules_path)
            .map_err(|_e| VmError::cache_err("Error creating modules directory"))?;

        let path = self.module_file(checksum);
        catch_unwind(|| {
            module
                .serialize_to_file(&path)
                .map_err(|e| VmError::cache_err(format!("Error writing module to disk: {e}")))
        })
        .map_err(|_| VmError::cache_err("Could not write module to disk"))??;
        let module_size = module_size(&path)?;
        Ok(module_size)
    }

    /// Removes a serialized module from the file system.
    ///
    /// Returns true if the file existed and false if the file did not exist.
    pub fn remove(&mut self, checksum: &Checksum) -> VmResult<bool> {
        let file_path = self.module_file(checksum);

        if file_path.exists() {
            fs::remove_file(file_path)
                .map_err(|_e| VmError::cache_err("Error deleting module from disk"))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Returns the size of the module stored on disk
fn module_size(module_path: &Path) -> VmResult<usize> {
    let module_size: usize = module_path
        .metadata()
        .map_err(|_e| VmError::cache_err("Error getting file metadata"))? // ensure error message is not system specific
        .len()
        .try_into()
        .expect("Could not convert file size to usize");
    Ok(module_size)
}

/// Creates an identifier for the Wasmer `Target` that is used for
/// cache invalidation. The output is reasonable human friendly to be usable
/// in file path component.
fn target_id(target: &Target) -> String {
    // Use a custom Hasher implementation to avoid randomization.
    let mut deterministic_hasher = crc32fast::Hasher::new();
    target.hash(&mut deterministic_hasher);
    let hash = deterministic_hasher.finalize();
    format!("{}-{:08X}", target.triple(), hash) // print 4 byte hash as 8 hex characters
}

/// The path to the latest version of the modules.
fn modules_path(base_path: &Path, wasmer_module_version: u32, target: &Target) -> PathBuf {
    let version_dir = format!(
        "{}-wasmer{wasmer_module_version}",
        module_version_discriminator()
    );
    let target_dir = target_id(target);
    base_path.join(version_dir).join(target_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wasm_backend::{compile, make_compiling_engine};
    use tempfile::TempDir;
    use wasmer::{imports, Instance as WasmerInstance, Store};
    use wasmer_middlewares::metering::set_remaining_points;

    const TESTING_MEMORY_LIMIT: Option<Size> = Some(Size::mebi(16));
    const TESTING_GAS_LIMIT: u64 = 500_000;

    const SOME_WAT: &str = r#"(module
        (type $t0 (func (param i32) (result i32)))
        (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
            local.get $p0
            i32.const 1
            i32.add))
    "#;

    #[test]
    fn file_system_cache_run() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { FileSystemCache::new(tmp_dir.path(), false).unwrap() };

        // Create module
        let wasm = wat::parse_str(SOME_WAT).unwrap();
        let checksum = Checksum::generate(&wasm);

        // Module does not exist
        let cached = cache.load(&checksum, TESTING_MEMORY_LIMIT).unwrap();
        assert!(cached.is_none());

        // Store module
        let compiling_engine = make_compiling_engine(TESTING_MEMORY_LIMIT);
        let module = compile(&compiling_engine, &wasm).unwrap();
        cache.store(&checksum, &module).unwrap();

        // Load module
        let cached = cache.load(&checksum, TESTING_MEMORY_LIMIT).unwrap();
        assert!(cached.is_some());

        // Check the returned module is functional.
        // This is not really testing the cache API but better safe than sorry.
        {
            let CachedModule {
                module: cached_module,
                engine: runtime_engine,
                size_estimate,
            } = cached.unwrap();
            assert_eq!(
                size_estimate,
                module.serialize().unwrap().len() + 10240 /* engine size estimate */
            );
            let import_object = imports! {};
            let mut store = Store::new(runtime_engine);
            let instance = WasmerInstance::new(&mut store, &cached_module, &import_object).unwrap();
            set_remaining_points(&mut store, &instance, TESTING_GAS_LIMIT);
            let add_one = instance.exports.get_function("add_one").unwrap();
            let result = add_one.call(&mut store, &[42.into()]).unwrap();
            assert_eq!(result[0].unwrap_i32(), 43);
        }
    }

    #[test]
    fn file_system_cache_store_uses_expected_path() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { FileSystemCache::new(tmp_dir.path(), false).unwrap() };

        // Create module
        let wasm = wat::parse_str(SOME_WAT).unwrap();
        let checksum = Checksum::generate(&wasm);

        // Store module
        let engine = make_compiling_engine(TESTING_MEMORY_LIMIT);
        let module = compile(&engine, &wasm).unwrap();
        cache.store(&checksum, &module).unwrap();

        let discriminator = raw_module_version_discriminator();
        let module_version = current_wasmer_module_version();
        let mut globber = glob::glob(&format!(
            "{}/{}-wasmer{}/**/{}.module",
            tmp_dir.path().to_string_lossy(),
            discriminator,
            module_version,
            checksum
        ))
        .expect("Failed to read glob pattern");
        let file_path = globber.next().unwrap().unwrap();
        let _serialized_module = fs::read(file_path).unwrap();
    }

    #[test]
    fn file_system_cache_remove_works() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { FileSystemCache::new(tmp_dir.path(), false).unwrap() };

        // Create module
        let wasm = wat::parse_str(SOME_WAT).unwrap();
        let checksum = Checksum::generate(&wasm);

        // Store module
        let compiling_engine = make_compiling_engine(TESTING_MEMORY_LIMIT);
        let module = compile(&compiling_engine, &wasm).unwrap();
        cache.store(&checksum, &module).unwrap();

        // It's there
        assert!(cache
            .load(&checksum, TESTING_MEMORY_LIMIT)
            .unwrap()
            .is_some());

        // Remove module
        let existed = cache.remove(&checksum).unwrap();
        assert!(existed);

        // it's gone now
        assert!(cache
            .load(&checksum, TESTING_MEMORY_LIMIT)
            .unwrap()
            .is_none());

        // Remove again
        let existed = cache.remove(&checksum).unwrap();
        assert!(!existed);
    }

    #[test]
    fn target_id_works() {
        // Test with default target
        let default_target = Target::default();
        let default_id = target_id(&default_target);
        assert!(!default_id.is_empty());
        assert!(default_id.contains(&default_target.triple().to_string()));

        // Test with different CPU features on same architecture
        let with_feature_target = wasmer::sys::Target::new(
            default_target.triple().clone(),
            wasmer::sys::CpuFeature::AVX512DQ.into(),
        );
        let with_feature_id = target_id(&with_feature_target);
        assert_ne!(default_id, with_feature_id);

        // Verify determinism
        let id1 = target_id(&default_target);
        let id2 = target_id(&default_target);
        assert_eq!(id1, id2);
    }

    #[test]
    fn target_id_includes_triple_elements() {
        let target = Target::default();
        let id = target_id(&target);

        // Basic verification that target ID contains arch
        let arch_str = format!("{}", target.triple().architecture);
        assert!(id.contains(&arch_str.to_lowercase()));
    }

    #[test]
    fn modules_path_works() {
        let base = PathBuf::from("modules");
        let target = Target::default();
        let p = modules_path(&base, 17, &target);
        let discriminator = raw_module_version_discriminator();

        // Test the basic structure but not exact values
        assert!(p
            .to_string_lossy()
            .contains(&format!("{discriminator}-wasmer17")));
        assert!(p.to_string_lossy().contains(&target.triple().to_string()));
    }

    #[test]
    fn module_version_discriminator_stays_the_same() {
        let v1 = raw_module_version_discriminator();
        let v2 = raw_module_version_discriminator();
        let v3 = raw_module_version_discriminator();
        let v4 = raw_module_version_discriminator();

        assert_eq!(v1, v2);
        assert_eq!(v2, v3);
        assert_eq!(v3, v4);
    }

    #[test]
    fn module_version_static() {
        let version = raw_module_version_discriminator();
        assert_eq!(version, "78e713cf6c");
    }
}
