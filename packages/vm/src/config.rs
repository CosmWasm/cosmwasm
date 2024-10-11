use std::{collections::HashSet, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::Size;

const DEFAULT_MEMORY_LIMIT: u32 = 512; // in pages
/// As of March 2023, on Juno mainnet the largest value for production contracts
/// is 485. Most are between 100 and 300.
const DEFAULT_TABLE_SIZE_LIMIT: u32 = 2500; // entries

/// We keep this number high since failing early gives less detailed error messages. Especially
/// when a user accidentally includes wasm-bindgen, they get a bunch of unsupported imports.
const DEFAULT_MAX_IMPORTS: usize = 100;

const DEFAULT_MAX_FUNCTIONS: usize = 20_000;

const DEFAULT_MAX_FUNCTION_PARAMS: usize = 100;

const DEFAULT_MAX_TOTAL_FUNCTION_PARAMS: usize = 10_000;

const DEFAULT_MAX_FUNCTION_RESULTS: usize = 1;

/// Various configurations for the VM.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Config {
    /// Configuration for limitations placed on Wasm files.
    /// This defines a few limits on the Wasm file that are checked during static validation before
    /// storing the Wasm file.
    pub wasm_limits: WasmLimits,

    /// Configuration for the cache.
    pub cache: CacheOptions,
}

impl Config {
    pub fn new(cache: CacheOptions) -> Self {
        Self {
            wasm_limits: WasmLimits::default(),
            cache,
        }
    }
}

/// Limits for static validation of Wasm files. These are checked before storing the Wasm file.
/// All limits are optional because they are coming from the Go-side and have default values.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct WasmLimits {
    /// Maximum number of memory pages that a module can request.
    ///
    /// Every Wasm memory has an initial size and an optional maximum size,
    /// both measured in Wasm pages. This limit applies to the initial size.
    pub initial_memory_limit_pages: Option<u32>,
    /// The upper limit for the `max` value of each table. CosmWasm contracts have
    /// initial=max for 1 table. See
    ///
    /// ```plain
    /// $ wasm-objdump --section=table -x packages/vm/testdata/hackatom.wasm
    /// Section Details:
    ///
    /// Table[1]:
    /// - table[0] type=funcref initial=161 max=161
    /// ```
    ///
    pub table_size_limit_elements: Option<u32>,
    /// If the contract has more than this amount of imports, it will be rejected
    /// during static validation before even looking into the imports.
    pub max_imports: Option<usize>,

    /// The maximum number of functions a contract can have.
    /// Any contract with more functions than this will be rejected during static validation.
    pub max_functions: Option<usize>,

    /// The maximum number of parameters a Wasm function can have.
    pub max_function_params: Option<usize>,
    /// The maximum total number of parameters of all functions in the Wasm.
    /// For each function in the Wasm, take the number of parameters and sum all of these up.
    /// If that sum exceeds this limit, the Wasm will be rejected during static validation.
    ///
    /// Be careful when adjusting this limit, as it prevents an attack where a small Wasm file
    /// explodes in size when compiled.
    pub max_total_function_params: Option<usize>,

    /// The maximum number of results a Wasm function type can have.
    pub max_function_results: Option<usize>,
}

impl WasmLimits {
    pub fn initial_memory_limit_pages(&self) -> u32 {
        self.initial_memory_limit_pages
            .unwrap_or(DEFAULT_MEMORY_LIMIT)
    }

    pub fn table_size_limit_elements(&self) -> u32 {
        self.table_size_limit_elements
            .unwrap_or(DEFAULT_TABLE_SIZE_LIMIT)
    }

    pub fn max_imports(&self) -> usize {
        self.max_imports.unwrap_or(DEFAULT_MAX_IMPORTS)
    }

    pub fn max_functions(&self) -> usize {
        self.max_functions.unwrap_or(DEFAULT_MAX_FUNCTIONS)
    }

    pub fn max_function_params(&self) -> usize {
        self.max_function_params
            .unwrap_or(DEFAULT_MAX_FUNCTION_PARAMS)
    }

    pub fn max_total_function_params(&self) -> usize {
        self.max_total_function_params
            .unwrap_or(DEFAULT_MAX_TOTAL_FUNCTION_PARAMS)
    }

    pub fn max_function_results(&self) -> usize {
        self.max_function_results
            .unwrap_or(DEFAULT_MAX_FUNCTION_RESULTS)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CacheOptions {
    /// The base directory of this cache.
    ///
    /// If this does not exist, it will be created. Not sure if this behaviour
    /// is desired but wasmd relies on it.
    pub base_dir: PathBuf,
    pub available_capabilities: HashSet<String>,
    /// Memory limit for the cache, in bytes.
    pub memory_cache_size_bytes: Size,
    /// Memory limit for instances, in bytes. Use a value that is divisible by the Wasm page size 65536,
    /// e.g. full MiBs.
    pub instance_memory_limit_bytes: Size,
}

impl CacheOptions {
    pub fn new(
        base_dir: impl Into<PathBuf>,
        available_capabilities: impl Into<HashSet<String>>,
        memory_cache_size_bytes: Size,
        instance_memory_limit_bytes: Size,
    ) -> Self {
        Self {
            base_dir: base_dir.into(),
            available_capabilities: available_capabilities.into(),
            memory_cache_size_bytes,
            instance_memory_limit_bytes,
        }
    }
}
