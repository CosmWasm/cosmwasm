//! This file has some helpers for integration tests.
//! They should be imported via full path to ensure there is no confusion
//! use cosmwasm_vm::testing::X
use cosmwasm_std::{Coin, HumanAddr};
use std::collections::HashSet;

use crate::compatibility::check_wasm;
use crate::features::features_from_csv;
use crate::instance::{Instance, InstanceOptions};
use crate::{Api, Backend, Querier, Storage};

use super::mock::{MockApi, MOCK_CONTRACT_ADDR};
use super::querier::MockQuerier;
use super::storage::MockStorage;

const DEFAULT_GAS_LIMIT: u64 = 500_000;
const DEFAULT_PRINT_DEBUG: bool = true;

pub fn mock_instance(
    wasm: &[u8],
    contract_balance: &[Coin],
) -> Instance<MockStorage, MockApi, MockQuerier> {
    mock_instance_with_options(
        wasm,
        MockInstanceOptions {
            contract_balance: Some(contract_balance),
            ..Default::default()
        },
    )
}

pub fn mock_instance_with_failing_api(
    wasm: &[u8],
    contract_balance: &[Coin],
    backend_error: &'static str,
) -> Instance<MockStorage, MockApi, MockQuerier> {
    mock_instance_with_options(
        wasm,
        MockInstanceOptions {
            contract_balance: Some(contract_balance),
            backend_error: Some(backend_error),
            ..Default::default()
        },
    )
}

pub fn mock_instance_with_balances(
    wasm: &[u8],
    balances: &[(&HumanAddr, &[Coin])],
) -> Instance<MockStorage, MockApi, MockQuerier> {
    mock_instance_with_options(
        wasm,
        MockInstanceOptions {
            balances,
            ..Default::default()
        },
    )
}

pub fn mock_instance_with_gas_limit(
    wasm: &[u8],
    gas_limit: u64,
) -> Instance<MockStorage, MockApi, MockQuerier> {
    mock_instance_with_options(
        wasm,
        MockInstanceOptions {
            gas_limit,
            ..Default::default()
        },
    )
}

#[derive(Debug)]
pub struct MockInstanceOptions<'a> {
    // dependencies
    pub balances: &'a [(&'a HumanAddr, &'a [Coin])],
    /// This option is merged into balances and might override an existing value
    pub contract_balance: Option<&'a [Coin]>,
    /// When set, all calls to the API fail with BackendError::Unknown containing this message
    pub backend_error: Option<&'static str>,

    // instance
    pub supported_features: HashSet<String>,
    pub gas_limit: u64,
    pub print_debug: bool,
}

impl Default for MockInstanceOptions<'_> {
    fn default() -> Self {
        Self {
            // dependencies
            balances: Default::default(),
            contract_balance: Default::default(),
            backend_error: None,

            // instance
            supported_features: features_from_csv("staking"),
            gas_limit: DEFAULT_GAS_LIMIT,
            print_debug: DEFAULT_PRINT_DEBUG,
        }
    }
}

pub fn mock_instance_with_options(
    wasm: &[u8],
    options: MockInstanceOptions,
) -> Instance<MockStorage, MockApi, MockQuerier> {
    check_wasm(wasm, &options.supported_features).unwrap();
    let contract_address = HumanAddr::from(MOCK_CONTRACT_ADDR);

    // merge balances
    let mut balances = options.balances.to_vec();
    if let Some(contract_balance) = options.contract_balance {
        // Remove old entry if exists
        if let Some(pos) = balances.iter().position(|item| *item.0 == contract_address) {
            balances.remove(pos);
        }
        balances.push((&contract_address, contract_balance));
    }

    let api = if let Some(backend_error) = options.backend_error {
        MockApi::new_failing(backend_error)
    } else {
        MockApi::default()
    };

    let backend = Backend {
        storage: MockStorage::default(),
        querier: MockQuerier::new(&balances),
        api,
    };
    let options = InstanceOptions {
        gas_limit: options.gas_limit,
        print_debug: options.print_debug,
    };
    Instance::from_code(wasm, backend, options).unwrap()
}

/// Creates InstanceOptions for testing
pub fn mock_instance_options() -> InstanceOptions {
    InstanceOptions {
        gas_limit: DEFAULT_GAS_LIMIT,
        print_debug: DEFAULT_PRINT_DEBUG,
    }
}

/// Runs a series of IO tests, hammering especially on allocate and deallocate.
/// This could be especially useful when run with some kind of leak detector.
pub fn test_io<S: Storage, A: Api + 'static, Q: Querier>(instance: &mut Instance<S, A, Q>) {
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
