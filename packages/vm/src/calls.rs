use serde::de::DeserializeOwned;
use std::fmt;

use cosmwasm_std::{Env, HandleResult, InitResult, MigrateResult, QueryResult};

use crate::errors::{VmError, VmResult};
use crate::instance::{Func, Instance};
use crate::serde::{from_slice, to_vec};
use crate::traits::{Api, BackendStorage, Querier};
use schemars::JsonSchema;

const MAX_LENGTH_INIT: usize = 100_000;
const MAX_LENGTH_HANDLE: usize = 100_000;
const MAX_LENGTH_MIGRATE: usize = 100_000;
const MAX_LENGTH_QUERY: usize = 100_000;

pub fn call_init<S, A, Q, U>(
    instance: &mut Instance<S, A, Q>,
    env: &Env,
    msg: &[u8],
) -> VmResult<InitResult<U>>
where
    S: BackendStorage + 'static,
    A: Api + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + Clone + fmt::Debug + JsonSchema + PartialEq,
{
    let env = to_vec(env)?;
    let data = call_init_raw(instance, &env, msg)?;
    let result: InitResult<U> = from_slice(&data)?;
    Ok(result)
}

pub fn call_handle<S, A, Q, U>(
    instance: &mut Instance<S, A, Q>,
    env: &Env,
    msg: &[u8],
) -> VmResult<HandleResult<U>>
where
    S: BackendStorage + 'static,
    A: Api + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + Clone + fmt::Debug + JsonSchema + PartialEq,
{
    let env = to_vec(env)?;
    let data = call_handle_raw(instance, &env, msg)?;
    let result: HandleResult<U> = from_slice(&data)?;
    Ok(result)
}

pub fn call_migrate<S, A, Q, U>(
    instance: &mut Instance<S, A, Q>,
    env: &Env,
    msg: &[u8],
) -> VmResult<MigrateResult<U>>
where
    S: BackendStorage + 'static,
    A: Api + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + Clone + fmt::Debug + JsonSchema + PartialEq,
{
    let env = to_vec(env)?;
    let data = call_migrate_raw(instance, &env, msg)?;
    let result: MigrateResult<U> = from_slice(&data)?;
    Ok(result)
}

pub fn call_query<S: BackendStorage + 'static, A: Api + 'static, Q: Querier + 'static>(
    instance: &mut Instance<S, A, Q>,
    msg: &[u8],
) -> VmResult<QueryResult> {
    let data = call_query_raw(instance, msg)?;
    let result: QueryResult = from_slice(&data)?;

    // Ensure query response is valid JSON
    if let Ok(binary_response) = &result {
        serde_json::from_slice::<serde_json::Value>(binary_response.as_slice()).map_err(|e| {
            VmError::generic_err(format!("Query response must be valid JSON. {}", e))
        })?;
    }

    Ok(result)
}

/// Calls Wasm export "init" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_init_raw<S: BackendStorage + 'static, A: Api + 'static, Q: Querier + 'static>(
    instance: &mut Instance<S, A, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>> {
    instance.set_storage_readonly(false);
    call_raw(instance, "init", &[env, msg], MAX_LENGTH_INIT)
}

/// Calls Wasm export "handle" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_handle_raw<S: BackendStorage + 'static, A: Api + 'static, Q: Querier + 'static>(
    instance: &mut Instance<S, A, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>> {
    instance.set_storage_readonly(false);
    call_raw(instance, "handle", &[env, msg], MAX_LENGTH_HANDLE)
}

/// Calls Wasm export "migrate" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_migrate_raw<S: BackendStorage + 'static, A: Api + 'static, Q: Querier + 'static>(
    instance: &mut Instance<S, A, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>> {
    instance.set_storage_readonly(false);
    call_raw(instance, "migrate", &[env, msg], MAX_LENGTH_MIGRATE)
}

/// Calls Wasm export "query" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_query_raw<S: BackendStorage + 'static, A: Api + 'static, Q: Querier + 'static>(
    instance: &mut Instance<S, A, Q>,
    msg: &[u8],
) -> VmResult<Vec<u8>> {
    instance.set_storage_readonly(true);
    call_raw(instance, "query", &[msg], MAX_LENGTH_QUERY)
}

fn call_raw<S: BackendStorage + 'static, A: Api + 'static, Q: Querier + 'static>(
    instance: &mut Instance<S, A, Q>,
    name: &str,
    args: &[&[u8]],
    result_max_length: usize,
) -> VmResult<Vec<u8>> {
    let mut arg_region_ptrs = Vec::<u32>::with_capacity(args.len());
    for arg in args {
        let region_ptr = instance.allocate(arg.len())?;
        instance.write_memory(region_ptr, arg)?;
        arg_region_ptrs.push(region_ptr);
    }

    let res_region_ptr = match args.len() {
        1 => {
            let func: Func<u32, u32> = instance.func(name)?;
            func.call(arg_region_ptrs[0])?
        }
        2 => {
            let func: Func<(u32, u32), u32> = instance.func(name)?;
            func.call(arg_region_ptrs[0], arg_region_ptrs[1])?
        }
        _ => panic!("call_raw called with unsupported number of arguments"),
    };

    let data = instance.read_memory(res_region_ptr, result_max_length)?;
    // free return value in wasm (arguments were freed in wasm code)
    instance.deallocate(res_region_ptr)?;
    Ok(data)
}
