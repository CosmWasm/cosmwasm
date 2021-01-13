#![cfg(feature = "stargate")]
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use std::fmt;

use cosmwasm_std::{
    ContractResult, Env, IbcAcknowledgement, IbcBasicResponse, IbcChannel, IbcPacket,
    IbcReceiveResponse,
};

use crate::ibc_calls::{
    call_ibc_channel_close, call_ibc_channel_connect, call_ibc_channel_open, call_ibc_packet_ack,
    call_ibc_packet_receive, call_ibc_packet_timeout,
};
use crate::instance::Instance;
use crate::{Api, Querier, Storage};

// ibc_channel_open mimicks the call signature of the smart contracts.
// thus it moves env and channel rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn ibc_channel_open<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: Env,
    channel: IbcChannel,
) -> ContractResult<()>
where
    A: Api + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    call_ibc_channel_open(instance, &env, &channel).expect("VM error")
}

// ibc_channel_connect mimicks the call signature of the smart contracts.
// thus it moves env and channel rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn ibc_channel_connect<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: Env,
    channel: IbcChannel,
) -> ContractResult<IbcBasicResponse<U>>
where
    A: Api + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + Clone + PartialEq + JsonSchema + fmt::Debug,
{
    call_ibc_channel_connect(instance, &env, &channel).expect("VM error")
}

// ibc_channel_close mimicks the call signature of the smart contracts.
// thus it moves env and channel rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn ibc_channel_close<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: Env,
    channel: IbcChannel,
) -> ContractResult<IbcBasicResponse<U>>
where
    A: Api + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + Clone + PartialEq + JsonSchema + fmt::Debug,
{
    call_ibc_channel_close(instance, &env, &channel).expect("VM error")
}

// ibc_packet_receive mimicks the call signature of the smart contracts.
// thus it moves env and packet rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn ibc_packet_receive<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: Env,
    packet: IbcPacket,
) -> ContractResult<IbcReceiveResponse<U>>
where
    A: Api + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + Clone + PartialEq + JsonSchema + fmt::Debug,
{
    call_ibc_packet_receive(instance, &env, &packet).expect("VM error")
}

// ibc_packet_ack mimicks the call signature of the smart contracts.
// thus it moves env and acknowledgement rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn ibc_packet_ack<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: Env,
    ack: IbcAcknowledgement,
) -> ContractResult<IbcBasicResponse<U>>
where
    A: Api + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + Clone + PartialEq + JsonSchema + fmt::Debug,
{
    call_ibc_packet_ack(instance, &env, &ack).expect("VM error")
}

// ibc_packet_timeout mimicks the call signature of the smart contracts.
// thus it moves env and packet rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn ibc_packet_timeout<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: Env,
    packet: IbcPacket,
) -> ContractResult<IbcBasicResponse<U>>
where
    A: Api + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + Clone + PartialEq + JsonSchema + fmt::Debug,
{
    call_ibc_packet_timeout(instance, &env, &packet).expect("VM error")
}
