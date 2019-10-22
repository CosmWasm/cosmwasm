use failure::Error;

use cosmwasm::serde::{from_slice, to_vec};
use cosmwasm::storage::Storage;
use cosmwasm::types::{ContractResult, Params};

use crate::instance::{Func, Instance};
use crate::memory::read_memory;

pub fn call_init<T>(
    instance: &mut Instance<T>,
    params: &Params,
    msg: &[u8],
) -> Result<ContractResult, Error>
where
    T: Storage + Send + Sync + Clone + 'static,
{
    let params = to_vec(params)?;
    let data = call_init_raw(instance, &params, msg)?;
    let res: ContractResult = from_slice(&data)?;
    Ok(res)
}

pub fn call_handle<T>(
    instance: &mut Instance<T>,
    params: &Params,
    msg: &[u8],
) -> Result<ContractResult, Error>
where
    T: Storage + Send + Sync + Clone + 'static,
{
    let params = to_vec(params)?;
    let data = call_handle_raw(instance, &params, msg)?;
    let res: ContractResult = from_slice(&data)?;
    Ok(res)
}

pub fn call_init_raw<T>(
    instance: &mut Instance<T>,
    params: &[u8],
    msg: &[u8],
) -> Result<Vec<u8>, Error>
where
    T: Storage + Send + Sync + Clone + 'static,
{
    call_raw(instance, "init", params, msg)
}

pub fn call_handle_raw<T>(
    instance: &mut Instance<T>,
    params: &[u8],
    msg: &[u8],
) -> Result<Vec<u8>, Error>
where
    T: Storage + Send + Sync + Clone + 'static,
{
    call_raw(instance, "handle", params, msg)
}

fn call_raw<T>(
    instance: &mut Instance<T>,
    name: &str,
    params: &[u8],
    msg: &[u8],
) -> Result<Vec<u8>, Error>
where
    T: Storage + Send + Sync + Clone + 'static,
{
    let param_offset = instance.allocate(params);
    let msg_offset = instance.allocate(msg);

    // TODO: failure cannot handle unwrap this error
    let func: Func<(u32, u32), (u32)> = instance.func(name)?;
    let res_offset = func.call(param_offset, msg_offset).unwrap();

    let data = read_memory(instance.context(), res_offset);
    Ok(data)
}
