use std::collections::HashSet;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::path::PathBuf;

use crate::backend::{Api, Backend, Querier, Storage};
use crate::checksum::Checksum;
use crate::compatibility::check_wasm;
use crate::errors::{VmError, VmResult};
use crate::instance::{Instance, InstanceOptions};
use crate::modules::{FileSystemCache, InMemoryCache};
use crate::size::Size;
use crate::wasm_backend::{compile_and_use, compile_only, make_runtime_store};

const WASM_DIR: &str = "wasm";
const MODULES_DIR: &str = "modules";

#[derive(Debug, Default, Clone, Copy)]
pub struct Stats {
    pub hits_memory_cache: u32,
    pub hits_fs_cache: u32,
    pub misses: u32,
}

#[derive(Clone, Debug)]
pub struct CacheOptions {
    pub base_dir: PathBuf,
    pub supported_features: HashSet<String>,
    pub memory_cache_size: Size,
}

pub struct Cache<A: Api, S: Storage, Q: Querier> {
    wasm_path: PathBuf,
    supported_features: HashSet<String>,
    memory_cache: InMemoryCache,
    fs_cache: FileSystemCache,
    stats: Stats,
    // Those two don't store data but only fix type information
    type_api: PhantomData<A>,
    type_storage: PhantomData<S>,
    type_querier: PhantomData<Q>,
}

