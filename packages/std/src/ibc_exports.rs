#![cfg(feature = "stargate")]
use std::fmt;
use std::vec::Vec;

use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};

use crate::deps::OwnedDeps;
use crate::imports::{ExternalApi, ExternalQuerier, ExternalStorage};
use crate::memory::{alloc, consume_region, release_buffer, Region};
use crate::results::{
    ContractResult, HandleResponse, InitResponse, MigrateResponse, QueryResponse,
};
use crate::serde::{from_slice, to_vec};
use crate::types::Env;
use crate::{Deps, DepsMut, MessageInfo};

// TODO: replace with https://doc.rust-lang.org/std/ops/trait.Try.html once stabilized
macro_rules! r#try_into_contract_result {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => {
                return ContractResult::Err(err.to_string());
            }
        }
    };
    ($expr:expr,) => {
        $crate::try_into_contract_result!($expr)
    };
}

/// do_init should be wrapped in an external "C" export, containing a contract-specific function as arg
///
/// - `M`: message type for request
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
pub fn do_init<M, C, E>(
    init_fn: &dyn Fn(DepsMut, Env, MessageInfo, M) -> Result<InitResponse<C>, E>,
    env_ptr: u32,
    info_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    M: DeserializeOwned + JsonSchema,
    C: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema,
    E: ToString,
{
    let res = _do_init(
        init_fn,
        env_ptr as *mut Region,
        info_ptr as *mut Region,
        msg_ptr as *mut Region,
    );
    let v = to_vec(&res).unwrap();
    release_buffer(v) as u32
}

/// do_handle should be wrapped in an external "C" export, containing a contract-specific function as arg
///
/// - `M`: message type for request
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
pub fn do_handle<M, C, E>(
    handle_fn: &dyn Fn(DepsMut, Env, MessageInfo, M) -> Result<HandleResponse<C>, E>,
    env_ptr: u32,
    info_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    M: DeserializeOwned + JsonSchema,
    C: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema,
    E: ToString,
{
    let res = _do_handle(
        handle_fn,
        env_ptr as *mut Region,
        info_ptr as *mut Region,
        msg_ptr as *mut Region,
    );
    let v = to_vec(&res).unwrap();
    release_buffer(v) as u32
}

/// do_migrate should be wrapped in an external "C" export, containing a contract-specific function as arg
///
/// - `M`: message type for request
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
pub fn do_migrate<M, C, E>(
    migrate_fn: &dyn Fn(DepsMut, Env, MessageInfo, M) -> Result<MigrateResponse<C>, E>,
    env_ptr: u32,
    info_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    M: DeserializeOwned + JsonSchema,
    C: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema,
    E: ToString,
{
    let res = _do_migrate(
        migrate_fn,
        env_ptr as *mut Region,
        info_ptr as *mut Region,
        msg_ptr as *mut Region,
    );
    let v = to_vec(&res).unwrap();
    release_buffer(v) as u32
}

/// do_query should be wrapped in an external "C" export, containing a contract-specific function as arg
///
/// - `M`: message type for request
/// - `E`: error type for responses
pub fn do_query<M, E>(
    query_fn: &dyn Fn(Deps, Env, M) -> Result<QueryResponse, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    M: DeserializeOwned + JsonSchema,
    E: ToString,
{
    let res = _do_query(query_fn, env_ptr as *mut Region, msg_ptr as *mut Region);
    let v = to_vec(&res).unwrap();
    release_buffer(v) as u32
}

fn _do_init<M, C, E>(
    init_fn: &dyn Fn(DepsMut, Env, MessageInfo, M) -> Result<InitResponse<C>, E>,
    env_ptr: *mut Region,
    info_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<InitResponse<C>>
where
    M: DeserializeOwned + JsonSchema,
    C: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema,
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let info: Vec<u8> = unsafe { consume_region(info_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_slice(&env));
    let info: MessageInfo = try_into_contract_result!(from_slice(&info));
    let msg: M = try_into_contract_result!(from_slice(&msg));

    let mut deps = make_dependencies();
    init_fn(deps.as_mut(), env, info, msg).into()
}

fn _do_handle<M, C, E>(
    handle_fn: &dyn Fn(DepsMut, Env, MessageInfo, M) -> Result<HandleResponse<C>, E>,
    env_ptr: *mut Region,
    info_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<HandleResponse<C>>
where
    M: DeserializeOwned + JsonSchema,
    C: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema,
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let info: Vec<u8> = unsafe { consume_region(info_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_slice(&env));
    let info: MessageInfo = try_into_contract_result!(from_slice(&info));
    let msg: M = try_into_contract_result!(from_slice(&msg));

    let mut deps = make_dependencies();
    handle_fn(deps.as_mut(), env, info, msg).into()
}

fn _do_migrate<M, C, E>(
    migrate_fn: &dyn Fn(DepsMut, Env, MessageInfo, M) -> Result<MigrateResponse<C>, E>,
    env_ptr: *mut Region,
    info_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<MigrateResponse<C>>
where
    M: DeserializeOwned + JsonSchema,
    C: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema,
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let info: Vec<u8> = unsafe { consume_region(info_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_slice(&env));
    let info: MessageInfo = try_into_contract_result!(from_slice(&info));
    let msg: M = try_into_contract_result!(from_slice(&msg));

    let mut deps = make_dependencies();
    migrate_fn(deps.as_mut(), env, info, msg).into()
}

fn _do_query<M, E>(
    query_fn: &dyn Fn(Deps, Env, M) -> Result<QueryResponse, E>,
    env_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<QueryResponse>
where
    M: DeserializeOwned + JsonSchema,
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_slice(&env));
    let msg: M = try_into_contract_result!(from_slice(&msg));

    let deps = make_dependencies();
    query_fn(deps.as_ref(), env, msg).into()
}
