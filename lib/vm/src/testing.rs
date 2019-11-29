// This file has some helpers for integration tests.
// They should be imported via full path to ensure there is no confusion
// use cosmwasm_vm::testing::X

use std::vec::Vec;

use cosmwasm::mock::MockStorage;
use cosmwasm::storage::Storage;
use cosmwasm::types::{ContractResult, Params};

use crate::calls::{call_handle, call_init};
use crate::instance::Instance;

pub fn mock_instance(wasm: &[u8]) -> Instance<MockStorage> {
    let storage = MockStorage::new();
    Instance::from_code(wasm, storage).unwrap()
}

// init mimicks the call signature of the smart contracts.
// thus it moves params and msg rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn init<T: Storage + 'static>(instance: &mut Instance<T>, params: Params, msg: Vec<u8>) -> ContractResult {
    call_init(instance, &params, &msg).unwrap()
}

// handle mimicks the call signature of the smart contracts.
// thus it moves params and msg rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn handle<T: Storage + 'static>(instance: &mut Instance<T>, params: Params, msg: Vec<u8>) -> ContractResult {
    call_handle(instance, &params, &msg).unwrap()
}

