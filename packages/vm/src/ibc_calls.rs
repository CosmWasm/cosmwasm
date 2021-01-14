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
    let channel = to_vec(channel)?;
    let data = call_ibc_channel_open_raw(instance, &env, &channel)?;
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
    let channel = to_vec(channel)?;
    let data = call_ibc_channel_connect_raw(instance, &env, &channel)?;
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
    let channel = to_vec(channel)?;
    let data = call_ibc_channel_close_raw(instance, &env, &channel)?;
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
    let packet = to_vec(packet)?;
    let data = call_ibc_packet_receive_raw(instance, &env, &packet)?;
    let result = from_slice(&data)?;
    Ok(result)
}

pub fn call_ibc_packet_ack<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    ack: &IbcAcknowledgement,
) -> VmResult<ContractResult<IbcBasicResponse<U>>>
where
    A: Api + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + Clone + fmt::Debug + JsonSchema + PartialEq,
{
    let env = to_vec(env)?;
    let ack = to_vec(ack)?;
    let data = call_ibc_packet_ack_raw(instance, &env, &ack)?;
    let result = from_slice(&data)?;
    Ok(result)
}

pub fn call_ibc_packet_timeout<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    packet: &IbcPacket,
) -> VmResult<ContractResult<IbcBasicResponse<U>>>
where
    A: Api + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + Clone + fmt::Debug + JsonSchema + PartialEq,
{
    let env = to_vec(env)?;
    let packet = to_vec(packet)?;
    let data = call_ibc_packet_timeout_raw(instance, &env, &packet)?;
    let result = from_slice(&data)?;
    Ok(result)
}

pub fn call_ibc_channel_open_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    channel: &[u8],
) -> VmResult<Vec<u8>>
    where
        A: Api + 'static,
        S: Storage + 'static,
        Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(instance, "ibc_channel_open", &[env, channel], MAX_LENGTH_IBC)
}

pub fn call_ibc_channel_connect_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    channel: &[u8],
) -> VmResult<Vec<u8>>
    where
        A: Api + 'static,
        S: Storage + 'static,
        Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(instance, "ibc_channel_connect", &[env, channel], MAX_LENGTH_IBC)
}

pub fn call_ibc_channel_close_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    channel: &[u8],
) -> VmResult<Vec<u8>>
    where
        A: Api + 'static,
        S: Storage + 'static,
        Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(instance, "ibc_channel_close", &[env, channel], MAX_LENGTH_IBC)
}

pub fn call_ibc_packet_receive_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    packet: &[u8],
) -> VmResult<Vec<u8>>
    where
        A: Api + 'static,
        S: Storage + 'static,
        Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(instance, "ibc_packet_receive", &[env, packet], MAX_LENGTH_IBC)
}

pub fn call_ibc_packet_ack_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    ack: &[u8],
) -> VmResult<Vec<u8>>
    where
        A: Api + 'static,
        S: Storage + 'static,
        Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(instance, "ibc_packet_ack", &[env, ack], MAX_LENGTH_IBC)
}

pub fn call_ibc_packet_timeout_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    packet: &[u8],
) -> VmResult<Vec<u8>>
    where
        A: Api + 'static,
        S: Storage + 'static,
        Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(instance, "ibc_packet_timeout", &[env, packet], MAX_LENGTH_IBC)
}
