//! This file has some helpers for integration tests.
//! They should be imported via full path to ensure there is no confusion
//! use cosmwasm_vm::testing::X
use cosmwasm_std::{Coin, HumanAddr};
use std::collections::HashSet;
use std::iter::FromIterator;

use crate::compatability::check_wasm;
use crate::instance::Instance;
use crate::{Api, Querier, Storage};

use super::mock::{mock_dependencies, mock_dependencies_with_balances, MockApi, MockQuerier};
use super::storage::MockStorage;

/// Gas limit for testing
const DEFAULT_GAS_LIMIT: u64 = 500_000;

fn default_features() -> HashSet<String> {
    HashSet::from_iter(["staking".to_string()].iter().cloned())
}

pub fn mock_instance(
    wasm: &[u8],
    contract_balance: &[Coin],
) -> Instance<MockStorage, MockApi, MockQuerier> {
    check_wasm(wasm, &default_features()).unwrap();
    let deps = mock_dependencies(20, contract_balance);
    Instance::from_code(wasm, deps, DEFAULT_GAS_LIMIT).unwrap()
}

pub fn mock_instance_with_balances(
    wasm: &[u8],
    balances: &[(&HumanAddr, &[Coin])],
) -> Instance<MockStorage, MockApi, MockQuerier> {
    check_wasm(wasm, &default_features()).unwrap();
    let deps = mock_dependencies_with_balances(20, balances);
    Instance::from_code(wasm, deps, DEFAULT_GAS_LIMIT).unwrap()
}

pub fn mock_instance_with_gas_limit(
    wasm: &[u8],
    contract_balance: &[Coin],
    gas_limit: u64,
) -> Instance<MockStorage, MockApi, MockQuerier> {
    check_wasm(wasm, &default_features()).unwrap();
    let deps = mock_dependencies(20, contract_balance);
    Instance::from_code(wasm, deps, gas_limit).unwrap()
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
