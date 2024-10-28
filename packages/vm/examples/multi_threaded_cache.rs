use std::sync::Arc;
use std::thread;
use tempfile::TempDir;

use cosmwasm_std::{coins, Empty};
use cosmwasm_vm::testing::{mock_backend, mock_env, mock_info, MockApi, MockQuerier, MockStorage};
use cosmwasm_vm::{
    call_execute, call_instantiate, capabilities_from_csv, Cache, CacheOptions, InstanceOptions,
    Size,
};

// Instance
const DEFAULT_MEMORY_LIMIT: Size = Size::mebi(64);
const DEFAULT_GAS_LIMIT: u64 = 400_000 * 150;
const DEFAULT_INSTANCE_OPTIONS: InstanceOptions = InstanceOptions {
    gas_limit: DEFAULT_GAS_LIMIT,
};
// Cache
const MEMORY_CACHE_SIZE: Size = Size::mebi(200);

static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");

const STORE_CODE_THREADS: usize = 32;
const INSTANTIATION_THREADS: usize = 2048;
const THREADS: usize = STORE_CODE_THREADS + INSTANTIATION_THREADS;

pub fn main() {
    let options = CacheOptions::new(
        TempDir::new().unwrap().into_path(),
        capabilities_from_csv("iterator,staking"),
        MEMORY_CACHE_SIZE,
        DEFAULT_MEMORY_LIMIT,
    );

    let cache: Cache<MockApi, MockStorage, MockQuerier> = unsafe { Cache::new(options).unwrap() };
    let cache = Arc::new(cache);

    let checksum = cache.store_code(CONTRACT, true, true).unwrap();

    let mut threads = Vec::with_capacity(THREADS);
    for _ in 0..STORE_CODE_THREADS {
        let cache = Arc::clone(&cache);

        threads.push(thread::spawn(move || {
            let checksum = cache.store_code(CONTRACT, true, true).unwrap();
            println!("Done saving Wasm {checksum}");
        }));
    }
    for i in 0..INSTANTIATION_THREADS {
        let cache = Arc::clone(&cache);

        threads.push(thread::spawn(move || {
            let mut instance = cache
                .get_instance(&checksum, mock_backend(&[]), DEFAULT_INSTANCE_OPTIONS)
                .unwrap();
            println!("Done instantiating contract {i}");

            let info = mock_info(&instance.api().addr_make("creator"), &coins(1000, "earth"));
            let verifier = instance.api().addr_make("verifies");
            let beneficiary = instance.api().addr_make("benefits");
            let msg = format!(r#"{{"verifier": "{verifier}", "beneficiary": "{beneficiary}"}}"#);
            let contract_result = call_instantiate::<_, _, _, Empty>(
                &mut instance,
                &mock_env(),
                &info,
                msg.as_bytes(),
            )
            .unwrap();
            assert!(contract_result.into_result().is_ok());

            let info = mock_info(&verifier, &coins(15, "earth"));
            let msg = br#"{"release":{}}"#;
            let contract_result =
                call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
            assert!(contract_result.into_result().is_ok());
        }));
    }

    threads.into_iter().for_each(|thread| {
        thread
            .join()
            .expect("The threaded instantiation or execution failed !")
    });

    assert_eq!(cache.stats().misses, 0);
    assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
    assert_eq!(
        cache.stats().hits_memory_cache,
        INSTANTIATION_THREADS as u32 - 1
    );
    assert_eq!(cache.stats().hits_fs_cache, 1);
}
