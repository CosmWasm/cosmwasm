use std::fs::create_dir_all;
use std::path::PathBuf;

use failure::Error;

use crate::backends::compile;
use crate::modules::{Cache, FileSystemCache, WasmHash};
use crate::wasm_store::{load, save};
use crate::wasmer::{instantiate, Instance};

pub struct CosmCache {
    wasm_path: PathBuf,
    modules: FileSystemCache,
}

static WASM_DIR: &str = "wasm";
static MODULES_DIR: &str = "modules";

impl CosmCache {
    /// new stores the data for cache under base_dir
    pub unsafe fn new<P: Into<PathBuf>>(base_dir: P) -> Self {
        let base = base_dir.into();
        let wasm_path = base.join(WASM_DIR);
        create_dir_all(&wasm_path).unwrap();
        let modules = FileSystemCache::new(base.join(MODULES_DIR)).unwrap();
        CosmCache { modules, wasm_path }
    }
}

impl CosmCache {
    pub fn save_wasm(&mut self, wasm: &[u8]) -> Result<Vec<u8>, Error> {
        let id = save(&self.wasm_path, wasm)?;
        // we fail if module doesn't compile - panic :(
        let module = compile(wasm);
        let hash = WasmHash::generate(&id);
        let saved = self.modules.store(hash, module);
        // ignore it (just log) if module cache not supported
        if let Err(e) = saved {
            println!("Cannot save module: {:?}", e);
        }
        Ok(id)
    }

    pub fn load_wasm(&self, id: &[u8]) -> Result<Vec<u8>, Error> {
        // TODO: load modules cache if present
        load(&self.wasm_path, id)
        // TODO: verify hash matches
    }

    /// get instance returns a wasmer Instance tied to a previously saved wasm
    pub fn get_instance(&self, id: &[u8]) -> Result<Instance, Error> {
        // TODO: we can definitely add some caches (module on disk, instance in memory) to make this faster
        let wasm = self.load_wasm(id)?;
        Ok(instantiate(&wasm))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::TempDir;

    use crate::calls::{call_handle, call_init};
    use cosmwasm::types::{coin, mock_params};

    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

    #[test]
    fn init_cached_contract() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { CosmCache::new(tmp_dir.path().to_str().unwrap()) };
        let id = cache.save_wasm(CONTRACT).unwrap();
        let mut instance = cache.get_instance(&id).unwrap();

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
        let mut cache = unsafe { CosmCache::new(tmp_dir.path().to_str().unwrap()) };
        let id = cache.save_wasm(CONTRACT).unwrap();
        let mut instance = cache.get_instance(&id).unwrap();

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
}
