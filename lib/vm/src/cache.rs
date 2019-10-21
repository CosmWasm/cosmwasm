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
    pub fn save_wasm(&self, wasm: &[u8]) -> Result<Vec<u8>, Error> {
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
