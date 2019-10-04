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
    // prepare arguments
    let params = to_vec(params)?;
    let param_offset = allocate(instance, &params);
    let msg_offset = allocate(instance, msg);
    let init: Func<(i32, i32), (i32)> = instance.func("init_wrapper")?;

    // call function (failure cannot handle unwrap this error)
    let res_offset = init.call(param_offset, msg_offset).unwrap();

    // read return value
    let data = read_memory(instance.context(), res_offset);
    let res: ContractResult = from_slice(&data)?;
    Ok(res)
}

pub fn call_send(
    instance: &mut Instance,
    params: &Params,
    msg: &[u8],
) -> Result<ContractResult, Error> {
    // prepare arguments
    let params = to_vec(params)?;
    let param_offset = allocate(instance, &params);
    let msg_offset = allocate(instance, msg);
    let send: Func<(i32, i32), (i32)> = instance.func("send_wrapper")?;

    // call function (failure cannot handle unwrap this error)
    let res_offset = send.call(param_offset, msg_offset).unwrap();

    // read return value
    let data = read_memory(instance.context(), res_offset);
    let res: ContractResult = from_slice(&data)?;
    Ok(res)
}
