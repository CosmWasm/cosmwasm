use std::collections::HashSet;
use std::sync::Arc;
use tempfile::TempDir;

use cosmwasm_std::{coins, to_vec};
use cosmwasm_vm::testing::{mock_backend, mock_env, mock_info, MockApi};
use cosmwasm_vm::{
    call_execute_raw, call_instantiate_raw, Cache, CacheOptions, InstanceOptions, Size,
};

// Instance
const DEFAULT_MEMORY_LIMIT: Size = Size::mebi(64);
const DEFAULT_GAS_LIMIT: u64 = 400_000 * 150_000;
const DEFAULT_INSTANCE_OPTIONS: InstanceOptions = InstanceOptions {
    gas_limit: DEFAULT_GAS_LIMIT,
    print_debug: false,
};
// Cache
const MEMORY_CACHE_SIZE: Size = Size::mebi(200);

static CONTRACT: &[u8] = include_bytes!("../testdata/oraichain_nft.wasm");

pub fn main() {
    let options = CacheOptions {
        base_dir: TempDir::new().unwrap().into_path(),
        available_capabilities: HashSet::new(),
        memory_cache_size: MEMORY_CACHE_SIZE,
        instance_memory_limit: DEFAULT_MEMORY_LIMIT,
    };

    let cache = Arc::new(unsafe { Cache::new(options).unwrap() });

    let checksum = cache.save_wasm(CONTRACT).unwrap();
    let mut backend = mock_backend(&[]);
    backend.api = MockApi::new(32);

    let mut instance = cache
        .get_instance(&checksum, backend, DEFAULT_INSTANCE_OPTIONS)
        .unwrap();

    let msg = br#"{"name": "name", "version": "version", "symbol": "symbol","minter":"creator"}"#;
    let env = to_vec(&mock_env()).unwrap();
    let info = to_vec(&mock_info("creator", &coins(1000, "earth"))).unwrap();
    let contract_result = call_instantiate_raw::<_, _, _>(&mut instance, &env, &info, msg).unwrap();
    println!(
        "Done instantiating contract: {}",
        String::from_utf8(contract_result).unwrap()
    );

    let env = to_vec(&mock_env()).unwrap();
    let info = to_vec(&mock_info("creator", &coins(15, "earth"))).unwrap();
    let msg = br#"{"mint":{"token_id": "token_id", "owner": "creator", "name": "name", "description": "description", "image": "image"}}"#;
    let contract_result = call_execute_raw::<_, _, _>(&mut instance, &env, &info, msg).unwrap();
    println!(
        "Done excuting contract: {}",
        String::from_utf8(contract_result).unwrap()
    );
}