impl<A, S, Q> Cache<A, S, Q>
where
    A: Api + 'static,     // 'static is needed by `impl<…> Instance`
    S: Storage + 'static, // 'static is needed by `impl<…> Instance`
    Q: Querier + 'static, // 'static is needed by `impl<…> Instance`
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
    pub unsafe fn new(options: CacheOptions) -> VmResult<Self> {
        let CacheOptions {
            base_dir,
            supported_features,
            memory_cache_size,
        } = options;
        let wasm_path = base_dir.join(WASM_DIR);
        create_dir_all(&wasm_path)
            .map_err(|e| VmError::cache_err(format!("Error creating Wasm dir for cache: {}", e)))?;

        let fs_cache = FileSystemCache::new(base_dir.join(MODULES_DIR))
            .map_err(|e| VmError::cache_err(format!("Error file system cache: {}", e)))?;
        Ok(Cache {
            wasm_path,
            supported_features,
            memory_cache: InMemoryCache::new(memory_cache_size),
            fs_cache,
            stats: Stats::default(),
            type_storage: PhantomData::<S>,
            type_api: PhantomData::<A>,
            type_querier: PhantomData::<Q>,
        })
    }

    pub fn stats(&self) -> Stats {
        self.stats
    }

    pub fn save_wasm(&mut self, wasm: &[u8]) -> VmResult<Checksum> {
        check_wasm(wasm, &self.supported_features)?;
        let checksum = save_wasm_to_disk(&self.wasm_path, wasm)?;
        let module = compile_only(wasm)?;
        self.fs_cache.store(&checksum, &module)?;
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
        backend: Backend<A, S, Q>,
        options: InstanceOptions,
    ) -> VmResult<Instance<A, S, Q>> {
        let store = make_runtime_store(options.memory_limit);
        // Get module from memory cache
        if let Some(module) = self.memory_cache.load(checksum, &store)? {
            self.stats.hits_memory_cache += 1;
            let instance =
                Instance::from_module(&module, backend, options.gas_limit, options.print_debug)?;
            return Ok(instance);
        }

        // Get module from file system cache
        if let Some(module) = self.fs_cache.load(checksum, &store)? {
            self.stats.hits_fs_cache += 1;
            let instance =
                Instance::from_module(&module, backend, options.gas_limit, options.print_debug)?;
            self.memory_cache.store(checksum, module)?;
            return Ok(instance);
        }

        // Re-compile module from wasm
        let wasm = self.load_wasm(checksum)?;
        self.stats.misses += 1;
        let module = compile_and_use(&wasm, options.memory_limit)?;
        let instance =
            Instance::from_module(&module, backend, options.gas_limit, options.print_debug)?;
        self.fs_cache.store(checksum, &module)?;
        self.memory_cache.store(checksum, module)?;
        Ok(instance)
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
mod tests {
    use super::*;
    use crate::calls::{call_handle, call_init};
    use crate::errors::VmError;
    use crate::features::features_from_csv;
    use crate::testing::{mock_backend, mock_env, mock_info, MockApi, MockQuerier, MockStorage};
    use cosmwasm_std::{coins, Empty};
    use std::fs::OpenOptions;
    use std::io::Write;
    use tempfile::TempDir;

    const TESTING_GAS_LIMIT: u64 = 4_000_000;
    const TESTING_MEMORY_LIMIT: Size = Size::mebi(16);
    const TESTING_OPTIONS: InstanceOptions = InstanceOptions {
        gas_limit: TESTING_GAS_LIMIT,
        memory_limit: TESTING_MEMORY_LIMIT,
        print_debug: false,
    };
    const TESTING_MEMORY_CACHE_SIZE: Size = Size::mebi(200);

    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

    fn default_features() -> HashSet<String> {
        features_from_csv("staking")
    }

    fn make_testing_options() -> CacheOptions {
        CacheOptions {
            base_dir: TempDir::new().unwrap().into_path(),
            supported_features: default_features(),
            memory_cache_size: TESTING_MEMORY_CACHE_SIZE,
        }
    }

    #[test]
    fn save_wasm_works() {
        let mut cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(make_testing_options()).unwrap() };
        cache.save_wasm(CONTRACT).unwrap();
    }

    #[test]
    // This property is required when the same bytecode is uploaded multiple times
    fn save_wasm_allows_saving_multiple_times() {
        let mut cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(make_testing_options()).unwrap() };
        cache.save_wasm(CONTRACT).unwrap();
        cache.save_wasm(CONTRACT).unwrap();
    }

    #[test]
    fn save_wasm_rejects_invalid_contract() {
        // Invalid because it doesn't contain required memory and exports
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

        let mut cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(make_testing_options()).unwrap() };
        let save_result = cache.save_wasm(&wasm);
        match save_result.unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                assert_eq!(msg, "Wasm contract doesn\'t have a memory section")
            }
            e => panic!("Unexpected error {:?}", e),
        }
    }

    #[test]
    fn save_wasm_fills_file_system_but_not_memory_cache() {
        // Who knows if and when the uploaded contract will be executed. Don't pollute
        // memory cache before the init call.

        let mut cache = unsafe { Cache::new(make_testing_options()).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        let backend = mock_backend(&[]);
        let _ = cache
            .get_instance(&checksum, backend, TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_memory_cache, 0);
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);
    }

    #[test]
    fn load_wasm_works() {
        let mut cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(make_testing_options()).unwrap() };
        let id = cache.save_wasm(CONTRACT).unwrap();

        let restored = cache.load_wasm(&id).unwrap();
        assert_eq!(restored, CONTRACT);
    }

    #[test]
    fn load_wasm_works_across_multiple_cache_instances() {
        let tmp_dir = TempDir::new().unwrap();
        let id: Checksum;

        {
            let options1 = CacheOptions {
                base_dir: tmp_dir.path().to_path_buf(),
                supported_features: default_features(),
                memory_cache_size: TESTING_MEMORY_CACHE_SIZE,
            };
            let mut cache1: Cache<MockApi, MockStorage, MockQuerier> =
                unsafe { Cache::new(options1).unwrap() };
            id = cache1.save_wasm(CONTRACT).unwrap();
        }

        {
            let options2 = CacheOptions {
                base_dir: tmp_dir.path().to_path_buf(),
                supported_features: default_features(),
                memory_cache_size: TESTING_MEMORY_CACHE_SIZE,
            };
            let cache2: Cache<MockApi, MockStorage, MockQuerier> =
                unsafe { Cache::new(options2).unwrap() };
            let restored = cache2.load_wasm(&id).unwrap();
            assert_eq!(restored, CONTRACT);
        }
    }

    #[test]
    fn load_wasm_errors_for_non_existent_id() {
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(make_testing_options()).unwrap() };
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
        let options = CacheOptions {
            base_dir: tmp_dir.path().to_path_buf(),
            supported_features: default_features(),
            memory_cache_size: TESTING_MEMORY_CACHE_SIZE,
        };
        let mut cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options).unwrap() };
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
        let mut cache = unsafe { Cache::new(make_testing_options()).unwrap() };
        let id = cache.save_wasm(CONTRACT).unwrap();
        let backend = mock_backend(&[]);
        let _instance = cache.get_instance(&id, backend, TESTING_OPTIONS).unwrap();
        assert_eq!(cache.stats().hits_memory_cache, 0);
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);
    }

    #[test]
    fn get_instance_finds_cached_modules_and_stores_to_memory() {
        let mut cache = unsafe { Cache::new(make_testing_options()).unwrap() };
        let id = cache.save_wasm(CONTRACT).unwrap();
        let backend1 = mock_backend(&[]);
        let backend2 = mock_backend(&[]);
        let backend3 = mock_backend(&[]);

        // from file system
        let _instance1 = cache.get_instance(&id, backend1, TESTING_OPTIONS).unwrap();
        assert_eq!(cache.stats().hits_memory_cache, 0);
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);

        // from memory
        let _instance2 = cache.get_instance(&id, backend2, TESTING_OPTIONS).unwrap();
        assert_eq!(cache.stats().hits_memory_cache, 1);
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);

        // from memory again
        let _instance3 = cache.get_instance(&id, backend3, TESTING_OPTIONS).unwrap();
        assert_eq!(cache.stats().hits_memory_cache, 2);
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);
    }

    #[test]
    fn execute_init_on_cached_contract() {
        let mut cache = unsafe { Cache::new(make_testing_options()).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        // from file system
        {
            let mut instance = cache
                .get_instance(&checksum, mock_backend(&[]), TESTING_OPTIONS)
                .unwrap();
            assert_eq!(cache.stats().hits_memory_cache, 0);
            assert_eq!(cache.stats().hits_fs_cache, 1);
            assert_eq!(cache.stats().misses, 0);

            // init
            let info = mock_info("creator", &coins(1000, "earth"));
            let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
            let res = call_init::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
            let msgs = res.unwrap().messages;
            assert_eq!(msgs.len(), 0);
        }

        // from memory
        {
            let mut instance = cache
                .get_instance(&checksum, mock_backend(&[]), TESTING_OPTIONS)
                .unwrap();
            assert_eq!(cache.stats().hits_memory_cache, 1);
            assert_eq!(cache.stats().hits_fs_cache, 1);
            assert_eq!(cache.stats().misses, 0);

            // init
            let info = mock_info("creator", &coins(1000, "earth"));
            let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
            let res = call_init::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
            let msgs = res.unwrap().messages;
            assert_eq!(msgs.len(), 0);
        }
    }

    #[test]
    fn execute_handle_on_cached_contract() {
        let mut cache = unsafe { Cache::new(make_testing_options()).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        // from file system
        {
            let mut instance = cache
                .get_instance(&checksum, mock_backend(&[]), TESTING_OPTIONS)
                .unwrap();
            assert_eq!(cache.stats().hits_memory_cache, 0);
            assert_eq!(cache.stats().hits_fs_cache, 1);
            assert_eq!(cache.stats().misses, 0);

            // init
            let info = mock_info("creator", &coins(1000, "earth"));
            let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
            let response = call_init::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
                .unwrap()
                .unwrap();
            assert_eq!(response.messages.len(), 0);

            // handle
            let info = mock_info("verifies", &coins(15, "earth"));
            let msg = br#"{"release":{}}"#;
            let response = call_handle::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
                .unwrap()
                .unwrap();
            assert_eq!(response.messages.len(), 1);
        }

        // from memory
        {
            let mut instance = cache
                .get_instance(&checksum, mock_backend(&[]), TESTING_OPTIONS)
                .unwrap();
            assert_eq!(cache.stats().hits_memory_cache, 1);
            assert_eq!(cache.stats().hits_fs_cache, 1);
            assert_eq!(cache.stats().misses, 0);

            // init
            let info = mock_info("creator", &coins(1000, "earth"));
            let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
            let response = call_init::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
                .unwrap()
                .unwrap();
            assert_eq!(response.messages.len(), 0);

            // handle
            let info = mock_info("verifies", &coins(15, "earth"));
            let msg = br#"{"release":{}}"#;
            let response = call_handle::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
                .unwrap()
                .unwrap();
            assert_eq!(response.messages.len(), 1);
        }
    }

    #[test]
    fn use_multiple_cached_instances_of_same_contract() {
        let mut cache = unsafe { Cache::new(make_testing_options()).unwrap() };
        let id = cache.save_wasm(CONTRACT).unwrap();

        // these differentiate the two instances of the same contract
        let backend1 = mock_backend(&[]);
        let backend2 = mock_backend(&[]);

        // init instance 1
        let mut instance = cache.get_instance(&id, backend1, TESTING_OPTIONS).unwrap();
        let info = mock_info("owner1", &coins(1000, "earth"));
        let msg = r#"{"verifier": "sue", "beneficiary": "mary"}"#.as_bytes();
        let res = call_init::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(msgs.len(), 0);
        let backend1 = instance.recycle().unwrap();

        // init instance 2
        let mut instance = cache.get_instance(&id, backend2, TESTING_OPTIONS).unwrap();
        let info = mock_info("owner2", &coins(500, "earth"));
        let msg = r#"{"verifier": "bob", "beneficiary": "john"}"#.as_bytes();
        let res = call_init::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(msgs.len(), 0);
        let backend2 = instance.recycle().unwrap();

        // run contract 2 - just sanity check - results validate in contract unit tests
        let mut instance = cache.get_instance(&id, backend2, TESTING_OPTIONS).unwrap();
        let info = mock_info("bob", &coins(15, "earth"));
        let msg = br#"{"release":{}}"#;
        let res = call_handle::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(1, msgs.len());

        // run contract 1 - just sanity check - results validate in contract unit tests
        let mut instance = cache.get_instance(&id, backend1, TESTING_OPTIONS).unwrap();
        let info = mock_info("sue", &coins(15, "earth"));
        let msg = br#"{"release":{}}"#;
        let res = call_handle::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(1, msgs.len());
    }

    #[test]
    fn resets_gas_when_reusing_instance() {
        let mut cache = unsafe { Cache::new(make_testing_options()).unwrap() };
        let id = cache.save_wasm(CONTRACT).unwrap();

        let backend1 = mock_backend(&[]);
        let backend2 = mock_backend(&[]);

        // Init from module cache
        let mut instance1 = cache.get_instance(&id, backend1, TESTING_OPTIONS).unwrap();
        assert_eq!(cache.stats().hits_memory_cache, 0);
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);
        let original_gas = instance1.get_gas_left();

        // Consume some gas
        let info = mock_info("owner1", &coins(1000, "earth"));
        let msg = r#"{"verifier": "sue", "beneficiary": "mary"}"#.as_bytes();
        call_init::<_, _, _, Empty>(&mut instance1, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();
        assert!(instance1.get_gas_left() < original_gas);

        // Init from memory cache
        let instance2 = cache.get_instance(&id, backend2, TESTING_OPTIONS).unwrap();
        assert_eq!(cache.stats().hits_memory_cache, 1);
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);
        assert_eq!(instance2.get_gas_left(), TESTING_GAS_LIMIT);
    }

    #[test]
    fn recovers_from_out_of_gas() {
        let mut cache = unsafe { Cache::new(make_testing_options()).unwrap() };
        let id = cache.save_wasm(CONTRACT).unwrap();

        let backend1 = mock_backend(&[]);
        let backend2 = mock_backend(&[]);

        // Init from module cache
        let options = InstanceOptions {
            gas_limit: 10,
            memory_limit: TESTING_MEMORY_LIMIT,
            print_debug: false,
        };
        let mut instance1 = cache.get_instance(&id, backend1, options).unwrap();
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);

        // Consume some gas. This fails
        let info1 = mock_info("owner1", &coins(1000, "earth"));
        let msg1 = r#"{"verifier": "sue", "beneficiary": "mary"}"#.as_bytes();
        match call_init::<_, _, _, Empty>(&mut instance1, &mock_env(), &info1, msg1).unwrap_err() {
            VmError::GasDepletion { .. } => (), // all good, continue
            e => panic!("unexpected error, {:?}", e),
        }
        assert_eq!(instance1.get_gas_left(), 0);

        // Init from memory cache
        let options = InstanceOptions {
            gas_limit: TESTING_GAS_LIMIT,
            memory_limit: TESTING_MEMORY_LIMIT,
            print_debug: false,
        };
        let mut instance2 = cache.get_instance(&id, backend2, options).unwrap();
        assert_eq!(cache.stats().hits_memory_cache, 1);
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);
        assert_eq!(instance2.get_gas_left(), TESTING_GAS_LIMIT);

        // Now it works
        let info2 = mock_info("owner2", &coins(500, "earth"));
        let msg2 = r#"{"verifier": "bob", "beneficiary": "john"}"#.as_bytes();
        call_init::<_, _, _, Empty>(&mut instance2, &mock_env(), &info2, msg2)
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
