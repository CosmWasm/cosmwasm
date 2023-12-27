use std::time::SystemTime;
use tempfile::TempDir;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use cosmwasm_std::{coins, Checksum, Empty};
use cosmwasm_vm::testing::{mock_backend, mock_env, mock_info, MockApi, MockQuerier, MockStorage};
use cosmwasm_vm::{
    call_execute, call_instantiate, capabilities_from_csv, Cache, CacheOptions, InstanceOptions,
    Size,
};

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

// Instance
const DEFAULT_MEMORY_LIMIT: Size = Size::mebi(128);
const DEFAULT_GAS_LIMIT: u64 = u64::MAX;
const DEFAULT_INSTANCE_OPTIONS: InstanceOptions = InstanceOptions {
    gas_limit: DEFAULT_GAS_LIMIT,
};
// Cache
const MEMORY_CACHE_SIZE: Size = Size::mebi(200);

// static HACKATHON: &[u8] = ;
// static CYBERPUNK: &[u8] = include_bytes!("../testdata/cyberpunk.wasm");

struct Execute {
    pub msg: &'static [u8],
    pub expect_error: bool,
}

struct Contract {
    pub wasm: &'static [u8],
    pub instantiate_msg: &'static [u8],
    pub execute_msgs: [Option<Execute>; 3],
}

const CONTRACTS: [Contract; 2] = [
    Contract {
        wasm: include_bytes!("../testdata/cyberpunk.wasm"),
        instantiate_msg: b"{}",
        execute_msgs: [
            Some(Execute {
                msg: br#"{"unreachable":{}}"#,
                expect_error: true,
            }),
            Some(Execute {
                msg: br#"{"allocate_large_memory":{"pages":1000}}"#,
                expect_error: false,
            }),
            Some(Execute {
                // mem_cost in KiB
                msg: br#"{"argon2":{"mem_cost":50000,"time_cost":1}}"#,
                expect_error: false,
            }),
        ],
    },
    Contract {
        wasm: include_bytes!("../testdata/hackatom.wasm"),
        instantiate_msg: br#"{"verifier": "verifies", "beneficiary": "benefits"}"#,
        execute_msgs: [
            Some(Execute {
                msg: br#"{"release":{}}"#,
                expect_error: false,
            }),
            None,
            None,
        ],
    },
];

const END_AFTER: u64 = 10 * 60; // seconds
const ROUNDS: usize = 1024;
const ROUND_LEN: usize = 16;

#[allow(clippy::collapsible_else_if)]
fn app() {
    let start_time = SystemTime::now();

    let options = CacheOptions::new(
        TempDir::new().unwrap().into_path(),
        capabilities_from_csv("iterator,staking"),
        MEMORY_CACHE_SIZE,
        DEFAULT_MEMORY_LIMIT,
    );

    let checksums = {
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options.clone()).unwrap() };

        let mut checksums = Vec::<Checksum>::new();
        for contract in CONTRACTS {
            checksums.push(cache.save_wasm(contract.wasm).unwrap());
        }
        checksums
    };

    let cache: Cache<MockApi, MockStorage, MockQuerier> =
        unsafe { Cache::new(options.clone()).unwrap() };
    for round in 0..ROUNDS {
        for _ in 0..ROUND_LEN {
            if SystemTime::now()
                .duration_since(start_time)
                .unwrap()
                .as_secs()
                > END_AFTER
            {
                eprintln!("End time reached. Ending the process");
                return; // ends app()
            }

            for idx in 0..=1 {
                let mut instance = cache
                    .get_instance(&checksums[idx], mock_backend(&[]), DEFAULT_INSTANCE_OPTIONS)
                    .unwrap();

                instance.set_debug_handler(|msg, info| {
                    let t = now_rfc3339();
                    let gas = info.gas_remaining;
                    eprintln!("[{t}]: {msg} (gas remaining: {gas})");
                });

                let info = mock_info("creator", &coins(1000, "earth"));
                let msg = CONTRACTS[idx].instantiate_msg;
                let contract_result =
                    call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
                        .unwrap();
                assert!(contract_result.into_result().is_ok());

                for (execution_idx, e) in CONTRACTS[idx].execute_msgs.iter().enumerate() {
                    let Some(execute) = e else {
                        continue;
                    };
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

            /*
                let mut instance = cache
                    .get_instance(&checksums[1], mock_backend(&[]), DEFAULT_INSTANCE_OPTIONS)
                    .unwrap();
                //        println!("Done instantiating contract {i}");

                instance.set_debug_handler(|msg, info| {
                    let t = now_rfc3339();
                    let gas = info.gas_remaining;
                    eprintln!("[{t}]: {msg} (gas remaining: {gas})");
                });

                let info = mock_info("creator", &coins(1000, "earth"));
                let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
                let contract_result =
                    call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
                assert!(contract_result.into_result().is_ok());

                let info = mock_info("verifies", &coins(15, "earth"));
                let msg = br#"{"release":{}}"#;
                let contract_result =
                    call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
                assert!(contract_result.into_result().is_ok());
            */
        }

        // let stats = cache.stats();
        // // eprintln!("Stats: {stats:?}");
        // assert_eq!(stats.misses, 0);
        // assert_eq!(stats.hits_fs_cache, 2);
        // assert_eq!(stats.hits_memory_cache as usize, 2 * (ROUND_LEN - 1));
    }
}

fn now_rfc3339() -> String {
    let dt = OffsetDateTime::from(SystemTime::now());
    dt.format(&Rfc3339).unwrap_or_default()
}

pub fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    app();
}
