//! This file has some helpers for integration tests.
//! They should be imported via full path to ensure there is no confusion
//! use cosmwasm_vm::testing::X
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt;

use cosmwasm_std::testing::{
    mock_dependencies, mock_dependencies_with_balances, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{
    to_vec, Api, ApiError, Coin, Env, HandleResponse, HumanAddr, InitResponse, NativeMsg, Querier,
    QueryResponse, Storage,
};

use crate::calls::{call_handle, call_init, call_query};
use crate::compatability::check_wasm;
use crate::instance::Instance;

// some helper types for parsing handle/init responses
pub type HandleOk<U = NativeMsg> = HandleResponse<U>;
pub type HandleRes<U = NativeMsg> = Result<HandleOk<U>, ApiError>;
pub type InitOk<U = NativeMsg> = InitResponse<U>;
pub type InitRes<U = NativeMsg> = Result<InitOk<U>, ApiError>;

/// Gas limit for testing
static DEFAULT_GAS_LIMIT: u64 = 500_000;

pub fn mock_instance(
    wasm: &[u8],
    contract_balance: &[Coin],
) -> Instance<MockStorage, MockApi, MockQuerier> {
    check_wasm(wasm).unwrap();
    let deps = mock_dependencies(20, contract_balance);
    Instance::from_code(wasm, deps, DEFAULT_GAS_LIMIT).unwrap()
}

pub fn mock_instance_with_balances(
    wasm: &[u8],
    balances: &[(&HumanAddr, &[Coin])],
) -> Instance<MockStorage, MockApi, MockQuerier> {
    check_wasm(wasm).unwrap();
    let deps = mock_dependencies_with_balances(20, balances);
    Instance::from_code(wasm, deps, DEFAULT_GAS_LIMIT).unwrap()
}

pub fn mock_instance_with_gas_limit(
    wasm: &[u8],
    contract_balance: &[Coin],
    gas_limit: u64,
) -> Instance<MockStorage, MockApi, MockQuerier> {
    check_wasm(wasm).unwrap();
    let deps = mock_dependencies(20, contract_balance);
    Instance::from_code(wasm, deps, gas_limit).unwrap()
}

// init mimicks the call signature of the smart contracts.
// thus it moves env and msg rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn init<
    S: Storage + 'static,
    A: Api + 'static,
    Q: Querier + 'static,
    T: Serialize + JsonSchema,
    U: DeserializeOwned + Clone + PartialEq + JsonSchema + fmt::Debug,
>(
    instance: &mut Instance<S, A, Q>,
    env: Env,
    msg: T,
) -> InitRes<U> {
    match to_vec(&msg) {
        Err(e) => Err(e.into()),
        Ok(serialized_msg) => call_init(instance, &env, &serialized_msg).unwrap(),
    }
}

// handle mimicks the call signature of the smart contracts.
// thus it moves env and msg rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn handle<
    S: Storage + 'static,
    A: Api + 'static,
    Q: Querier + 'static,
    T: Serialize + JsonSchema,
    U: DeserializeOwned + Clone + PartialEq + JsonSchema + fmt::Debug,
>(
    instance: &mut Instance<S, A, Q>,
    env: Env,
    msg: T,
) -> HandleRes<U> {
    match to_vec(&msg) {
        Err(e) => Err(e.into()),
        Ok(serialized_msg) => call_handle(instance, &env, &serialized_msg).unwrap(),
    }
}

// query mimicks the call signature of the smart contracts.
// thus it moves env and msg rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn query<
    S: Storage + 'static,
    A: Api + 'static,
    Q: Querier + 'static,
    T: Serialize + JsonSchema,
>(
    instance: &mut Instance<S, A, Q>,
    msg: T,
) -> Result<QueryResponse, ApiError> {
    match to_vec(&msg) {
        Err(e) => Err(e.into()),
        Ok(serialized_msg) => call_query(instance, &serialized_msg).unwrap(),
    }
}

/// Runs a series of IO tests, hammering especially on allocate and deallocate.
/// This could be especially useful when run with some kind of leak detector.
pub fn test_io<S: Storage + 'static, A: Api + 'static, Q: Querier + 'static>(
    instance: &mut Instance<S, A, Q>,
) {
    let sizes: Vec<usize> = vec![0, 1, 3, 10, 200, 2000, 5 * 1024];
    let bytes: Vec<u8> = vec![0x00, 0xA5, 0xFF];

    for size in sizes.into_iter() {
        for byte in bytes.iter() {
            let original = vec![*byte; size];
            let wasm_ptr = instance
                .allocate(original.len())
                .expect("Could not allocate memory");
            instance
                .write_memory(wasm_ptr, &original)
                .expect("Could not write data");
            let wasm_data = instance.read_memory(wasm_ptr, size).expect("error reading");
            assert_eq!(
                original, wasm_data,
                "failed for size {}; expected: {:?}; actual: {:?}",
                size, original, wasm_data
            );
            instance
                .deallocate(wasm_ptr)
                .expect("Could not deallocate memory");
        }
    }
}
