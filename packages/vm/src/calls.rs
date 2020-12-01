use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use std::fmt;
use wasmer::Val;

use cosmwasm_std::{
    ContractResult, Env, HandleResponse, InitResponse, MessageInfo, MigrateResponse, QueryResponse,
};

use crate::backend::{Api, Querier, Storage};
use crate::errors::{VmError, VmResult};
use crate::instance::Instance;
use crate::serde::{from_slice, to_vec};

const MAX_LENGTH_INIT: usize = 100_000;
const MAX_LENGTH_HANDLE: usize = 100_000;
const MAX_LENGTH_MIGRATE: usize = 100_000;
const MAX_LENGTH_QUERY: usize = 100_000;

pub fn call_init<S, A, Q, U>(
    instance: &mut Instance<S, A, Q>,
    env: &Env,
    info: &MessageInfo,
    msg: &[u8],
) -> VmResult<ContractResult<InitResponse<U>>>
where
    S: Storage + 'static,
    A: Api + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + Clone + fmt::Debug + JsonSchema + PartialEq,
{
    let env = to_vec(env)?;
    let info = to_vec(info)?;
    let data = call_init_raw(instance, &env, &info, msg)?;
    let result: ContractResult<InitResponse<U>> = from_slice(&data)?;
    Ok(result)
}

pub fn call_handle<S, A, Q, U>(
    instance: &mut Instance<S, A, Q>,
    env: &Env,
    info: &MessageInfo,
    msg: &[u8],
) -> VmResult<ContractResult<HandleResponse<U>>>
where
    S: Storage + 'static,
    A: Api + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + Clone + fmt::Debug + JsonSchema + PartialEq,
{
    let env = to_vec(env)?;
    let info = to_vec(info)?;
    let data = call_handle_raw(instance, &env, &info, msg)?;
    let result: ContractResult<HandleResponse<U>> = from_slice(&data)?;
    Ok(result)
}

pub fn call_migrate<S, A, Q, U>(
    instance: &mut Instance<S, A, Q>,
    env: &Env,
    info: &MessageInfo,
    msg: &[u8],
) -> VmResult<ContractResult<MigrateResponse<U>>>
where
    S: Storage + 'static,
    A: Api + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + Clone + fmt::Debug + JsonSchema + PartialEq,
{
    let env = to_vec(env)?;
    let info = to_vec(info)?;
    let data = call_migrate_raw(instance, &env, &info, msg)?;
    let result: ContractResult<MigrateResponse<U>> = from_slice(&data)?;
    Ok(result)
}

pub fn call_query<S: Storage + 'static, A: Api + 'static, Q: Querier + 'static>(
    instance: &mut Instance<S, A, Q>,
    env: &Env,
    msg: &[u8],
) -> VmResult<ContractResult<QueryResponse>> {
    let env = to_vec(env)?;
    let data = call_query_raw(instance, &env, msg)?;
    let result: ContractResult<QueryResponse> = from_slice(&data)?;

    // Ensure query response is valid JSON
    if let ContractResult::Ok(binary_response) = &result {
        serde_json::from_slice::<serde_json::Value>(binary_response.as_slice()).map_err(|e| {
            VmError::generic_err(format!("Query response must be valid JSON. {}", e))
        })?;
    }

    Ok(result)
}

/// Calls Wasm export "init" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_init_raw<S: Storage + 'static, A: Api + 'static, Q: Querier + 'static>(
    instance: &mut Instance<S, A, Q>,
    env: &[u8],
    info: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>> {
    instance.set_storage_readonly(false);
    call_raw(instance, "init", &[env, info, msg], MAX_LENGTH_INIT)
}

/// Calls Wasm export "handle" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_handle_raw<S: Storage + 'static, A: Api + 'static, Q: Querier + 'static>(
    instance: &mut Instance<S, A, Q>,
    env: &[u8],
    info: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>> {
    instance.set_storage_readonly(false);
    call_raw(instance, "handle", &[env, info, msg], MAX_LENGTH_HANDLE)
}

/// Calls Wasm export "migrate" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_migrate_raw<S: Storage + 'static, A: Api + 'static, Q: Querier + 'static>(
    instance: &mut Instance<S, A, Q>,
    env: &[u8],
    info: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>> {
    instance.set_storage_readonly(false);
    call_raw(instance, "migrate", &[env, info, msg], MAX_LENGTH_MIGRATE)
}

/// Calls Wasm export "query" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_query_raw<S: Storage + 'static, A: Api + 'static, Q: Querier + 'static>(
    instance: &mut Instance<S, A, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>> {
    instance.set_storage_readonly(true);
    call_raw(instance, "query", &[env, msg], MAX_LENGTH_QUERY)
}

fn call_raw<S: Storage + 'static, A: Api + 'static, Q: Querier + 'static>(
    instance: &mut Instance<S, A, Q>,
    name: &str,
    args: &[&[u8]],
    result_max_length: usize,
) -> VmResult<Vec<u8>> {
    let mut arg_region_ptrs = Vec::<Val>::with_capacity(args.len());
    for arg in args {
        let region_ptr = instance.allocate(arg.len())?;
        instance.write_memory(region_ptr, arg)?;
        arg_region_ptrs.push(region_ptr.into());
    }
    let result = instance.call_function(name, &arg_region_ptrs)?;
    let res_region_ptr = result[0].unwrap_i32() as u32;
    let data = instance.read_memory(res_region_ptr, result_max_length)?;
    // free return value in wasm (arguments were freed in wasm code)
    instance.deallocate(res_region_ptr)?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{mock_env, mock_info, mock_instance};
    use cosmwasm_std::{coins, Empty};

    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

    #[test]
    fn call_init_works() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        // init
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        call_init::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();
    }

    #[test]
    fn call_handle_works() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        // init
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        call_init::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();

        // handle
        let info = mock_info("verifies", &coins(15, "earth"));
        let msg = br#"{"release":{}}"#;
        call_handle::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();
    }

    #[test]
    fn call_query_works() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        // init
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        call_init::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();

        // query
        let msg = r#"{"verifier":{}}"#.as_bytes();
        let contract_result = call_query(&mut instance, &mock_env(), msg).unwrap();
        let query_response = contract_result.unwrap();
        assert_eq!(query_response.as_slice(), b"{\"verifier\":\"verifies\"}");
    }
}
