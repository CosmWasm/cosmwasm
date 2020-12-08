use criterion::{criterion_group, criterion_main, Criterion, PlottingBackend};
use std::time::Duration;
use tempfile::TempDir;

use cosmwasm_std::{coins, Empty};
use cosmwasm_vm::testing::{
    mock_backend, mock_env, mock_info, mock_instance_options, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_vm::{
    call_handle, call_init, features_from_csv, Cache, CacheOptions, Checksum, Instance,
    InstanceOptions, Size,
};

const MEMORY_CACHE_SIZE: Size = Size::mebi(200);
const GAS_LIMIT: u64 = 400_000;
const INSTANCE_OPTIONS: InstanceOptions = InstanceOptions {
    gas_limit: GAS_LIMIT,
    print_debug: false,
};

static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

fn bench_instance(c: &mut Criterion) {
    let mut group = c.benchmark_group("Instance");

    group.bench_function("compile and instantiate", |b| {
        b.iter(|| {
            let backend = mock_backend(&[]);
            let _instance =
                Instance::from_code(CONTRACT, backend, mock_instance_options()).unwrap();
        });
    });

    group.bench_function("execute init", |b| {
        let backend = mock_backend(&[]);
        let much_gas: InstanceOptions = InstanceOptions {
            gas_limit: 5_000_000_000,
            print_debug: false,
        };
        let mut instance = Instance::from_code(CONTRACT, backend, much_gas).unwrap();

        b.iter(|| {
            let info = mock_info("creator", &coins(1000, "earth"));
            let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
            let contract_result =
                call_init::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
            assert!(contract_result.into_result().is_ok());
        });
    });

    group.bench_function("execute handle", |b| {
        let backend = mock_backend(&[]);
        let much_gas: InstanceOptions = InstanceOptions {
            gas_limit: 5_000_000_000,
            print_debug: false,
        };
        let mut instance = Instance::from_code(CONTRACT, backend, much_gas).unwrap();

        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
        let contract_result =
            call_init::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
        assert!(contract_result.into_result().is_ok());

        b.iter(|| {
            let info = mock_info("verifies", &coins(15, "earth"));
            let msg = br#"{"release":{}}"#;
            let contract_result =
                call_handle::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
            assert!(contract_result.into_result().is_ok());
        });
    });

    group.finish();
}

fn bench_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("Cache");

    let options = CacheOptions {
        base_dir: TempDir::new().unwrap().into_path(),
        supported_features: features_from_csv("staking"),
        memory_cache_size: MEMORY_CACHE_SIZE,
    };

    group.bench_function("save wasm", |b| {
        let mut cache: Cache<MockStorage, MockApi, MockQuerier> =
            unsafe { Cache::new(options.clone()).unwrap() };

        b.iter(|| {
            let result = cache.save_wasm(CONTRACT);
            assert!(result.is_ok());
        });
    });

    group.bench_function("instantiate from fs", |b| {
        let non_memcache = CacheOptions {
            base_dir: TempDir::new().unwrap().into_path(),
            supported_features: features_from_csv("staking"),
            memory_cache_size: Size(0),
        };
        let mut cache: Cache<MockStorage, MockApi, MockQuerier> =
            unsafe { Cache::new(non_memcache).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        b.iter(|| {
            let _ = cache
                .get_instance(&checksum, mock_backend(&[]), INSTANCE_OPTIONS)
                .unwrap();
            assert_eq!(cache.stats().hits_memory_cache, 0);
            assert!(cache.stats().hits_fs_cache >= 1);
            assert_eq!(cache.stats().misses, 0);
        });
    });

    group.bench_function("instantiate from memory", |b| {
        let checksum = Checksum::generate(CONTRACT);
        let mut cache: Cache<MockStorage, MockApi, MockQuerier> =
            unsafe { Cache::new(options.clone()).unwrap() };
        // Load into memory
        cache
            .get_instance(&checksum, mock_backend(&[]), INSTANCE_OPTIONS)
            .unwrap();

        b.iter(|| {
            let backend = mock_backend(&[]);
            let _ = cache
                .get_instance(&checksum, backend, INSTANCE_OPTIONS)
                .unwrap();
            assert_eq!(cache.stats().hits_fs_cache, 1);
            assert!(cache.stats().hits_memory_cache >= 1);
            assert_eq!(cache.stats().misses, 0);
        });
    });

    group.finish();
}

fn make_config() -> Criterion {
    Criterion::default()
        .plotting_backend(PlottingBackend::Plotters)
        .without_plots()
        .measurement_time(Duration::new(10, 0))
        .sample_size(12)
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
criterion_main!(instance, cache);
