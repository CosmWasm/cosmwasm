// Run with
// cargo run --features dhat-heap --example heap_profiling --release

use std::time::{Duration, SystemTime};
use tempfile::TempDir;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use cosmwasm_std::{coins, Checksum, Empty};
use cosmwasm_vm::testing::{mock_backend, mock_env, mock_info, MockApi, MockQuerier, MockStorage};
use cosmwasm_vm::{
    call_execute, call_instantiate, capabilities_from_csv, Cache, CacheOptions, InstanceOptions,
    Size,
};

use clap::{Arg, Command};

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

/// Number of seconds after which the test stops
const ROUNDS: usize = 1024;
const ROUND_LEN: usize = 16;

// Instance
const DEFAULT_MEMORY_LIMIT: Size = Size::mebi(64);
const DEFAULT_GAS_LIMIT: u64 = u64::MAX;
const DEFAULT_INSTANCE_OPTIONS: InstanceOptions = InstanceOptions {
    gas_limit: DEFAULT_GAS_LIMIT,
};
// Cache
const MEMORY_CACHE_SIZE: Size = Size::mebi(5);

struct Execute {
    pub msg: &'static [u8],
    pub expect_error: bool,
}

struct Contract {
    pub wasm: &'static [u8],
    pub instantiate_msg: Option<Vec<u8>>,
    pub execute_msgs: Vec<Execute>,
}

fn contracts() -> Vec<Contract> {
    let api = MockApi::default();
    let verifier = api.addr_make("verifies");
    let beneficiary = api.addr_make("benefits");
    vec![
        Contract {
            wasm: include_bytes!("../testdata/cyberpunk.wasm"),
            instantiate_msg: Some(b"{}".to_vec()),
            execute_msgs: vec![
                Execute {
                    msg: br#"{"unreachable":{}}"#,
                    expect_error: true,
                },
                Execute {
                    msg: br#"{"allocate_large_memory":{"pages":1000}}"#,
                    expect_error: false,
                },
                Execute {
                    // mem_cost in KiB
                    msg: br#"{"argon2":{"mem_cost":256,"time_cost":1}}"#,
                    expect_error: false,
                },
                Execute {
                    msg: br#"{"memory_loop":{}}"#,
                    expect_error: true,
                },
            ],
        },
        Contract {
            wasm: include_bytes!("../testdata/hackatom.wasm"),
            instantiate_msg: Some(
                format!(r#"{{"verifier": "{verifier}", "beneficiary": "{beneficiary}"}}"#)
                    .into_bytes(),
            ),
            execute_msgs: vec![Execute {
                msg: br#"{"release":{}}"#,
                expect_error: false,
            }],
        },
        Contract {
            wasm: include_bytes!("../testdata/hackatom_1.0.wasm"),
            instantiate_msg: Some(
                format!(r#"{{"verifier": "{verifier}", "beneficiary": "{beneficiary}"}}"#)
                    .into_bytes(),
            ),
            execute_msgs: vec![Execute {
                msg: br#"{"release":{}}"#,
                expect_error: false,
            }],
        },
        Contract {
            wasm: include_bytes!("../testdata/ibc_reflect.wasm"),
            instantiate_msg: None,
            execute_msgs: vec![],
        },
    ]
}

#[allow(clippy::collapsible_else_if)]
fn app(runtime: u64) {
    let start_time = SystemTime::now();

    let options = CacheOptions::new(
        TempDir::new().unwrap().into_path(),
        capabilities_from_csv("iterator,staking,stargate,cosmwasm_1_1,cosmwasm_1_2,cosmwasm_1_3,cosmwasm_1_4,cosmwasm_2_0,cosmwasm_2_1"),
        MEMORY_CACHE_SIZE,
        DEFAULT_MEMORY_LIMIT,
    );

    let contracts = contracts();

    let checksums = {
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options.clone()).unwrap() };

        let mut checksums = Vec::<Checksum>::new();
        for contract in &contracts {
            checksums.push(cache.store_code(contract.wasm, true, true).unwrap());
        }
        checksums
    };

    let after = SystemTime::now().duration_since(start_time).unwrap();
    eprintln!("Done compiling after {after:?}");

    let cache: Cache<MockApi, MockStorage, MockQuerier> = unsafe { Cache::new(options).unwrap() };
    for round in 0..ROUNDS {
        for _ in 0..ROUND_LEN {
            if SystemTime::now().duration_since(start_time).unwrap() > Duration::from_secs(runtime)
            {
                eprintln!("Round {round}. End time reached. Ending the process");

                let metrics = cache.metrics();
                eprintln!("Cache metrics: {metrics:?}");

                return; // ends app()
            }

            for idx in 0..contracts.len() {
                let mut instance = cache
                    .get_instance(&checksums[idx], mock_backend(&[]), DEFAULT_INSTANCE_OPTIONS)
                    .unwrap();

                instance.set_debug_handler(|_msg, info| {
                    let _t = now_rfc3339();
                    let _gas = info.gas_remaining;
                    //eprintln!("[{t}]: {msg} (gas remaining: {gas})");
                });

                if let Some(msg) = &contracts[idx].instantiate_msg {
                    let info = mock_info("creator", &coins(1000, "earth"));
                    let contract_result =
                        call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
                            .unwrap();
                    assert!(contract_result.into_result().is_ok());
                }

                for (execution_idx, execute) in contracts[idx].execute_msgs.iter().enumerate() {
                    let info = mock_info("verifies", &coins(15, "earth"));
                    let msg = execute.msg;
                    let res =
                        call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg);

                    if execute.expect_error {
                        if res.is_ok() {
                            panic!(
                                "Round {round}, Execution {execution_idx}, Contract {idx}. Expected error but got {res:?}"
                            );
                        }
                    } else {
                        if res.is_err() {
                            panic!("Round {round}, Execution {execution_idx}, Contract {idx}. Expected no error but got {res:?}");
                        }
                    }
                }
            }
        }
    }
}

fn now_rfc3339() -> String {
    let dt = OffsetDateTime::from(SystemTime::now());
    dt.format(&Rfc3339).unwrap_or_default()
}

pub fn main() {
    let matches = Command::new("Heap profiling")
        .version("0.0.0")
        .arg(
            Arg::new("runtime")
                .long("runtime")
                .help("Time in seconds how long the tests should be running")
                .value_parser(clap::value_parser!(u64).range(1..10_000))
                .default_value("30"),
        )
        .get_matches();
    let runtime = matches
        .get_one::<u64>("runtime")
        .expect("Error parsing time argument");

    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    app(*runtime);
}
