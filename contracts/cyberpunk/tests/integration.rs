//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.
//!
//! You can easily convert unit tests to integration tests as follows:
//! 1. Copy them over verbatim
//! 2. Then change
//!      let mut deps = mock_dependencies(20, &[]);
//!    to
//!      let mut deps = mock_instance(WASM, &[]);
//! 3. If you access raw storage, where ever you see something like:
//!      deps.storage.get(CONFIG_KEY).expect("no data stored");
//!    replace it with:
//!      deps.with_storage(|store| {
//!          let data = store.get(CONFIG_KEY).expect("no data stored");
//!          //...
//!      });
//! 4. Anywhere you see query(&deps, ...) you must replace it with query(&mut deps, ...)

use cosmwasm_std::{from_json, Empty, Env, Response};
use cosmwasm_vm::testing::{
    execute, instantiate, mock_env, mock_info, mock_instance, mock_instance_with_gas_limit, query,
};
use std::io::Write;
use std::time::SystemTime;
use tempfile::NamedTempFile;

use cyberpunk::msg::{ExecuteMsg, QueryMsg};

static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/cyberpunk.wasm");

#[test]
fn execute_argon2() {
    let mut deps = mock_instance_with_gas_limit(WASM, 100_000_000_000);

    let init_info = mock_info("admin", &[]);
    let init_res: Response = instantiate(&mut deps, mock_env(), init_info, Empty {}).unwrap();
    assert_eq!(0, init_res.messages.len());

    let gas_before = deps.get_gas_left();
    let _execute_res: Response = execute(
        &mut deps,
        mock_env(),
        mock_info("admin", &[]),
        ExecuteMsg::Argon2 {
            mem_cost: 256,
            time_cost: 5,
        },
    )
    .unwrap();
    let gas_used = gas_before - deps.get_gas_left();
    // Note: the exact gas usage depends on the Rust version used to compile Wasm,
    // which we only fix when using rust-optimizer, not integration tests.
    let expected = 8635688250; // +/- 20%
    assert!(gas_used > expected * 80 / 100, "Gas used: {gas_used}");
    assert!(gas_used < expected * 120 / 100, "Gas used: {gas_used}");
}

// Test with
// cargo integration-test debug_works -- --nocapture
#[test]
fn debug_works() {
    let mut deps = mock_instance_with_gas_limit(WASM, 100_000_000_000);

    let _res: Response =
        instantiate(&mut deps, mock_env(), mock_info("admin", &[]), Empty {}).unwrap();

    let msg = ExecuteMsg::Debug {};
    let _res: Response = execute(&mut deps, mock_env(), mock_info("caller", &[]), msg).unwrap();

    let start = SystemTime::now();
    deps.set_debug_handler(move |msg, info| {
        let gas = info.gas_remaining;
        let runtime = SystemTime::now().duration_since(start).unwrap().as_micros();
        eprintln!("{msg} (gas: {gas}, runtime: {runtime}µs)");
    });

    let msg = ExecuteMsg::Debug {};
    let _res: Response = execute(&mut deps, mock_env(), mock_info("caller", &[]), msg).unwrap();

    eprintln!("Unsetting debug handler. From here nothing is printed anymore.");
    deps.unset_debug_handler();

    let msg = ExecuteMsg::Debug {};
    let _res: Response = execute(&mut deps, mock_env(), mock_info("caller", &[]), msg).unwrap();
}

// Test with
// cargo integration-test debug_timing -- --nocapture
#[test]
fn debug_timing() {
    let mut deps = mock_instance_with_gas_limit(WASM, 100_000_000_000);

    let _res: Response =
        instantiate(&mut deps, mock_env(), mock_info("admin", &[]), Empty {}).unwrap();

    let mut last_time = None;
    deps.set_debug_handler(move |msg, _info| {
        if let Some(last_time) = last_time {
            let diff = SystemTime::now()
                .duration_since(last_time)
                .unwrap_or_default()
                .as_micros();
            eprintln!("{msg} (time since last debug: {diff}µs)");
        } else {
            eprintln!("{msg}");
        }

        last_time = Some(SystemTime::now());
    });

    let msg = ExecuteMsg::Debug {};
    let _res: Response = execute(&mut deps, mock_env(), mock_info("caller", &[]), msg).unwrap();
}

#[test]
fn debug_file() {
    let mut deps = mock_instance_with_gas_limit(WASM, 100_000_000_000);

    let _res: Response =
        instantiate(&mut deps, mock_env(), mock_info("admin", &[]), Empty {}).unwrap();

    let temp_file = NamedTempFile::new().unwrap();
    let (mut temp_file, temp_path) = temp_file.into_parts();

    deps.set_debug_handler(move |msg, _info| {
        writeln!(temp_file, "{msg}").unwrap();
    });

    let msg = ExecuteMsg::Debug {};
    let _res: Response = execute(&mut deps, mock_env(), mock_info("caller", &[]), msg).unwrap();

    // check if file contains the expected output
    let file_content = std::fs::read_to_string(temp_path).unwrap();
    assert!(file_content.contains("Round 9 done"));
}

#[test]
fn test_env() {
    let mut deps = mock_instance(WASM, &[]);

    let init_info = mock_info("admin", &[]);
    let init_res: Response = instantiate(&mut deps, mock_env(), init_info, Empty {}).unwrap();
    assert_eq!(0, init_res.messages.len());

    let env = mock_env();
    let res: Response = execute(
        &mut deps,
        env.clone(),
        mock_info("admin", &[]),
        ExecuteMsg::MirrorEnv {},
    )
    .unwrap();

    let received_env: Env = from_json(res.data.unwrap()).unwrap();

    assert_eq!(received_env, env);

    let env = mock_env();
    let received_env: Env =
        from_json(query(&mut deps, env.clone(), QueryMsg::MirrorEnv {}).unwrap()).unwrap();

    assert_eq!(received_env, env);
}
