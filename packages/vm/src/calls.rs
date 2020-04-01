use snafu::ResultExt;

use cosmwasm_std::{
    Api, ApiError, Env, HandleResponse, HandleResult, InitResponse, InitResult, QueryResponse,
    QueryResult, Storage,
};

use crate::errors::{Error, RuntimeErr};
use crate::instance::{Func, Instance};
use crate::serde::{from_slice, to_vec};

static MAX_LENGTH_INIT: usize = 100_000;
static MAX_LENGTH_HANDLE: usize = 100_000;
static MAX_LENGTH_QUERY: usize = 100_000;

pub fn call_init<S: Storage + 'static, A: Api + 'static>(
    instance: &mut Instance<S, A>,
    env: &Env,
    msg: &[u8],
) -> Result<Result<InitResponse, ApiError>, Error> {
    let env = to_vec(env)?;
    let data = call_init_raw(instance, &env, msg)?;
    let res: InitResult = from_slice(&data)?;
    Ok(res.into())
}

pub fn call_handle<S: Storage + 'static, A: Api + 'static>(
    instance: &mut Instance<S, A>,
    env: &Env,
    msg: &[u8],
) -> Result<Result<HandleResponse, ApiError>, Error> {
    let env = to_vec(env)?;
    let data = call_handle_raw(instance, &env, msg)?;
    let res: HandleResult = from_slice(&data)?;
    Ok(res.into())
}

pub fn call_query<S: Storage + 'static, A: Api + 'static>(
    instance: &mut Instance<S, A>,
    msg: &[u8],
) -> Result<Result<QueryResponse, ApiError>, Error> {
    let data = call_query_raw(instance, msg)?;
    let res: QueryResult = from_slice(&data)?;
    Ok(res.into())
}

/// Calls Wasm export "init" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_init_raw<S: Storage + 'static, A: Api + 'static>(
    instance: &mut Instance<S, A>,
    env: &[u8],
    msg: &[u8],
) -> Result<Vec<u8>, Error> {
    call_raw(instance, "init", &[env, msg], MAX_LENGTH_INIT)
}

/// Calls Wasm export "handle" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_handle_raw<S: Storage + 'static, A: Api + 'static>(
    instance: &mut Instance<S, A>,
    env: &[u8],
    msg: &[u8],
) -> Result<Vec<u8>, Error> {
    call_raw(instance, "handle", &[env, msg], MAX_LENGTH_HANDLE)
}

/// Calls Wasm export "query" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_query_raw<S: Storage + 'static, A: Api + 'static>(
    instance: &mut Instance<S, A>,
    msg: &[u8],
) -> Result<Vec<u8>, Error> {
    call_raw(instance, "query", &[msg], MAX_LENGTH_QUERY)
}

fn call_raw<S: Storage + 'static, A: Api + 'static>(
    instance: &mut Instance<S, A>,
    name: &str,
    args: &[&[u8]],
    result_max_length: usize,
) -> Result<Vec<u8>, Error> {
    let mut arg_region_ptrs = Vec::<u32>::with_capacity(args.len());
    for arg in args {
        let region_ptr = instance.allocate(arg.len())?;
        instance.write_memory(region_ptr, arg)?;
        arg_region_ptrs.push(region_ptr);
    }

    let res_region_ptr = match args.len() {
        1 => {
            let func: Func<u32, u32> = instance.func(name)?;
            func.call(arg_region_ptrs[0]).context(RuntimeErr {})?
        }
        2 => {
            let func: Func<(u32, u32), u32> = instance.func(name)?;
            func.call(arg_region_ptrs[0], arg_region_ptrs[1])
                .context(RuntimeErr {})?
        }
        _ => panic!("call_raw called with unsupported number of arguments"),
    };

    let data = instance.read_memory(res_region_ptr, result_max_length)?;
    // free return value in wasm (arguments were freed in wasm code)
    instance.deallocate(res_region_ptr)?;
    Ok(data)
}
