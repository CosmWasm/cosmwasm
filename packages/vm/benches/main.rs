use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::seq::SliceRandom;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

use cosmwasm_std::{coins, Empty};
use cosmwasm_vm::testing::{
    mock_backend, mock_env, mock_info, mock_instance_options, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_vm::{
    call_execute, call_instantiate, features_from_csv, Cache, CacheOptions, Checksum, Instance,
    InstanceOptions, Size,
};

// Instance
const DEFAULT_MEMORY_LIMIT: Size = Size::mebi(64);
const DEFAULT_GAS_LIMIT: u64 = 1_000_000_000_000; // ~1ms
const DEFAULT_INSTANCE_OPTIONS: InstanceOptions = InstanceOptions {
    gas_limit: DEFAULT_GAS_LIMIT,
    print_debug: false,
};
const HIGH_GAS_LIMIT: u64 = 20_000_000_000_000_000; // ~20s, allows many calls on one instance

// Cache
const MEMORY_CACHE_SIZE: Size = Size::mebi(200);

// Multi-threaded get_instance benchmark
const INSTANTIATION_THREADS: usize = 1024;

static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");
static CONTRACTS: &[&[u8]] = &[CONTRACT, include_bytes!("../testdata/ibc_reflect.wasm")];

fn bench_instance(c: &mut Criterion) {
    let mut group = c.benchmark_group("Instance");

    group.bench_function("compile and instantiate", |b| {
        b.iter(|| {
            let backend = mock_backend(&[]);
            let (instance_options, memory_limit) = mock_instance_options();
            let _instance =
                Instance::from_code(CONTRACT, backend, instance_options, memory_limit).unwrap();
        });
    });

    group.bench_function("execute init", |b| {
        let backend = mock_backend(&[]);
        let much_gas: InstanceOptions = InstanceOptions {
            gas_limit: HIGH_GAS_LIMIT,
            ..DEFAULT_INSTANCE_OPTIONS
        };
        let mut instance =
            Instance::from_code(CONTRACT, backend, much_gas, Some(DEFAULT_MEMORY_LIMIT)).unwrap();

        b.iter(|| {
            let info = mock_info("creator", &coins(1000, "earth"));
            let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
            let contract_result =
                call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
            assert!(contract_result.into_result().is_ok());
        });
    });

    group.bench_function("execute execute (release)", |b| {
        let backend = mock_backend(&[]);
        let much_gas: InstanceOptions = InstanceOptions {
            gas_limit: HIGH_GAS_LIMIT,
            ..DEFAULT_INSTANCE_OPTIONS
        };
        let mut instance =
            Instance::from_code(CONTRACT, backend, much_gas, Some(DEFAULT_MEMORY_LIMIT)).unwrap();

        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
        let contract_result =
            call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
        assert!(contract_result.into_result().is_ok());

        b.iter(|| {
            let info = mock_info("verifies", &coins(15, "earth"));
            let msg = br#"{"release":{}}"#;
            let contract_result =
                call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
            assert!(contract_result.into_result().is_ok());
        });
    });

    group.bench_function("execute execute (argon2)", |b| {
        let backend = mock_backend(&[]);
        let much_gas: InstanceOptions = InstanceOptions {
            gas_limit: HIGH_GAS_LIMIT,
            ..DEFAULT_INSTANCE_OPTIONS
        };
        let mut instance =
            Instance::from_code(CONTRACT, backend, much_gas, Some(DEFAULT_MEMORY_LIMIT)).unwrap();

        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
        let contract_result =
            call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
        assert!(contract_result.into_result().is_ok());

        let mut gas_used = 0;
        b.iter(|| {
            let gas_before = instance.get_gas_left();
            let info = mock_info("hasher", &[]);
            let msg = br#"{"argon2":{"mem_cost":256,"time_cost":3}}"#;
            let contract_result =
                call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
            assert!(contract_result.into_result().is_ok());
            gas_used = gas_before - instance.get_gas_left();
        });
        println!("Gas used: {}", gas_used);
    });

    group.finish();
}

fn bench_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("Cache");

    let options = CacheOptions {
        base_dir: TempDir::new().unwrap().into_path(),
        supported_features: features_from_csv("iterator,staking"),
        memory_cache_size: MEMORY_CACHE_SIZE,
        instance_memory_limit: DEFAULT_MEMORY_LIMIT,
    };

    group.bench_function("save wasm", |b| {
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options.clone()).unwrap() };

        b.iter(|| {
            let result = cache.save_wasm(CONTRACT);
            assert!(result.is_ok());
        });
    });

    group.bench_function("load wasm", |b| {
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options.clone()).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        b.iter(|| {
            let result = cache.load_wasm(&checksum);
            assert!(result.is_ok());
        });
    });

    group.bench_function("analyze", |b| {
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options.clone()).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        b.iter(|| {
            let result = cache.analyze(&checksum);
            assert!(result.is_ok());
        });
    });

    group.bench_function("instantiate from fs", |b| {
        let non_memcache = CacheOptions {
            base_dir: TempDir::new().unwrap().into_path(),
            supported_features: features_from_csv("iterator,staking"),
            memory_cache_size: Size(0),
            instance_memory_limit: DEFAULT_MEMORY_LIMIT,
        };
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(non_memcache).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        b.iter(|| {
            let _ = cache
                .get_instance(&checksum, mock_backend(&[]), DEFAULT_INSTANCE_OPTIONS)
                .unwrap();
            assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
            assert_eq!(cache.stats().hits_memory_cache, 0);
            assert!(cache.stats().hits_fs_cache >= 1);
            assert_eq!(cache.stats().misses, 0);
        });
    });

    group.bench_function("instantiate from memory", |b| {
        let checksum = Checksum::generate(CONTRACT);
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options.clone()).unwrap() };
        // Load into memory
        cache
            .get_instance(&checksum, mock_backend(&[]), DEFAULT_INSTANCE_OPTIONS)
            .unwrap();

        b.iter(|| {
            let backend = mock_backend(&[]);
            let _ = cache
                .get_instance(&checksum, backend, DEFAULT_INSTANCE_OPTIONS)
                .unwrap();
            assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
            assert!(cache.stats().hits_memory_cache >= 1);
            assert_eq!(cache.stats().hits_fs_cache, 1);
            assert_eq!(cache.stats().misses, 0);
        });
    });

    group.bench_function("instantiate from pinned memory", |b| {
        let checksum = Checksum::generate(CONTRACT);
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options.clone()).unwrap() };
        // Load into pinned memory
        cache.pin(&checksum).unwrap();

        b.iter(|| {
            let backend = mock_backend(&[]);
            let _ = cache
                .get_instance(&checksum, backend, DEFAULT_INSTANCE_OPTIONS)
                .unwrap();
            assert_eq!(cache.stats().hits_memory_cache, 0);
            assert!(cache.stats().hits_pinned_memory_cache >= 1);
            assert_eq!(cache.stats().hits_fs_cache, 1);
            assert_eq!(cache.stats().misses, 0);
        });
    });

    group.finish();
}

