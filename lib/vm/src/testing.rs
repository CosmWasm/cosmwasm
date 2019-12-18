// This file has some helpers for integration tests.
// They should be imported via full path to ensure there is no confusion
// use cosmwasm_vm::testing::X

use std::vec::Vec;

use cosmwasm::mock::{dependencies, MockApi, MockStorage};
use cosmwasm::traits::{Api, Storage};
use cosmwasm::types::{ContractResult, Params, QueryResult};

use crate::calls::{call_handle, call_init, call_query};
use crate::instance::Instance;

pub fn mock_instance(wasm: &[u8]) -> Instance<MockStorage, MockApi> {
    let deps = dependencies(20);
    Instance::from_code(wasm, deps).unwrap()
}

// init mimicks the call signature of the smart contracts.
// thus it moves params and msg rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn init<S: Storage + 'static, A: Api + 'static>(
    instance: &mut Instance<S, A>,
    params: Params,
    msg: Vec<u8>,
) -> ContractResult {
    call_init(instance, &params, &msg).unwrap()
}

// handle mimicks the call signature of the smart contracts.
// thus it moves params and msg rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn handle<S: Storage + 'static, A: Api + 'static>(
    instance: &mut Instance<S, A>,
    params: Params,
    msg: Vec<u8>,
) -> ContractResult {
    call_handle(instance, &params, &msg).unwrap()
}

// query mimicks the call signature of the smart contracts.
// thus it moves params and msg rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn query<S: Storage + 'static, A: Api + 'static>(
    instance: &mut Instance<S, A>,
    msg: Vec<u8>,
) -> QueryResult {
    call_query(instance, &msg).unwrap()
}
