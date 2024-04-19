use criterion::{
    black_box, criterion_group, criterion_main, measurement::Measurement, BenchmarkId, Criterion,
    Throughput,
};
use criterion_inverted_throughput::InvertedThroughput;

use rand::Rng;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::{fs, thread};
use tempfile::TempDir;

use cosmwasm_std::{coins, Checksum, Empty};
use cosmwasm_vm::testing::{
    mock_backend, mock_env, mock_info, mock_instance_options, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_vm::{
    call_execute, call_instantiate, capabilities_from_csv, Cache, CacheOptions, Instance,
    InstanceOptions, Size,
};

// Instance
const DEFAULT_MEMORY_LIMIT: Size = Size::mebi(64);
const DEFAULT_GAS_LIMIT: u64 = 1_000_000_000; // ~1ms
const DEFAULT_INSTANCE_OPTIONS: InstanceOptions = InstanceOptions {
    gas_limit: DEFAULT_GAS_LIMIT,
};
const HIGH_GAS_LIMIT: u64 = 20_000_000_000_000; // ~20s, allows many calls on one instance

// Cache
const MEMORY_CACHE_SIZE: Size = Size::mebi(200);

// Multi-threaded get_instance benchmark
const INSTANTIATION_THREADS: usize = 128;
const CONTRACTS: u64 = 10;

const DEFAULT_CAPABILITIES: &str = "cosmwasm_1_1,cosmwasm_1_2,cosmwasm_1_3,iterator,staking";
static HACKATOM: &[u8] = include_bytes!("../testdata/hackatom.wasm");
static CYBERPUNK: &[u8] = include_bytes!("../testdata/cyberpunk.wasm");
static SHA1: &[u8] = include_bytes!("../testdata/bench_sha1.wasm");

static BENCH_CONTRACTS: &[&str] = &[
    "cyberpunk_rust170.wasm",
    "cyberpunk.wasm",
    "floaty_1.0.wasm",
    "floaty_1.2.wasm",
    "floaty_2.0.wasm",
    "hackatom_1.0.wasm",
    "hackatom_1.2.wasm",
    "hackatom.wasm",
];

fn bench_instance(c: &mut Criterion) {
    let mut group = c.benchmark_group("Instance");

    group.bench_function("compile and instantiate", |b| {
        b.iter(|| {
            let backend = mock_backend(&[]);
            let (instance_options, memory_limit) = mock_instance_options();
            let _instance =
                Instance::from_code(HACKATOM, backend, instance_options, memory_limit).unwrap();
        });
    });

    group.bench_function("execute init", |b| {
        let backend = mock_backend(&[]);
        let much_gas: InstanceOptions = InstanceOptions {
            gas_limit: HIGH_GAS_LIMIT,
        };
        let mut instance =
            Instance::from_code(HACKATOM, backend, much_gas, Some(DEFAULT_MEMORY_LIMIT)).unwrap();

        b.iter(|| {
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
        });
    });

    group.bench_function("execute execute (release)", |b| {
        let backend = mock_backend(&[]);
        let much_gas: InstanceOptions = InstanceOptions {
            gas_limit: HIGH_GAS_LIMIT,
        };
        let mut instance =
            Instance::from_code(HACKATOM, backend, much_gas, Some(DEFAULT_MEMORY_LIMIT)).unwrap();

        let info = mock_info(&instance.api().addr_make("creator"), &coins(1000, "earth"));
        let verifier = instance.api().addr_make("verifies");
        let beneficiary = instance.api().addr_make("benefits");
        let msg = format!(r#"{{"verifier": "{verifier}", "beneficiary": "{beneficiary}"}}"#);
        let contract_result =
            call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg.as_bytes())
                .unwrap();
        assert!(contract_result.into_result().is_ok());

        b.iter(|| {
            let info = mock_info(&verifier, &coins(15, "earth"));
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
        };
        let mut instance =
            Instance::from_code(CYBERPUNK, backend, much_gas, Some(DEFAULT_MEMORY_LIMIT)).unwrap();

        let info = mock_info("creator", &coins(1000, "earth"));
        let contract_result =
            call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, b"{}").unwrap();
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
        println!("Gas used: {gas_used}");
    });

    group.finish();
}

fn bench_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("Cache");

    let options = CacheOptions::new(
        TempDir::new().unwrap().into_path(),
        capabilities_from_csv(DEFAULT_CAPABILITIES),
        MEMORY_CACHE_SIZE,
        DEFAULT_MEMORY_LIMIT,
    );

    group.bench_function("save wasm", |b| {
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options.clone()).unwrap() };

        b.iter(|| {
            let result = cache.save_wasm(HACKATOM);
            assert!(result.is_ok());
        });
    });

    group.bench_function("load wasm", |b| {
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options.clone()).unwrap() };
        let checksum = cache.save_wasm(HACKATOM).unwrap();

        b.iter(|| {
            let result = cache.load_wasm(&checksum);
            assert!(result.is_ok());
        });
    });

    group.bench_function("load wasm unchecked", |b| {
        let options = options.clone();
        let mut cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options).unwrap() };
        cache.set_module_unchecked(true);
        let checksum = cache.save_wasm(HACKATOM).unwrap();

        b.iter(|| {
            let result = cache.load_wasm(&checksum);
            assert!(result.is_ok());
        });
    });

    for contract_name in BENCH_CONTRACTS {
        let contract_wasm = fs::read(format!("testdata/{contract_name}")).unwrap();
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options.clone()).unwrap() };
        let checksum = cache.save_wasm(&contract_wasm).unwrap();

        group.bench_function(format!("analyze_{contract_name}"), |b| {
            b.iter(|| {
                let result = cache.analyze(&checksum);
                assert!(result.is_ok());
            });
        });
    }

    group.bench_function("instantiate from fs", |b| {
        let non_memcache = CacheOptions::new(
            TempDir::new().unwrap().into_path(),
            capabilities_from_csv(DEFAULT_CAPABILITIES),
            Size::new(0),
            DEFAULT_MEMORY_LIMIT,
        );
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(non_memcache).unwrap() };
        let checksum = cache.save_wasm(HACKATOM).unwrap();

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

    group.bench_function("instantiate from fs unchecked", |b| {
        let non_memcache = CacheOptions::new(
            TempDir::new().unwrap().into_path(),
            capabilities_from_csv(DEFAULT_CAPABILITIES),
            Size::new(0),
            DEFAULT_MEMORY_LIMIT,
        );
        let mut cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(non_memcache).unwrap() };
        cache.set_module_unchecked(true);
        let checksum = cache.save_wasm(HACKATOM).unwrap();

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
        let checksum = Checksum::generate(HACKATOM);
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
        let checksum = Checksum::generate(HACKATOM);
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

fn bench_instance_threads(c: &mut Criterion) {
    c.bench_function("multi-threaded get_instance", |b| {
        let options = CacheOptions::new(
            TempDir::new().unwrap().into_path(),
            capabilities_from_csv(DEFAULT_CAPABILITIES),
            MEMORY_CACHE_SIZE,
            DEFAULT_MEMORY_LIMIT,
        );

        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options).unwrap() };
        let cache = Arc::new(cache);

        // Find sub-sequence helper
        fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
            haystack
                .windows(needle.len())
                .position(|window| window == needle)
        }

        // Offset to the i32.const (0x41) 15731626 (0xf00baa) (unsigned leb128 encoded) instruction
        // data we want to replace
        let query_int_data = b"\x41\xaa\x97\xc0\x07";
        let offset = find_subsequence(HACKATOM, query_int_data).unwrap() + 1;

        let mut leb128_buf = [0; 4];
        let mut contract = HACKATOM.to_vec();

        let mut random_checksum = || {
            let mut writable = &mut leb128_buf[..];

            // Generates a random number in the range of a 4-byte unsigned leb128 encoded number
            let r = rand::thread_rng().gen_range(2097152..2097152 + CONTRACTS);

            leb128::write::unsigned(&mut writable, r).expect("Should write number");

            // Splice data in contract
            contract.splice(offset..offset + leb128_buf.len(), leb128_buf);

            cache.save_wasm(contract.as_slice()).unwrap()
            // let checksum = cache.save_wasm(contract.as_slice()).unwrap();
            // Preload into memory
            // cache
            //     .get_instance(&checksum, mock_backend(&[]), DEFAULT_INSTANCE_OPTIONS)
            //     .unwrap();
            // checksum
        };

        b.iter_custom(|iters| {
            let mut res = Duration::from_secs(0);
            for _ in 0..iters {
                let mut durations: Vec<_> = (0..INSTANTIATION_THREADS)
                    .map(|_id| {
                        let cache = Arc::clone(&cache);
                        let checksum = random_checksum();

                        thread::spawn(move || {
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

fn bench_combined(c: &mut Criterion) {
    let mut group = c.benchmark_group("Combined");

    let options = CacheOptions::new(
        TempDir::new().unwrap().into_path(),
        capabilities_from_csv("cosmwasm_1_1,cosmwasm_1_2,cosmwasm_1_3,iterator,staking"),
        MEMORY_CACHE_SIZE,
        DEFAULT_MEMORY_LIMIT,
    );

    // Store contracts for all benchmarks in this group
    let checksum: Checksum = {
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options.clone()).unwrap() };
        cache.save_wasm(CYBERPUNK).unwrap()
    };

    group.bench_function("get instance from fs cache and execute", |b| {
        let mut non_memcache = options.clone();
        non_memcache.memory_cache_size = Size::kibi(0);

        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(non_memcache).unwrap() };

        b.iter(|| {
            let mut instance = cache
                .get_instance(&checksum, mock_backend(&[]), DEFAULT_INSTANCE_OPTIONS)
                .unwrap();
            assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
            assert_eq!(cache.stats().hits_memory_cache, 0);
            assert!(cache.stats().hits_fs_cache >= 1);
            assert_eq!(cache.stats().misses, 0);

            let info = mock_info("guest", &[]);
            let msg = br#"{"noop":{}}"#;
            let contract_result =
                call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
            contract_result.into_result().unwrap();
        });
    });

    group.bench_function("get instance from memory cache and execute", |b| {
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options.clone()).unwrap() };

        // Load into memory
        cache
            .get_instance(&checksum, mock_backend(&[]), DEFAULT_INSTANCE_OPTIONS)
            .unwrap();

        b.iter(|| {
            let backend = mock_backend(&[]);
            let mut instance = cache
                .get_instance(&checksum, backend, DEFAULT_INSTANCE_OPTIONS)
                .unwrap();
            assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
            assert!(cache.stats().hits_memory_cache >= 1);
            assert_eq!(cache.stats().hits_fs_cache, 1);
            assert_eq!(cache.stats().misses, 0);

            let info = mock_info("guest", &[]);
            let msg = br#"{"noop":{}}"#;
            let contract_result =
                call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
            contract_result.into_result().unwrap();
        });
    });

    group.bench_function("get instance from pinned memory and execute", |b| {
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options.clone()).unwrap() };

        // Load into pinned memory
        cache.pin(&checksum).unwrap();

        b.iter(|| {
            let backend = mock_backend(&[]);
            let mut instance = cache
                .get_instance(&checksum, backend, DEFAULT_INSTANCE_OPTIONS)
                .unwrap();
            assert_eq!(cache.stats().hits_memory_cache, 0);
            assert!(cache.stats().hits_pinned_memory_cache >= 1);
            assert_eq!(cache.stats().hits_fs_cache, 1);
            assert_eq!(cache.stats().misses, 0);

            let info = mock_info("guest", &[]);
            let msg = br#"{"noop":{}}"#;
            let contract_result =
                call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
            contract_result.into_result().unwrap();
        });
    });

    group.finish();
}

fn bench_sha1<M: Measurement>(c: &mut Criterion<M>) {
    let mut group = c.benchmark_group("sha1");

    // bytes, blocks, used_gas
    let mut gas_reports_wasm: Vec<(u64, u64, u64)> = vec![];
    let mut gas_reports_api: Vec<(u64, u64, u64)> = vec![];

    for i in 6..=16 {
        // Every 64 bytes needs a hassing and there are
        // 8 bytes header (message length) and 1 byte tail (EOF).
        let len = u64::pow(2, i) - 9;
        let block = (len + 9) / 64;
        group.throughput(Throughput::Elements(block));
        group.bench_function(BenchmarkId::new("WASM/blocks", block), |b| {
            let backend = mock_backend(&[]);
            let much_gas: InstanceOptions = InstanceOptions {
                gas_limit: HIGH_GAS_LIMIT,
            };
            let mut instance =
                Instance::from_code(SHA1, backend, much_gas, Some(DEFAULT_MEMORY_LIMIT)).unwrap();

            let info = mock_info("creator", &coins(1000, "earth"));
            let msg = br#"{}"#;
            let contract_result =
                call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
            contract_result.into_result().unwrap();

            let mut gas_used = 0;
            b.iter(|| {
                let gas_before = instance.get_gas_left();
                let info = mock_info("foo", &[]);
                let msg = format!(r#"{{"wasm":{{"len":{}}}}}"#, len).into_bytes();
                let contract_result =
                    call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, &msg)
                        .unwrap();
                contract_result.into_result().unwrap();
                gas_used = gas_before - instance.get_gas_left();
            });
            if gas_reports_wasm.len() < (i - 5) as usize {
                gas_reports_wasm.push((len, block, gas_used))
            }
        });

        group.bench_function(BenchmarkId::new("API/blocks", block), |b| {
            let backend = mock_backend(&[]);
            let much_gas: InstanceOptions = InstanceOptions {
                gas_limit: HIGH_GAS_LIMIT,
            };
            let mut instance =
                Instance::from_code(SHA1, backend, much_gas, Some(DEFAULT_MEMORY_LIMIT)).unwrap();

            let info = mock_info("creator", &coins(1000, "earth"));
            let msg = br#"{}"#;
            let contract_result =
                call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
            contract_result.into_result().unwrap();

            let mut gas_used = 0;
            b.iter(|| {
                let gas_before = instance.get_gas_left();
                let info = mock_info("foo", &[]);
                let msg = format!(r#"{{"api":{{"len":{}}}}}"#, len).into_bytes();
                let contract_result =
                    call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, &msg)
                        .unwrap();
                contract_result.into_result().unwrap();
                gas_used = gas_before - instance.get_gas_left();
            });
            if gas_reports_api.len() < (i - 5) as usize {
                gas_reports_api.push((len, block, gas_used))
            }
        });
    }
    for i in 0..gas_reports_wasm.len() {
        let wasm = gas_reports_wasm[i];
        let api = gas_reports_api[i];
        println!("gas wasm/{}bytes({}blocks): {}", wasm.0, wasm.1, wasm.2);
        println!("gas api /{}bytes({}blocks): {}", api.0, api.1, api.2);
    }

    group.finish();
}

fn make_config(measurement_time_s: u64) -> Criterion {
    Criterion::default()
        .without_plots()
        .measurement_time(Duration::new(measurement_time_s, 0))
        .sample_size(12)
        .configure_from_args()
}

fn make_inversion_throughput_config() -> Criterion<InvertedThroughput> {
    Criterion::default()
        .with_measurement(InvertedThroughput::new())
        .without_plots()
        .measurement_time(Duration::new(10, 0))
        .sample_size(12)
        .configure_from_args()
}

criterion_group!(
    name = instance;
    config = make_config(8);
    targets = bench_instance
);
criterion_group!(
    name = cache;
    config = make_config(8);
    targets = bench_cache
);
// Combines loading module from cache, instantiating it and executing the instance.
// This is what every call in libwasmvm does.
criterion_group!(
    name = combined;
    config = make_config(5);
    targets = bench_combined
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
criterion_group!(
    name = sha1;
    config = make_inversion_throughput_config();
    targets = bench_sha1
);
criterion_main!(instance, cache, combined, multi_threaded_instance, sha1);
