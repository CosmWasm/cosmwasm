use std::path::{Path, PathBuf};

use failure::Error;

use crate::wasm_store::{ensure_dir, load, save};
use crate::wasmer::{instantiate, Instance};

pub struct Cache {
    wasm_dir: PathBuf,
}

static WASM_DIR: &str = "wasm";

impl Cache {
    /// new stores the data for cache under base_dir
    pub fn new(base_dir: &str) -> Self {
        let wasm_dir = Path::new(base_dir).join(WASM_DIR);
        let cache = Cache { wasm_dir };
        ensure_dir(cache.wasm_path()).unwrap();
        cache
    }

    fn wasm_path(&self) -> &str {
        self.wasm_dir.to_str().unwrap()
    }
}

impl Cache {
    pub fn save_wasm(&mut self, wasm: &[u8]) -> Result<Vec<u8>, Error> {
        save(self.wasm_path(), wasm)
    }

    pub fn load_wasm(&self, id: &[u8]) -> Result<Vec<u8>, Error> {
        load(self.wasm_path(), id)
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
    use tempdir::TempDir;

    use crate::calls::call_init;
    use cosmwasm::types::{coin, mock_params};

    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

    #[test]
    fn run_cached_contract() {
        let tmp_dir = TempDir::new("comswasm_cache_test").unwrap();
        let mut cache = Cache::new(tmp_dir.path().to_str().unwrap());
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
}
