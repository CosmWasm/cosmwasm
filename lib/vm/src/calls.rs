use failure::Error;
use serde_json::{from_slice, to_vec};

use cosmwasm::types::{ContractResult, Params};

use crate::memory::{allocate, read_memory};
use crate::wasmer::{Func, Instance};

pub fn call_init(
    instance: &mut Instance,
    params: &Params,
    msg: &[u8],
) -> Result<ContractResult, Error> {
    let params = to_vec(params)?;
    let data = call_init_raw(instance, &params, msg)?;
    let res: ContractResult = from_slice(&data)?;
    Ok(res)
}

pub fn call_handle(
    instance: &mut Instance,
    params: &Params,
    msg: &[u8],
) -> Result<ContractResult, Error> {
    let params = to_vec(params)?;
    let data = call_handle_raw(instance, &params, msg)?;
    let res: ContractResult = from_slice(&data)?;
    Ok(res)
}

pub fn call_init_raw(instance: &mut Instance, params: &[u8], msg: &[u8]) -> Result<Vec<u8>, Error> {
    call_raw(instance, "init", params, msg)
}

pub fn call_handle_raw(
    instance: &mut Instance,
    params: &[u8],
    msg: &[u8],
) -> Result<Vec<u8>, Error> {
    call_raw(instance, "handle", params, msg)
}

fn call_raw(
    instance: &mut Instance,
    name: &str,
    params: &[u8],
    msg: &[u8],
) -> Result<Vec<u8>, Error> {
    let param_offset = allocate(instance, params);
    let msg_offset = allocate(instance, msg);

    // TODO: failure cannot handle unwrap this error
    let func: Func<(u32, u32), (u32)> = instance.func(name)?;
    let res_offset = func.call(param_offset, msg_offset).unwrap();

    let data = read_memory(instance.context(), res_offset);
    Ok(data)
}
