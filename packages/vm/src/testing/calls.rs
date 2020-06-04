//! This file has some helpers for integration tests.
//! They should be imported via full path to ensure there is no confusion
//! use cosmwasm_vm::testing::X
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt;

use cosmwasm_std::{
    to_vec, Env, HandleResult, InitResult, MigrateResult, QueryResponse, StdResult,
};

use crate::calls::{call_handle, call_init, call_migrate, call_query};
use crate::instance::Instance;
use crate::{Api, Querier, Storage};

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
) -> InitResult<U> {
    let serialized_msg = to_vec(&msg)?;
    call_init(instance, &env, &serialized_msg).expect("VM error")
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
) -> HandleResult<U> {
    let serialized_msg = to_vec(&msg)?;
    call_handle(instance, &env, &serialized_msg).expect("VM error")
}

// migrate mimicks the call signature of the smart contracts.
// thus it moves env and msg rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn migrate<
    S: Storage + 'static,
    A: Api + 'static,
    Q: Querier + 'static,
    T: Serialize + JsonSchema,
    U: DeserializeOwned + Clone + PartialEq + JsonSchema + fmt::Debug,
>(
    instance: &mut Instance<S, A, Q>,
    env: Env,
    msg: T,
) -> MigrateResult<U> {
    let serialized_msg = to_vec(&msg)?;
    call_migrate(instance, &env, &serialized_msg).expect("VM error")
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
) -> StdResult<QueryResponse> {
    let serialized_msg = to_vec(&msg)?;
    call_query(instance, &serialized_msg).expect("VM error")
}
