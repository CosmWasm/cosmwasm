// This file has some helpers for integration tests.
// They should be imported via full path to ensure there is no confusion
// use cosmwasm_vm::testing::X

use std::vec::Vec;

use cosmwasm::mock::{MockPrecompiles, MockStorage};
use cosmwasm::traits::{Precompiles, Storage};
use cosmwasm::types::{ContractResult, Params, QueryResult};

use crate::calls::{call_handle, call_init, call_query};
use crate::instance::Instance;

pub fn mock_instance(wasm: &[u8]) -> Instance<MockStorage, MockPrecompiles> {
    let storage = MockStorage::new();
    let precompiles = MockPrecompiles::new(20);
    Instance::from_code(wasm, storage, precompiles).unwrap()
}

// init mimicks the call signature of the smart contracts.
// thus it moves params and msg rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn init<T: Storage + 'static, U: Precompiles + 'static>(
    instance: &mut Instance<T, U>,
    params: Params,
    msg: Vec<u8>,
) -> ContractResult {
    call_init(instance, &params, &msg).unwrap()
}

// handle mimicks the call signature of the smart contracts.
// thus it moves params and msg rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn handle<T: Storage + 'static, U: Precompiles + 'static>(
    instance: &mut Instance<T, U>,
    params: Params,
    msg: Vec<u8>,
) -> ContractResult {
    call_handle(instance, &params, &msg).unwrap()
}

// query mimicks the call signature of the smart contracts.
// thus it moves params and msg rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn query<T: Storage + 'static, U: Precompiles + 'static>(
    instance: &mut Instance<T, U>,
    msg: Vec<u8>,
) -> QueryResult {
    call_query(instance, &msg).unwrap()
}
