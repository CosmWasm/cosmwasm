use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, PlottingBackend};
use std::time::Duration;

use cosmwasm_std::{coins, to_json_binary, Empty};
use cosmwasm_vm::testing::{mock_backend, mock_env, mock_info};
use cosmwasm_vm::{call_execute, call_instantiate, Instance, InstanceOptions, Size};

use cyberpunk::contract::execute_argon2;
use cyberpunk::msg::ExecuteMsg;

// Compile with `RUSTFLAGS='-C link-arg=-s' cargo wasm` which should be something like 240KB large
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/cyberpunk.wasm");

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

fn make_config() -> Criterion {
    Criterion::default()
        .plotting_backend(PlottingBackend::Plotters)
        .without_plots()
        .warm_up_time(Duration::new(1, 0))
        .measurement_time(Duration::new(6, 0))
        // Increase for higher precision results
        // .warm_up_time(Duration::new(4, 0))
        // .measurement_time(Duration::new(10, 0))
        .configure_from_args()
}

criterion_group!(
    name = argon2;
    config = make_config();
    targets = bench_argon2
);
criterion_main!(argon2);
