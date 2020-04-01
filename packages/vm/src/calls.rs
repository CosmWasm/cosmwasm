use snafu::ResultExt;

use cosmwasm_std::{
    Api, ApiError, Env, HandleResponse, HandleResult, InitResponse, InitResult, QueryResponse,
    QueryResult, Storage,
};

use crate::errors::{Error, RuntimeErr};
use crate::instance::{Func, Instance};
use crate::serde::{from_slice, to_vec};

static MAX_LENGTH_INIT_HANDLE: usize = 100_000;
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

pub fn call_query_raw<S: Storage + 'static, A: Api + 'static>(
    instance: &mut Instance<S, A>,
    msg: &[u8],
) -> Result<Vec<u8>, Error> {
    // we cannot resuse the call_raw functionality as it assumes a param variable... just do it inline
    let msg_region_ptr = instance.allocate(msg.len())?;
    instance.write_memory(msg_region_ptr, msg)?;
    let func: Func<u32, u32> = instance.func("query")?;
    let res_region_ptr = func.call(msg_region_ptr).context(RuntimeErr {})?;
    let data = instance.read_memory(res_region_ptr, MAX_LENGTH_INIT_HANDLE)?;
    // free return value in wasm (arguments were freed in wasm code)
    instance.deallocate(res_region_ptr)?;
    Ok(data)
}

pub fn call_init_raw<S: Storage + 'static, A: Api + 'static>(
    instance: &mut Instance<S, A>,
    env: &[u8],
    msg: &[u8],
) -> Result<Vec<u8>, Error> {
    call_raw(instance, "init", env, msg)
}

pub fn call_handle_raw<S: Storage + 'static, A: Api + 'static>(
    instance: &mut Instance<S, A>,
    env: &[u8],
    msg: &[u8],
) -> Result<Vec<u8>, Error> {
    call_raw(instance, "handle", env, msg)
}

fn call_raw<S: Storage + 'static, A: Api + 'static>(
    instance: &mut Instance<S, A>,
    name: &str,
    env: &[u8],
    msg: &[u8],
) -> Result<Vec<u8>, Error> {
    let env_region_ptr = instance.allocate(env.len())?;
    instance.write_memory(env_region_ptr, env)?;
    let msg_region_ptr = instance.allocate(msg.len())?;
    instance.write_memory(msg_region_ptr, msg)?;

    let func: Func<(u32, u32), u32> = instance.func(name)?;
    let res_region_ptr = func
        .call(env_region_ptr, msg_region_ptr)
        .context(RuntimeErr {})?;

    let data = instance.read_memory(res_region_ptr, MAX_LENGTH_QUERY)?;
    // free return value in wasm (arguments were freed in wasm code)
    instance.deallocate(res_region_ptr)?;
    Ok(data)
}