pub fn bench_instance_threads(c: &mut Criterion) {
    c.bench_function("multi-threaded get_instance", |b| {
        let options = CacheOptions {
            base_dir: TempDir::new().unwrap().into_path(),
            supported_features: features_from_csv("iterator,staking,stargate"),
            memory_cache_size: MEMORY_CACHE_SIZE,
            instance_memory_limit: DEFAULT_MEMORY_LIMIT,
        };

        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options).unwrap() };
        let cache = Arc::new(cache);

        let checksums: Vec<_> = CONTRACTS
            .iter()
            .map(|&wasm| cache.save_wasm(wasm).unwrap())
            .collect();

        let random_checksum = || *checksums.choose(&mut rand::thread_rng()).unwrap();

        b.iter_custom(|iters| {
            let mut res = Duration::from_secs(0);
            for _ in 0..iters {
                let mut durations: Vec<_> = (0..INSTANTIATION_THREADS)
                    .map(|_id| {
                        let cache = Arc::clone(&cache);
                        let checksum = random_checksum();

                        thread::spawn(move || {
                            let checksum = checksum;
                            // Perform measurement internally
                            let t = SystemTime::now();
                            black_box(
                                cache
                                    .get_instance(
                                        &checksum,
                                        mock_backend(&[]),
                                        DEFAULT_INSTANCE_OPTIONS,
                                    )
                                    .unwrap(),
                            );
                            t.elapsed().unwrap()
                        })
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|handle| handle.join().unwrap())
                    .collect(); // join threads, collect durations

                // Calculate median thread duration
                durations.sort_unstable();
                res += durations[durations.len() / 2];
            }
            res
        });
    });
}

fn make_config() -> Criterion {
    Criterion::default()
        .without_plots()
        .measurement_time(Duration::new(10, 0))
        .sample_size(12)
        .configure_from_args()
}

criterion_group!(
    name = instance;
    config = make_config();
    targets = bench_instance
);
criterion_group!(
    name = cache;
    config = make_config();
    targets = bench_cache
);
criterion_group!(
    name = multi_threaded_instance;
    config = Criterion::default()
        .without_plots()
        .measurement_time(Duration::new(16, 0))
        .sample_size(10)
        .configure_from_args();
    targets = bench_instance_threads
);
criterion_main!(instance, cache, multi_threaded_instance);
