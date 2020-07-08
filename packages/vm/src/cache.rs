use std::collections::HashSet;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::path::PathBuf;

use crate::backends::{backend, compile};
use crate::checksum::Checksum;
use crate::compatability::check_wasm;
use crate::errors::{VmError, VmResult};
use crate::instance::Instance;
use crate::modules::FileSystemCache;
use crate::traits::{Api, Extern, Querier, Storage};

static WASM_DIR: &str = "wasm";
static MODULES_DIR: &str = "modules";

#[derive(Debug, Default, Clone)]
struct Stats {
    hits_module: u32,
    misses: u32,
}

pub struct CosmCache<S: Storage + 'static, A: Api + 'static, Q: Querier + 'static> {
    wasm_path: PathBuf,
    supported_features: HashSet<String>,
    modules: FileSystemCache,
    stats: Stats,
    // Those two don't store data but only fix type information
    type_storage: PhantomData<S>,
    type_api: PhantomData<A>,
    type_querier: PhantomData<Q>,
}

impl<S, A, Q> CosmCache<S, A, Q>
where
    S: Storage + 'static,
    A: Api + 'static,
    Q: Querier + 'static,
{
    /// new stores the data for cache under base_dir
    ///
    /// Instance caching is disabled since 0.8.1 and any cache size value will be treated as 0.
    ///
    /// # Safety
    ///
    /// This function is marked unsafe due to `FileSystemCache::new`, which implicitly
    /// assumes the disk contents are correct, and there's no way to ensure the artifacts
    //  stored in the cache haven't been corrupted or tampered with.
    pub unsafe fn new<P: Into<PathBuf>>(
        base_dir: P,
        supported_features: HashSet<String>,
    ) -> VmResult<Self> {
        let base = base_dir.into();
        let wasm_path = base.join(WASM_DIR);
        create_dir_all(&wasm_path)
            .map_err(|e| VmError::cache_err(format!("Error creating Wasm dir for cache: {}", e)))?;

        let modules = FileSystemCache::new(base.join(MODULES_DIR))
            .map_err(|e| VmError::cache_err(format!("Error file system cache: {}", e)))?;
        Ok(CosmCache {
            wasm_path,
            supported_features,
            modules,
            stats: Stats::default(),
            type_storage: PhantomData::<S>,
            type_api: PhantomData::<A>,
            type_querier: PhantomData::<Q>,
        })
    }

    pub fn save_wasm(&mut self, wasm: &[u8]) -> VmResult<Checksum> {
        check_wasm(wasm, &self.supported_features)?;
        let checksum = save_wasm_to_disk(&self.wasm_path, wasm)?;
        let module = compile(wasm)?;
        self.modules.store(&checksum, module)?;
        Ok(checksum)
    }

    /// Retrieves a Wasm blob that was previously stored via save_wasm.
    /// When the cache is instantiated with the same base dir, this finds Wasm files on disc across multiple cache instances (i.e. node restarts).
    /// This function is public to allow a checksum to Wasm lookup in the blockchain.
    ///
    /// If the given ID is not found or the content does not match the hash (=ID), an error is returned.
    pub fn load_wasm(&self, checksum: &Checksum) -> VmResult<Vec<u8>> {
        let code = load_wasm_from_disk(&self.wasm_path, checksum)?;
        // verify hash matches (integrity check)
        if Checksum::generate(&code) != *checksum {
            Err(VmError::integrity_err())
        } else {
            Ok(code)
        }
    }

    /// Returns an Instance tied to a previously saved Wasm.
    /// Depending on availability, this is either generated from a cached instance, a cached module or Wasm code.
    pub fn get_instance(
        &mut self,
        checksum: &Checksum,
        deps: Extern<S, A, Q>,
        gas_limit: u64,
    ) -> VmResult<Instance<S, A, Q>> {
        // try from the module cache
        let res = self.modules.load_with_backend(checksum, backend());
        if let Ok(module) = res {
            self.stats.hits_module += 1;
            return Instance::from_module(&module, deps, gas_limit);
        }

        // fall back to wasm cache (and re-compiling) - this is for backends that don't support serialization
        let wasm = self.load_wasm(checksum)?;
        self.stats.misses += 1;
        Instance::from_code(&wasm, deps, gas_limit)
    }
}

/// save stores the wasm code in the given directory and returns an ID for lookup.
/// It will create the directory if it doesn't exist.
/// Saving the same byte code multiple times is allowed.
fn save_wasm_to_disk<P: Into<PathBuf>>(dir: P, wasm: &[u8]) -> VmResult<Checksum> {
    // calculate filename
    let checksum = Checksum::generate(wasm);
    let filename = checksum.to_hex();
    let filepath = dir.into().join(&filename);

    // write data to file
    // Since the same filename (a collision resistent hash) cannot be generated from two different byte codes
    // (even if a malicious actor tried), it is safe to override.
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(filepath)
        .map_err(|e| VmError::cache_err(format!("Error opening Wasm file for writing: {}", e)))?;
    file.write_all(wasm)
        .map_err(|e| VmError::cache_err(format!("Error writing Wasm file: {}", e)))?;

    Ok(checksum)
}

