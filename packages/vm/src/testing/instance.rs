//! This file has some helpers for integration tests.
//! They should be imported via full path to ensure there is no confusion
//! use cosmwasm_vm::testing::X
use cosmwasm_std::{Coin, HumanAddr};
use std::collections::HashSet;

use crate::compatability::check_wasm;
use crate::features::features_from_csv;
use crate::instance::Instance;
use crate::{Api, Extern, Querier, Storage};

use super::mock::{MockApi, MockQuerier, MOCK_CONTRACT_ADDR};
use super::storage::MockStorage;

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
    contract_balance: &[Coin],
    gas_limit: u64,
) -> Instance<MockStorage, MockApi, MockQuerier> {
    mock_instance_with_options(
        wasm,
        MockInstanceOptions {
            contract_balance: Some(contract_balance),
            gas_limit,
            ..Default::default()
        },
    )
}

#[derive(Debug)]
pub struct MockInstanceOptions<'a> {
    // dependencies
    pub canonical_address_length: usize,
    pub balances: &'a [(&'a HumanAddr, &'a [Coin])],
    /// This option is merged into balances and might override an existing value
    pub contract_balance: Option<&'a [Coin]>,

    // instance
    pub supported_features: HashSet<String>,
    pub gas_limit: u64,
}

impl Default for MockInstanceOptions<'_> {
    fn default() -> Self {
        Self {
            // dependencies
            canonical_address_length: 20,
            balances: Default::default(),
            contract_balance: Default::default(),

            // instance
            supported_features: features_from_csv("staking"),
            gas_limit: 500_000,
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

    let deps = Extern {
        storage: MockStorage::default(),
        api: MockApi::new(options.canonical_address_length),
        querier: MockQuerier::new(&balances),
    };
    Instance::from_code(wasm, deps, options.gas_limit).unwrap()
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
