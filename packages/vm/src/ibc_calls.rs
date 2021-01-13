#![cfg(feature = "stargate")]

use cosmwasm_std::{
    ContractResult, Env, IbcAcknowledgement, IbcBasicResponse, IbcChannel, IbcPacket,
    IbcReceiveResponse,
};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use std::fmt;

use crate::backend::{Api, Querier, Storage};
use crate::calls::call_raw;
use crate::errors::VmResult;
use crate::instance::Instance;
use crate::serde::{from_slice, to_vec};

const MAX_LENGTH_IBC: usize = 100_000;

pub fn call_ibc_channel_open<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    channel: &IbcChannel,
) -> VmResult<ContractResult<()>>
where
    A: Api + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    let env = to_vec(env)?;
    let msg = to_vec(channel)?;
    instance.set_storage_readonly(false);
    let data = call_raw(instance, "ibc_channel_open", &[&env, &msg], MAX_LENGTH_IBC)?;
    let result: ContractResult<()> = from_slice(&data)?;
    Ok(result)
}

pub fn call_ibc_channel_connect<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    channel: &IbcChannel,
) -> VmResult<ContractResult<IbcBasicResponse<U>>>
where
    A: Api + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + Clone + fmt::Debug + JsonSchema + PartialEq,
{
    let env = to_vec(env)?;
    let msg = to_vec(channel)?;
    instance.set_storage_readonly(false);
    let data = call_raw(
        instance,
        "ibc_channel_connect",
        &[&env, &msg],
        MAX_LENGTH_IBC,
    )?;
    let result = from_slice(&data)?;
    Ok(result)
}

pub fn call_ibc_channel_close<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    channel: &IbcChannel,
) -> VmResult<ContractResult<IbcBasicResponse<U>>>
where
    A: Api + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + Clone + fmt::Debug + JsonSchema + PartialEq,
{
    let env = to_vec(env)?;
    let msg = to_vec(channel)?;
    instance.set_storage_readonly(false);
    let data = call_raw(instance, "ibc_channel_close", &[&env, &msg], MAX_LENGTH_IBC)?;
    let result = from_slice(&data)?;
    Ok(result)
}

pub fn call_ibc_packet_receive<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    packet: &IbcPacket,
) -> VmResult<ContractResult<IbcReceiveResponse<U>>>
where
    A: Api + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + Clone + fmt::Debug + JsonSchema + PartialEq,
{
    let env = to_vec(env)?;
    let msg = to_vec(packet)?;
    instance.set_storage_readonly(false);
    let data = call_raw(
        instance,
        "ibc_packet_receive",
        &[&env, &msg],
        MAX_LENGTH_IBC,
    )?;
    let result = from_slice(&data)?;
    Ok(result)
}

pub fn call_ibc_packet_ack<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    channel: &IbcAcknowledgement,
) -> VmResult<ContractResult<IbcBasicResponse<U>>>
where
    A: Api + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + Clone + fmt::Debug + JsonSchema + PartialEq,
{
    let env = to_vec(env)?;
    let msg = to_vec(channel)?;
    instance.set_storage_readonly(false);
    let data = call_raw(instance, "ibc_packet_ack", &[&env, &msg], MAX_LENGTH_IBC)?;
    let result = from_slice(&data)?;
    Ok(result)
}

pub fn call_ibc_packet_timeout<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    channel: &IbcPacket,
) -> VmResult<ContractResult<IbcBasicResponse<U>>>
where
    A: Api + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + Clone + fmt::Debug + JsonSchema + PartialEq,
{
    let env = to_vec(env)?;
    let msg = to_vec(channel)?;
    instance.set_storage_readonly(false);
    let data = call_raw(
        instance,
        "ibc_packet_timeout",
        &[&env, &msg],
        MAX_LENGTH_IBC,
    )?;
    let result = from_slice(&data)?;
    Ok(result)
}
