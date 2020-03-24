use std::fs::create_dir_all;
use std::marker::PhantomData;
use std::path::PathBuf;

use lru::LruCache;
use snafu::ResultExt;

use cosmwasm::{Api, Extern, Storage};

use crate::backends::{backend, compile};
use crate::compatability::check_wasm;
use crate::errors::{Error, IntegrityErr, IoErr};
use crate::instance::Instance;
use crate::modules::{FileSystemCache, WasmHash};
use crate::wasm_store::{load, save, wasm_hash};

static WASM_DIR: &str = "wasm";
static MODULES_DIR: &str = "modules";

#[derive(Debug, Default, Clone)]
struct Stats {
    hits_instance: u32,
    hits_module: u32,
    misses: u32,
}

pub struct CosmCache<S: Storage + 'static, A: Api + 'static> {
    wasm_path: PathBuf,
    modules: FileSystemCache,
    instances: Option<LruCache<WasmHash, wasmer_runtime_core::Instance>>,
    stats: Stats,
    // Those two don't store data but only fix type information
    type_storage: PhantomData<S>,
    type_api: PhantomData<A>,
}

impl<S, A> CosmCache<S, A>
where
    S: Storage + 'static,
    A: Api + 'static,
{
    /// new stores the data for cache under base_dir
    ///
    /// # Safety
    ///
    /// This function is marked unsafe due to `FileSystemCache::new`, which implicitly
    /// assumes the disk contents are correct, and there's no way to ensure the artifacts
    //  stored in the cache haven't been corrupted or tampered with.
    pub unsafe fn new<P: Into<PathBuf>>(base_dir: P, cache_size: usize) -> Result<Self, Error> {
        let base = base_dir.into();
        let wasm_path = base.join(WASM_DIR);
        create_dir_all(&wasm_path).context(IoErr {})?;
        let modules = FileSystemCache::new(base.join(MODULES_DIR)).context(IoErr {})?;
        let instances = if cache_size > 0 {
            Some(LruCache::new(cache_size))
        } else {
            None
        };
        Ok(CosmCache {
            modules,
            wasm_path,
            instances,
            stats: Stats::default(),
            type_storage: PhantomData::<S> {},
            type_api: PhantomData::<A> {},
        })
    }

    pub fn save_wasm(&mut self, wasm: &[u8]) -> Result<Vec<u8>, Error> {
        check_wasm(wasm)?;
        let id = save(&self.wasm_path, wasm)?;
        let module = compile(wasm)?;
        let hash = WasmHash::generate(&id);
        // singlepass cannot store a module, just make best effort
        let _ = self.modules.store(hash, module);
        Ok(id)
    }

    pub fn load_wasm(&self, id: &[u8]) -> Result<Vec<u8>, Error> {
        let code = load(&self.wasm_path, id)?;
        // verify hash matches (integrity check)
        let hash = wasm_hash(&code);
        if hash.ne(&id) {
            IntegrityErr {}.fail()
        } else {
            Ok(code)
        }
    }

    /// get instance returns a wasmer Instance tied to a previously saved wasm
    pub fn get_instance(
        &mut self,
        id: &[u8],
        deps: Extern<S, A>,
        gas_limit: u64,
    ) -> Result<Instance<S, A>, Error> {
        let hash = WasmHash::generate(&id);

        // pop from lru cache if present
        if let Some(cache) = &mut self.instances {
            if let Some(cached_instance) = cache.pop(&hash) {
                self.stats.hits_instance += 1;
                return Ok(Instance::from_wasmer(cached_instance, deps, gas_limit));
            }
        }

        // try from the module cache
        let res = self.modules.load_with_backend(hash, backend());
        if let Ok(module) = res {
            self.stats.hits_module += 1;
            return Instance::from_module(&module, deps, gas_limit);
        }

        // fall back to wasm cache (and re-compiling) - this is for backends that don't support serialization
        let wasm = self.load_wasm(id)?;
        self.stats.misses += 1;
        Instance::from_code(&wasm, deps, gas_limit)
    }

    pub fn store_instance(&mut self, id: &[u8], instance: Instance<S, A>) -> Option<Extern<S, A>> {
        if let Some(cache) = &mut self.instances {
            let hash = WasmHash::generate(&id);
            let (wasmer_instance, ext) = Instance::recycle(instance);
            cache.put(hash, wasmer_instance);
            ext
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::calls::{call_handle, call_init};
    use cosmwasm::coin;
    use cosmwasm::testing::{mock_dependencies, mock_env, MockApi, MockStorage};
    use tempfile::TempDir;

    static TESTING_GAS_LIMIT: u64 = 400_000;
    static CONTRACT_0_7: &[u8] = include_bytes!("../testdata/contract_0.7.wasm");

    #[test]
    fn save_wasm_works() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache: CosmCache<MockStorage, MockApi> =
            unsafe { CosmCache::new(tmp_dir.path(), 10).unwrap() };
        cache.save_wasm(CONTRACT_0_7).unwrap();
    }

    #[test]
    // This property is required when the same bytecode is uploaded multiple times
    fn save_wasm_allows_saving_multiple_times() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache: CosmCache<MockStorage, MockApi> =
            unsafe { CosmCache::new(tmp_dir.path(), 10).unwrap() };
        cache.save_wasm(CONTRACT_0_7).unwrap();
        cache.save_wasm(CONTRACT_0_7).unwrap();
    }

    #[test]
    fn save_wasm_rejects_invalid_contract() {
        use wabt::wat2wasm;

        // this is invalid, as it doesn't contain all required exports
        static WAT: &'static str = r#"
            (module
              (type $t0 (func (param i32) (result i32)))
              (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
                get_local $p0
                i32.const 1
                i32.add))
        "#;

        let wasm = wat2wasm(WAT).unwrap();

        let tmp_dir = TempDir::new().unwrap();
        let mut cache: CosmCache<MockStorage, MockApi> =
            unsafe { CosmCache::new(tmp_dir.path(), 10).unwrap() };
        let save_result = cache.save_wasm(&wasm);
        match save_result {
            Err(Error::ValidationErr { .. }) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn finds_cached_module() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { CosmCache::new(tmp_dir.path(), 10).unwrap() };
        let id = cache.save_wasm(CONTRACT_0_7).unwrap();
        let deps = mock_dependencies(20);
        let _instance = cache.get_instance(&id, deps, TESTING_GAS_LIMIT).unwrap();
        assert_eq!(cache.stats.hits_instance, 0);
        assert_eq!(cache.stats.hits_module, 1);
        assert_eq!(cache.stats.misses, 0);
    }

    #[test]
    fn finds_cached_instance() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { CosmCache::new(tmp_dir.path(), 10).unwrap() };
        let id = cache.save_wasm(CONTRACT_0_7).unwrap();
        let deps1 = mock_dependencies(20);
        let deps2 = mock_dependencies(20);
        let deps3 = mock_dependencies(20);
        let instance1 = cache.get_instance(&id, deps1, TESTING_GAS_LIMIT).unwrap();
        cache.store_instance(&id, instance1);
        let instance2 = cache.get_instance(&id, deps2, TESTING_GAS_LIMIT).unwrap();
        cache.store_instance(&id, instance2);
        let instance3 = cache.get_instance(&id, deps3, TESTING_GAS_LIMIT).unwrap();
        cache.store_instance(&id, instance3);
        assert_eq!(cache.stats.hits_instance, 2);
        assert_eq!(cache.stats.hits_module, 1);
        assert_eq!(cache.stats.misses, 0);
    }

    #[test]
    fn init_cached_contract() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { CosmCache::new(tmp_dir.path(), 10).unwrap() };
        let id = cache.save_wasm(CONTRACT_0_7).unwrap();
        let deps = mock_dependencies(20);
        let mut instance = cache.get_instance(&id, deps, TESTING_GAS_LIMIT).unwrap();

        // run contract
        let env = mock_env(&instance.api, "creator", &coin("1000", "earth"), &[]);
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();

        // call and check
        let res = call_init(&mut instance, &env, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(msgs.len(), 0);
    }

    #[test]
    fn run_cached_contract() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { CosmCache::new(tmp_dir.path(), 10).unwrap() };
        let id = cache.save_wasm(CONTRACT_0_7).unwrap();
        let deps = mock_dependencies(20);
        let mut instance = cache.get_instance(&id, deps, TESTING_GAS_LIMIT).unwrap();

        // init contract
        let env = mock_env(&instance.api, "creator", &coin("1000", "earth"), &[]);
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        let res = call_init(&mut instance, &env, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(msgs.len(), 0);

        // run contract - just sanity check - results validate in contract unit tests
        let env = mock_env(
            &instance.api,
            "verifies",
            &coin("15", "earth"),
            &coin("1015", "earth"),
        );
        let msg = br#"{"release":{}}"#;
        let res = call_handle(&mut instance, &env, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(1, msgs.len());
    }

    #[test]
    fn use_multiple_cached_instances_of_same_contract() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { CosmCache::new(tmp_dir.path(), 10).unwrap() };
        let id = cache.save_wasm(CONTRACT_0_7).unwrap();

        // these differentiate the two instances of the same contract
        let deps1 = mock_dependencies(20);
        let deps2 = mock_dependencies(20);

        // init instance 1
        let mut instance = cache.get_instance(&id, deps1, TESTING_GAS_LIMIT).unwrap();
        let env = mock_env(&instance.api, "owner1", &coin("1000", "earth"), &[]);
        let msg = r#"{"verifier": "sue", "beneficiary": "mary"}"#.as_bytes();
        let res = call_init(&mut instance, &env, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(msgs.len(), 0);
        let deps1 = cache.store_instance(&id, instance).unwrap();

        // init instance 2
        let mut instance = cache.get_instance(&id, deps2, TESTING_GAS_LIMIT).unwrap();
        let env = mock_env(&instance.api, "owner2", &coin("500", "earth"), &[]);
        let msg = r#"{"verifier": "bob", "beneficiary": "john"}"#.as_bytes();
        let res = call_init(&mut instance, &env, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(msgs.len(), 0);
        let deps2 = cache.store_instance(&id, instance).unwrap();

        // run contract 2 - just sanity check - results validate in contract unit tests
        let mut instance = cache.get_instance(&id, deps2, TESTING_GAS_LIMIT).unwrap();
        let env = mock_env(
            &instance.api,
            "bob",
            &coin("15", "earth"),
            &coin("1015", "earth"),
        );
        let msg = br#"{"release":{}}"#;
        let res = call_handle(&mut instance, &env, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(1, msgs.len());
        let _ = cache.store_instance(&id, instance).unwrap();

        // run contract 1 - just sanity check - results validate in contract unit tests
        let mut instance = cache.get_instance(&id, deps1, TESTING_GAS_LIMIT).unwrap();
        let env = mock_env(
            &instance.api,
            "sue",
            &coin("15", "earth"),
            &coin("1015", "earth"),
        );
        let msg = br#"{"release":{}}"#;
        let res = call_handle(&mut instance, &env, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(1, msgs.len());
        let _ = cache.store_instance(&id, instance);
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn resets_gas_when_reusing_instance() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { CosmCache::new(tmp_dir.path(), 10).unwrap() };
        let id = cache.save_wasm(CONTRACT_0_7).unwrap();

        let deps1 = mock_dependencies(20);
        let deps2 = mock_dependencies(20);

        // Init from module cache
        let mut instance1 = cache.get_instance(&id, deps1, TESTING_GAS_LIMIT).unwrap();
        assert_eq!(cache.stats.hits_module, 1);
        assert_eq!(cache.stats.hits_instance, 0);
        assert_eq!(cache.stats.misses, 0);
        let original_gas = instance1.get_gas();

        // Consume some gas
        let env = mock_env(&instance1.api, "owner1", &coin("1000", "earth"), &[]);
        let msg = r#"{"verifier": "sue", "beneficiary": "mary"}"#.as_bytes();
        call_init(&mut instance1, &env, msg).unwrap();
        assert!(instance1.get_gas() < original_gas);
        cache.store_instance(&id, instance1).unwrap();

        // Init from instance cache
        let instance2 = cache.get_instance(&id, deps2, TESTING_GAS_LIMIT).unwrap();
        assert_eq!(cache.stats.hits_module, 1);
        assert_eq!(cache.stats.hits_instance, 1);
        assert_eq!(cache.stats.misses, 0);
        assert_eq!(instance2.get_gas(), TESTING_GAS_LIMIT);
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn recovers_from_out_of_gas() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { CosmCache::new(tmp_dir.path(), 10).unwrap() };
        let id = cache.save_wasm(CONTRACT_0_7).unwrap();

        let deps1 = mock_dependencies(20);
        let deps2 = mock_dependencies(20);

        // Init from module cache
        let mut instance1 = cache.get_instance(&id, deps1, 10).unwrap();
        assert_eq!(cache.stats.hits_module, 1);
        assert_eq!(cache.stats.hits_instance, 0);
        assert_eq!(cache.stats.misses, 0);

        // Consume some gas. This fails
        let env1 = mock_env(&instance1.api, "owner1", &coin("1000", "earth"), &[]);
        let msg1 = r#"{"verifier": "sue", "beneficiary": "mary"}"#.as_bytes();
        match call_init(&mut instance1, &env1, msg1) {
            Err(Error::RuntimeErr { .. }) => (), // all good, continue
            Err(e) => panic!("unexpected error, {:?}", e),
            Ok(_) => panic!("call_init must run out of gas"),
        }
        assert_eq!(instance1.get_gas(), 0);
        cache.store_instance(&id, instance1).unwrap();

        // Init from instance cache
        let mut instance2 = cache.get_instance(&id, deps2, TESTING_GAS_LIMIT).unwrap();
        assert_eq!(cache.stats.hits_module, 1);
        assert_eq!(cache.stats.hits_instance, 1);
        assert_eq!(cache.stats.misses, 0);
        assert_eq!(instance2.get_gas(), TESTING_GAS_LIMIT);

        // Now it works
        let env2 = mock_env(&instance2.api, "owner2", &coin("500", "earth"), &[]);
        let msg2 = r#"{"verifier": "bob", "beneficiary": "john"}"#.as_bytes();
        call_init(&mut instance2, &env2, msg2).unwrap();
    }
}
