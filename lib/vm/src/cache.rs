use std::fs::create_dir_all;
use std::path::PathBuf;

use lru::LruCache;
use snafu::ResultExt;

use cosmwasm::storage::Storage;

use crate::backends::{backend, compile};
use crate::errors::{Error, IntegrityErr, IoErr};
use crate::instance::Instance;
use crate::modules::{Cache, FileSystemCache, WasmHash};
use crate::wasm_store::{load, save, wasm_hash};

static WASM_DIR: &str = "wasm";
static MODULES_DIR: &str = "modules";

pub struct CosmCache<T: Storage + 'static> {
    wasm_path: PathBuf,
    modules: FileSystemCache,
    instances: Option<LruCache<WasmHash, Instance<T>>>,
}

impl<T> CosmCache<T>
where
    T: Storage + 'static,
{
    /// new stores the data for cache under base_dir
    pub unsafe fn new<P: Into<PathBuf>>(base_dir: P, cache_size: usize) -> Result<Self, Error> {
        let base = base_dir.into();
        let wasm_path = base.join(WASM_DIR);
        create_dir_all(&wasm_path).context(IoErr{})?;
        let modules = FileSystemCache::new(base.join(MODULES_DIR)).context(IoErr{})?;
        let instances = if cache_size > 0 {
            Some(LruCache::new(cache_size))
        } else {
            None
        };
        Ok(CosmCache {
            modules,
            wasm_path,
            instances,
        })
    }

    pub fn save_wasm(&mut self, wasm: &[u8]) -> Result<Vec<u8>, Error> {
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
    pub fn get_instance(&mut self, id: &[u8], storage: T) -> Result<Instance<T>, Error> {
        let hash = WasmHash::generate(&id);

        // pop from lru cache if present
        if let Some(cache) = &mut self.instances {
            let val = cache.pop(&hash);
            if let Some(inst) = val {
                inst.leave_storage(Some(storage));
                return Ok(inst);
            }
        }

        // try from the module cache
        let res = self.modules.load_with_backend(hash, backend());
        if let Ok(module) = res {
            return Instance::from_module(&module, storage);
        }

        // fall back to wasm cache (and re-compiling) - this is for backends that don't support serialization
        let wasm = self.load_wasm(id)?;
        Instance::from_code(&wasm, storage)
    }

    pub fn store_instance(&mut self, id: &[u8], instance: Instance<T>) -> Option<T> {
        if let Some(cache) = &mut self.instances {
            let hash = WasmHash::generate(&id);
            let storage = instance.take_storage();
            cache.put(hash, instance);
            storage
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::TempDir;

    use crate::calls::{call_handle, call_init};
    use cosmwasm::mock::MockStorage;
    use cosmwasm::types::{coin, mock_params};

    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

    #[test]
    fn init_cached_contract() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { CosmCache::new(tmp_dir.path(), 10).unwrap() };
        let id = cache.save_wasm(CONTRACT).unwrap();
        let storage = MockStorage::new();
        let mut instance = cache.get_instance(&id, storage).unwrap();

        // run contract
        let params = mock_params("creator", &coin("1000", "earth"), &[]);
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();

        // call and check
        let res = call_init(&mut instance, &params, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(msgs.len(), 0);
    }

    #[test]
    fn run_cached_contract() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { CosmCache::new(tmp_dir.path(), 10).unwrap() };
        let id = cache.save_wasm(CONTRACT).unwrap();
        let storage = MockStorage::new();
        let mut instance = cache.get_instance(&id, storage).unwrap();

        // init contract
        let params = mock_params("creator", &coin("1000", "earth"), &[]);
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        let res = call_init(&mut instance, &params, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(msgs.len(), 0);

        // run contract - just sanity check - results validate in contract unit tests
        let params = mock_params("verifies", &coin("15", "earth"), &coin("1015", "earth"));
        let msg = b"{}";
        let res = call_handle(&mut instance, &params, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(1, msgs.len());
    }

    #[test]
    fn use_multiple_cached_instances_of_same_contract() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { CosmCache::new(tmp_dir.path(), 10).unwrap() };
        let id = cache.save_wasm(CONTRACT).unwrap();

        // these differentiate the two instances of the same contract
        let storage1 = MockStorage::new();
        let storage2 = MockStorage::new();

        // init instance 1
        let mut instance = cache.get_instance(&id, storage1).unwrap();
        let params = mock_params("owner1", &coin("1000", "earth"), &[]);
        let msg = r#"{"verifier": "sue", "beneficiary": "mary"}"#.as_bytes();
        let res = call_init(&mut instance, &params, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(msgs.len(), 0);
        let storage1 = cache.store_instance(&id, instance).unwrap();

        // init instance 2
        let mut instance = cache.get_instance(&id, storage2).unwrap();
        let params = mock_params("owner2", &coin("500", "earth"), &[]);
        let msg = r#"{"verifier": "bob", "beneficiary": "john"}"#.as_bytes();
        let res = call_init(&mut instance, &params, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(msgs.len(), 0);
        let storage2 = cache.store_instance(&id, instance).unwrap();

        // run contract 2 - just sanity check - results validate in contract unit tests
        let mut instance = cache.get_instance(&id, storage2).unwrap();
        let params = mock_params("bob", &coin("15", "earth"), &coin("1015", "earth"));
        let msg = b"{}";
        let res = call_handle(&mut instance, &params, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(1, msgs.len());
        let _ = cache.store_instance(&id, instance).unwrap();

        // run contract 1 - just sanity check - results validate in contract unit tests
        let mut instance = cache.get_instance(&id, storage1).unwrap();
        let params = mock_params("sue", &coin("15", "earth"), &coin("1015", "earth"));
        let msg = b"{}";
        let res = call_handle(&mut instance, &params, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(1, msgs.len());
        let _ = cache.store_instance(&id, instance);
    }
}
