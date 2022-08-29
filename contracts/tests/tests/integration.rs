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

use cosmwasm_std::{Empty, Response};
use cosmwasm_vm::testing::{
    execute, instantiate, mock_env, mock_info, mock_instance_with_gas_limit,
};

use tests::msg::ExecuteMsg;

static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/tests.wasm");

#[test]
fn execute_argon2() {
    let mut deps = mock_instance_with_gas_limit(WASM, 100_000_000_000_000);

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
    let expected = 8635688250000; // +/- 20%
    assert!(gas_used > expected * 80 / 100, "Gas used: {}", gas_used);
    assert!(gas_used < expected * 120 / 100, "Gas used: {}", gas_used);
}
