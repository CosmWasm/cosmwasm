use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, PlottingBackend};
use std::time::Duration;

use cosmwasm_std::{coins, to_json_binary, Empty};
use cosmwasm_vm::testing::{mock_backend, mock_env, mock_info};
use cosmwasm_vm::{call_execute, call_instantiate, Instance, InstanceOptions, Size};

use cyberpunk::contract::{execute_argon2, execute_drand_verify_g1, execute_drand_verify_g2};
use cyberpunk::msg::ExecuteMsg;

// Compile with `RUSTFLAGS='-C link-arg=-s' cargo wasm` which should be something like 500KB large
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/cyberpunk.wasm");

// Compile with `RUSTFLAGS='-C link-arg=-s' cargo wasm && wasm-opt -O4 target/wasm32-unknown-unknown/release/cyberpunk.wasm -o optimized.wasm`
// static WASM: &[u8] = include_bytes!("../optimized.wasm");

/* Compile with:
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_cyberpunk",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer-arm64:0.15.0 ./contracts/cyberpunk
 */
//static WASM: &[u8] = include_bytes!("../../../artifacts/cyberpunk-aarch64.wasm");

const DEFAULT_MEMORY_LIMIT: Size = Size::mebi(128);
const HIGH_GAS_LIMIT: u64 = 20_000_000_000_000; // ~20s

#[derive(Clone, Debug)]
struct Argon2Config {
    name: &'static str,
    mem_cost: u32,
    time_cost: u32,
}

fn bench_argon2(c: &mut Criterion) {
    let mut group = c.benchmark_group("argon2");

    let low_mem = Argon2Config {
        name: "low_mem",
        mem_cost: 64, // 2**6
        time_cost: 3,
    };
    let medium_mem = Argon2Config {
        name: "medium_mem",
        mem_cost: 2048, // 2**11
        time_cost: 2,
    };
    let high_mem = Argon2Config {
        name: "high_mem",
        mem_cost: 65536, // 2**16
        time_cost: 1,
    };

    let configs = vec![low_mem, medium_mem, high_mem];

    for config in configs.clone() {
        group.bench_with_input(
            BenchmarkId::new("host", config.name),
            &config,
            |b, config| {
                b.iter(|| {
                    let res = execute_argon2(config.mem_cost, config.time_cost).unwrap();
                    assert!(res.data.unwrap_or_default().len() > 0);
                });
            },
        );
    }

    for config in configs.clone() {
        group.bench_with_input(
            BenchmarkId::new("wasm", config.name),
            &config,
            |b, config| {
                let backend = mock_backend(&[]);
                let much_gas: InstanceOptions = InstanceOptions {
                    gas_limit: HIGH_GAS_LIMIT,
                };
                let mut instance =
                    Instance::from_code(WASM, backend, much_gas, Some(DEFAULT_MEMORY_LIMIT))
                        .unwrap();

                let info = mock_info("creator", &coins(1000, "earth"));
                let contract_result =
                    call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, b"{}")
                        .unwrap();
                assert!(contract_result.into_result().is_ok());

                let msg = to_json_binary(&ExecuteMsg::Argon2 {
                    mem_cost: config.mem_cost,
                    time_cost: config.time_cost,
                })
                .unwrap();

                let env = mock_env();
                let info = mock_info("hasher", &[]);
                b.iter(|| {
                    let res =
                        call_execute::<_, _, _, Empty>(&mut instance, &env, &info, &msg).unwrap();
                    assert!(res.into_result().is_ok());
                });
            },
        );
    }

    group.finish();
}

fn bench_drand_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("drand_verify");

    group.bench_function(BenchmarkId::new("host", "g1"), |b| {
        b.iter(|| {
            let response = execute_drand_verify_g1().unwrap();
            assert_eq!(response.data.unwrap_or_default(), [0x01]);
        });
    });

    group.bench_function(BenchmarkId::new("host", "g2"), |b| {
        b.iter(|| {
            let response = execute_drand_verify_g2().unwrap();
            assert_eq!(response.data.unwrap_or_default(), [0x01]);
        });
    });

    group.bench_function(BenchmarkId::new("wasm", "g1"), |b| {
        let backend = mock_backend(&[]);
        let much_gas: InstanceOptions = InstanceOptions {
            gas_limit: HIGH_GAS_LIMIT,
        };
        let mut instance =
            Instance::from_code(WASM, backend, much_gas, Some(DEFAULT_MEMORY_LIMIT)).unwrap();

        let info = mock_info("creator", &coins(1000, "earth"));
        let contract_result =
            call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, b"{}").unwrap();
        assert!(contract_result.into_result().is_ok());

        let msg = to_json_binary(&ExecuteMsg::DrandVerifyG1 {}).unwrap();

        let env = mock_env();
        let info = mock_info("hasher", &[]);
        b.iter(|| {
            let response = call_execute::<_, _, _, Empty>(&mut instance, &env, &info, &msg)
                .unwrap()
                .unwrap();
            assert_eq!(response.data.unwrap_or_default(), [0x01]);
        });
    });

    group.bench_function(BenchmarkId::new("wasm", "g2"), |b| {
        let backend = mock_backend(&[]);
        let much_gas: InstanceOptions = InstanceOptions {
            gas_limit: HIGH_GAS_LIMIT,
        };
        let mut instance =
            Instance::from_code(WASM, backend, much_gas, Some(DEFAULT_MEMORY_LIMIT)).unwrap();

        let info = mock_info("creator", &coins(1000, "earth"));
        let contract_result =
            call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, b"{}").unwrap();
        assert!(contract_result.into_result().is_ok());

        let msg = to_json_binary(&ExecuteMsg::DrandVerifyG2 {}).unwrap();

        let env = mock_env();
        let info = mock_info("hasher", &[]);
        b.iter(|| {
            let response = call_execute::<_, _, _, Empty>(&mut instance, &env, &info, &msg)
                .unwrap()
                .unwrap();
            assert_eq!(response.data.unwrap_or_default(), [0x01]);
        });
    });

    group.finish();
}

fn make_config() -> Criterion {
    Criterion::default()
        .plotting_backend(PlottingBackend::Plotters)
        .without_plots()
        .warm_up_time(Duration::new(1, 0))
        .measurement_time(Duration::new(6, 0))
        // Increase for higher precision results
        // .warm_up_time(Duration::new(4, 0))
        // .measurement_time(Duration::new(10, 0))
        .sample_size(10)
        .configure_from_args()
}

criterion_group!(
    name = argon2;
    config = make_config();
    targets = bench_argon2
);
criterion_group!(
    name = drand_verify;
    config = make_config();
    targets = bench_drand_verify
);
criterion_main!(argon2, drand_verify);
