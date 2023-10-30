//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.
//!
//! You can easily convert unit tests to integration tests.
//! 1. First copy them over verbatum,
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

use cosmwasm_std::{coins, to_json_vec, ContractResult, Empty, Response};
use cosmwasm_vm::{
    call_instantiate,
    testing::{mock_env, mock_info, mock_instance},
    VmError,
};

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/empty.wasm");
// You can uncomment this line instead to test productionified build from rust-optimizer
// static WASM: &[u8] = include_bytes!("../contract.wasm");

#[test]
fn instantiate_fails() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = Empty {};
    let info = mock_info("creator", &coins(1000, "earth"));

    let serialized_msg = to_json_vec(&msg).unwrap();
    let result: Result<ContractResult<Response>, VmError> =
        call_instantiate(&mut deps, &mock_env(), &info, &serialized_msg);
    let err = result.unwrap_err();

    assert!(matches!(
        err,
        VmError::ResolveErr {
            msg,
            ..
        }
        if msg == "Could not get export: Missing export instantiate"
    ));
}