fn load_wasm_from_disk<P: Into<PathBuf>>(dir: P, checksum: &Checksum) -> VmResult<Vec<u8>> {
    // this requires the directory and file to exist
    let path = dir.into().join(checksum.to_hex());
    let mut file = File::open(path)
        .map_err(|e| VmError::cache_err(format!("Error opening Wasm file for reading: {}", e)))?;

    let mut wasm = Vec::<u8>::new();
    file.read_to_end(&mut wasm)
        .map_err(|e| VmError::cache_err(format!("Error reading Wasm file: {}", e)))?;
    Ok(wasm)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::calls::{call_handle, call_init};
    use crate::errors::VmError;
    use crate::features::features_from_csv;
    use crate::testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage};
    use cosmwasm_std::{coins, Empty};
    use std::fs::OpenOptions;
    use std::io::Write;
    use tempfile::TempDir;
    use wabt::wat2wasm;

    static TESTING_GAS_LIMIT: u64 = 400_000;
    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

    fn default_features() -> HashSet<String> {
        features_from_csv("staking")
    }

    #[test]
    fn save_wasm_works() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache: CosmCache<MockStorage, MockApi, MockQuerier> =
            unsafe { CosmCache::new(tmp_dir.path(), default_features()).unwrap() };
        cache.save_wasm(CONTRACT).unwrap();
    }

    #[test]
    // This property is required when the same bytecode is uploaded multiple times
    fn save_wasm_allows_saving_multiple_times() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache: CosmCache<MockStorage, MockApi, MockQuerier> =
            unsafe { CosmCache::new(tmp_dir.path(), default_features()).unwrap() };
        cache.save_wasm(CONTRACT).unwrap();
        cache.save_wasm(CONTRACT).unwrap();
    }

    #[test]
    fn save_wasm_rejects_invalid_contract() {
        // Invalid because it doesn't contain required memory and exports
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

        let tmp_dir = TempDir::new().unwrap();
        let mut cache: CosmCache<MockStorage, MockApi, MockQuerier> =
            unsafe { CosmCache::new(tmp_dir.path(), default_features()).unwrap() };
        let save_result = cache.save_wasm(&wasm);
        match save_result.unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                assert_eq!(msg, "Wasm contract doesn\'t have a memory section")
            }
            e => panic!("Unexpected error {:?}", e),
        }
    }

    #[test]
    fn load_wasm_works() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache: CosmCache<MockStorage, MockApi, MockQuerier> =
            unsafe { CosmCache::new(tmp_dir.path(), default_features()).unwrap() };
        let id = cache.save_wasm(CONTRACT).unwrap();

        let restored = cache.load_wasm(&id).unwrap();
        assert_eq!(restored, CONTRACT);
    }

    #[test]
    fn load_wasm_works_across_multiple_cache_instances() {
        let tmp_dir = TempDir::new().unwrap();
        let tmp_path = tmp_dir.path();
        let id: Checksum;

        {
            let mut cache1: CosmCache<MockStorage, MockApi, MockQuerier> =
                unsafe { CosmCache::new(tmp_path, default_features()).unwrap() };
            id = cache1.save_wasm(CONTRACT).unwrap();
        }

        {
            let cache2: CosmCache<MockStorage, MockApi, MockQuerier> =
                unsafe { CosmCache::new(tmp_path, default_features()).unwrap() };
            let restored = cache2.load_wasm(&id).unwrap();
            assert_eq!(restored, CONTRACT);
        }
    }

    #[test]
    fn load_wasm_errors_for_non_existent_id() {
        let tmp_dir = TempDir::new().unwrap();
        let cache: CosmCache<MockStorage, MockApi, MockQuerier> =
            unsafe { CosmCache::new(tmp_dir.path(), default_features()).unwrap() };
        let checksum = Checksum::from([
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            5, 5, 5,
        ]);

        match cache.load_wasm(&checksum).unwrap_err() {
            VmError::CacheErr { msg, .. } => {
                assert!(msg
                    .starts_with("Error opening Wasm file for reading: No such file or directory"))
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn load_wasm_errors_for_corrupted_wasm() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache: CosmCache<MockStorage, MockApi, MockQuerier> =
            unsafe { CosmCache::new(tmp_dir.path(), default_features()).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        // Corrupt cache file
        let filepath = tmp_dir.path().join(WASM_DIR).join(&checksum.to_hex());
        let mut file = OpenOptions::new().write(true).open(filepath).unwrap();
        file.write_all(b"broken data").unwrap();

        let res = cache.load_wasm(&checksum);
        match res {
            Err(VmError::IntegrityErr { .. }) => {}
            Err(e) => panic!("Unexpected error: {:?}", e),
            Ok(_) => panic!("This must not succeed"),
        }
    }

    #[test]
    fn get_instance_finds_cached_module() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { CosmCache::new(tmp_dir.path(), default_features()).unwrap() };
        let id = cache.save_wasm(CONTRACT).unwrap();
        let deps = mock_dependencies(20, &[]);
        let _instance = cache.get_instance(&id, deps, TESTING_GAS_LIMIT).unwrap();
        assert_eq!(cache.stats.hits_module, 1);
        assert_eq!(cache.stats.misses, 0);
    }

    #[test]
    fn get_instance_finds_cached_instance() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { CosmCache::new(tmp_dir.path(), default_features()).unwrap() };
        let id = cache.save_wasm(CONTRACT).unwrap();
        let deps1 = mock_dependencies(20, &[]);
        let deps2 = mock_dependencies(20, &[]);
        let deps3 = mock_dependencies(20, &[]);
        let _instance1 = cache.get_instance(&id, deps1, TESTING_GAS_LIMIT).unwrap();
        let _instance2 = cache.get_instance(&id, deps2, TESTING_GAS_LIMIT).unwrap();
        let _instance3 = cache.get_instance(&id, deps3, TESTING_GAS_LIMIT).unwrap();
        assert_eq!(cache.stats.hits_module, 3);
        assert_eq!(cache.stats.misses, 0);
    }

    #[test]
    fn init_cached_contract() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { CosmCache::new(tmp_dir.path(), default_features()).unwrap() };
        let id = cache.save_wasm(CONTRACT).unwrap();
        let deps = mock_dependencies(20, &[]);
        let mut instance = cache.get_instance(&id, deps, TESTING_GAS_LIMIT).unwrap();

        // run contract
        let env = mock_env(&instance.api, "creator", &coins(1000, "earth"));
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();

        // call and check
        let res = call_init::<_, _, _, Empty>(&mut instance, &env, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(msgs.len(), 0);
    }

    #[test]
    fn run_cached_contract() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { CosmCache::new(tmp_dir.path(), default_features()).unwrap() };
        let id = cache.save_wasm(CONTRACT).unwrap();
        // TODO: contract balance
        let deps = mock_dependencies(20, &[]);
        let mut instance = cache.get_instance(&id, deps, TESTING_GAS_LIMIT).unwrap();

        // init contract
        let env = mock_env(&instance.api, "creator", &coins(1000, "earth"));
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        let res = call_init::<_, _, _, Empty>(&mut instance, &env, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(msgs.len(), 0);

        // run contract - just sanity check - results validate in contract unit tests
        let env = mock_env(&instance.api, "verifies", &coins(15, "earth"));
        let msg = br#"{"release":{}}"#;
        let res = call_handle::<_, _, _, Empty>(&mut instance, &env, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(1, msgs.len());
    }

    #[test]
    fn use_multiple_cached_instances_of_same_contract() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { CosmCache::new(tmp_dir.path(), default_features()).unwrap() };
        let id = cache.save_wasm(CONTRACT).unwrap();

        // these differentiate the two instances of the same contract
        let deps1 = mock_dependencies(20, &[]);
        let deps2 = mock_dependencies(20, &[]);

        // init instance 1
        let mut instance = cache.get_instance(&id, deps1, TESTING_GAS_LIMIT).unwrap();
        let env = mock_env(&instance.api, "owner1", &coins(1000, "earth"));
        let msg = r#"{"verifier": "sue", "beneficiary": "mary"}"#.as_bytes();
        let res = call_init::<_, _, _, Empty>(&mut instance, &env, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(msgs.len(), 0);
        let deps1 = instance.recycle().unwrap();

        // init instance 2
        let mut instance = cache.get_instance(&id, deps2, TESTING_GAS_LIMIT).unwrap();
        let env = mock_env(&instance.api, "owner2", &coins(500, "earth"));
        let msg = r#"{"verifier": "bob", "beneficiary": "john"}"#.as_bytes();
        let res = call_init::<_, _, _, Empty>(&mut instance, &env, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(msgs.len(), 0);
        let deps2 = instance.recycle().unwrap();

        // run contract 2 - just sanity check - results validate in contract unit tests
        let mut instance = cache.get_instance(&id, deps2, TESTING_GAS_LIMIT).unwrap();
        let env = mock_env(&instance.api, "bob", &coins(15, "earth"));
        let msg = br#"{"release":{}}"#;
        let res = call_handle::<_, _, _, Empty>(&mut instance, &env, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(1, msgs.len());

        // run contract 1 - just sanity check - results validate in contract unit tests
        let mut instance = cache.get_instance(&id, deps1, TESTING_GAS_LIMIT).unwrap();
        let env = mock_env(&instance.api, "sue", &coins(15, "earth"));
        let msg = br#"{"release":{}}"#;
        let res = call_handle::<_, _, _, Empty>(&mut instance, &env, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(1, msgs.len());
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn resets_gas_when_reusing_instance() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { CosmCache::new(tmp_dir.path(), default_features()).unwrap() };
        let id = cache.save_wasm(CONTRACT).unwrap();

        let deps1 = mock_dependencies(20, &[]);
        let deps2 = mock_dependencies(20, &[]);

        // Init from module cache
        let mut instance1 = cache.get_instance(&id, deps1, TESTING_GAS_LIMIT).unwrap();
        assert_eq!(cache.stats.hits_module, 1);
        assert_eq!(cache.stats.misses, 0);
        let original_gas = instance1.get_gas_left();

        // Consume some gas
        let env = mock_env(&instance1.api, "owner1", &coins(1000, "earth"));
        let msg = r#"{"verifier": "sue", "beneficiary": "mary"}"#.as_bytes();
        call_init::<_, _, _, Empty>(&mut instance1, &env, msg)
            .unwrap()
            .unwrap();
        assert!(instance1.get_gas_left() < original_gas);

        // Init from instance cache
        let instance2 = cache.get_instance(&id, deps2, TESTING_GAS_LIMIT).unwrap();
        assert_eq!(cache.stats.hits_module, 2);
        assert_eq!(cache.stats.misses, 0);
        assert_eq!(instance2.get_gas_left(), TESTING_GAS_LIMIT);
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn recovers_from_out_of_gas() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { CosmCache::new(tmp_dir.path(), default_features()).unwrap() };
        let id = cache.save_wasm(CONTRACT).unwrap();

        let deps1 = mock_dependencies(20, &[]);
        let deps2 = mock_dependencies(20, &[]);

        // Init from module cache
        let mut instance1 = cache.get_instance(&id, deps1, 10).unwrap();
        assert_eq!(cache.stats.hits_module, 1);
        assert_eq!(cache.stats.misses, 0);

        // Consume some gas. This fails
        let env1 = mock_env(&instance1.api, "owner1", &coins(1000, "earth"));
        let msg1 = r#"{"verifier": "sue", "beneficiary": "mary"}"#.as_bytes();
        match call_init::<_, _, _, Empty>(&mut instance1, &env1, msg1).unwrap_err() {
            VmError::GasDepletion { .. } => (), // all good, continue
            e => panic!("unexpected error, {:?}", e),
        }
        assert_eq!(instance1.get_gas_left(), 0);

        // Init from instance cache
        let mut instance2 = cache.get_instance(&id, deps2, TESTING_GAS_LIMIT).unwrap();
        assert_eq!(cache.stats.hits_module, 2);
        assert_eq!(cache.stats.misses, 0);
        assert_eq!(instance2.get_gas_left(), TESTING_GAS_LIMIT);

        // Now it works
        let env2 = mock_env(&instance2.api, "owner2", &coins(500, "earth"));
        let msg2 = r#"{"verifier": "bob", "beneficiary": "john"}"#.as_bytes();
        call_init::<_, _, _, Empty>(&mut instance2, &env2, msg2)
            .unwrap()
            .unwrap();
    }

    #[test]
    fn save_wasm_to_disk_works_for_same_data_multiple_times() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path();
        let code = vec![12u8; 17];

        save_wasm_to_disk(path, &code).unwrap();
        save_wasm_to_disk(path, &code).unwrap();
    }

    #[test]
    fn save_wasm_to_disk_fails_on_non_existent_dir() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().join("something");
        let code = vec![12u8; 17];
        let res = save_wasm_to_disk(path.to_str().unwrap(), &code);
        assert!(res.is_err());
    }

    #[test]
    fn load_wasm_from_disk_works() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path();
        let code = vec![12u8; 17];
        let id = save_wasm_to_disk(path, &code).unwrap();

        let loaded = load_wasm_from_disk(path, &id).unwrap();
        assert_eq!(code, loaded);
    }

    #[test]
    fn load_wasm_from_disk_works_in_subfolder() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().join("something");
        create_dir_all(&path).unwrap();
        let code = vec![12u8; 17];
        let id = save_wasm_to_disk(&path, &code).unwrap();

        let loaded = load_wasm_from_disk(&path, &id).unwrap();
        assert_eq!(code, loaded);
    }
}
