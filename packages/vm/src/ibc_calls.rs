#![cfg(feature = "stargate")]

use cosmwasm_std::{
    ContractResult, Env, IbcAcknowledgement, IbcBasicResponse, IbcChannel, IbcPacket,
    IbcReceiveResponse,
};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use std::fmt;

use crate::backend::{BackendApi, Querier, Storage};
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
    A: BackendApi + 'static,
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
    A: BackendApi + 'static,
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
    A: BackendApi + 'static,
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
    A: BackendApi + 'static,
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
    A: BackendApi + 'static,
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
    A: BackendApi + 'static,
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
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "ibc_channel_open",
        &[env, channel],
        MAX_LENGTH_IBC,
    )
}

pub fn call_ibc_channel_connect_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    channel: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "ibc_channel_connect",
        &[env, channel],
        MAX_LENGTH_IBC,
    )
}

pub fn call_ibc_channel_close_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    channel: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "ibc_channel_close",
        &[env, channel],
        MAX_LENGTH_IBC,
    )
}

pub fn call_ibc_packet_receive_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    packet: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "ibc_packet_receive",
        &[env, packet],
        MAX_LENGTH_IBC,
    )
}

pub fn call_ibc_packet_ack_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    ack: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
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
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "ibc_packet_timeout",
        &[env, packet],
        MAX_LENGTH_IBC,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calls::{call_instantiate, call_reply};
    use crate::testing::{mock_env, mock_info, mock_instance, MockApi, MockQuerier, MockStorage};
    use cosmwasm_std::testing::{mock_ibc_channel, mock_ibc_packet_ack};
    use cosmwasm_std::{attr, Empty, Event, IbcOrder, Reply, SubcallResponse};

    static CONTRACT: &[u8] = include_bytes!("../testdata/ibc_reflect.wasm");
    const IBC_VERSION: &str = "ibc-reflect-v1";

    fn setup(
        instance: &mut Instance<MockApi, MockStorage, MockQuerier>,
        channel_id: &str,
        account: &str,
    ) {
        // init
        let info = mock_info("creator", &[]);
        let msg = br#"{"reflect_code_id":77}"#;
        call_instantiate::<_, _, _, Empty>(instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();

        // first we try to open with a valid handshake
        let mut handshake_open = mock_ibc_channel(channel_id, IbcOrder::Ordered, IBC_VERSION);
        handshake_open.counterparty_version = None;
        call_ibc_channel_open(instance, &mock_env(), &handshake_open)
            .unwrap()
            .unwrap();

        // then we connect (with counter-party version set)
        let handshake_connect = mock_ibc_channel(channel_id, IbcOrder::Ordered, IBC_VERSION);
        let res: IbcBasicResponse =
            call_ibc_channel_connect::<_, _, _, Empty>(instance, &mock_env(), &handshake_connect)
                .unwrap()
                .unwrap();
        assert_eq!(1, res.submessages.len());
        let id = res.submessages[0].id;

        let event = Event {
            kind: "message".into(),
            attributes: vec![attr("contract_address", &account)],
        };
        // which creates a reflect account. here we get the callback
        let response = Reply {
            id,
            result: ContractResult::Ok(SubcallResponse {
                events: vec![event],
                data: None,
            }),
        };
        call_reply::<_, _, _, Empty>(instance, &mock_env(), &response).unwrap();
    }

    const CHANNEL_ID: &str = "channel-123";
    const ACCOUNT: &str = "account-456";

    #[test]
    fn call_ibc_channel_open_and_connect_works() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        setup(&mut instance, CHANNEL_ID, ACCOUNT);
    }

    #[test]
    fn call_ibc_channel_close_works() {
        let mut instance = mock_instance(&CONTRACT, &[]);
        setup(&mut instance, CHANNEL_ID, ACCOUNT);

        let handshake_close = mock_ibc_channel(CHANNEL_ID, IbcOrder::Ordered, IBC_VERSION);
        call_ibc_channel_close::<_, _, _, Empty>(&mut instance, &mock_env(), &handshake_close)
            .unwrap()
            .unwrap();
    }

    #[test]
    fn call_ibc_packet_ack_works() {
        let mut instance = mock_instance(&CONTRACT, &[]);
        setup(&mut instance, CHANNEL_ID, ACCOUNT);

        let packet = mock_ibc_packet_ack(CHANNEL_ID, br#"{}"#).unwrap();
        let ack = IbcAcknowledgement {
            acknowledgement: br#"{}"#.into(),
            original_packet: packet,
        };
        call_ibc_packet_ack::<_, _, _, Empty>(&mut instance, &mock_env(), &ack)
            .unwrap()
            .unwrap();
    }

    #[test]
    fn call_ibc_packet_timeout_works() {
        let mut instance = mock_instance(&CONTRACT, &[]);
        setup(&mut instance, CHANNEL_ID, ACCOUNT);

        let packet = mock_ibc_packet_ack(CHANNEL_ID, br#"{}"#).unwrap();
        call_ibc_packet_timeout::<_, _, _, Empty>(&mut instance, &mock_env(), &packet)
            .unwrap()
            .unwrap();
    }

    #[test]
    fn call_ibc_packet_receive_works() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        setup(&mut instance, CHANNEL_ID, ACCOUNT);

        let who_am_i = br#"{"who_am_i":{}}"#;
        let packet = mock_ibc_packet_ack(CHANNEL_ID, who_am_i).unwrap();

        call_ibc_packet_receive::<_, _, _, Empty>(&mut instance, &mock_env(), &packet)
            .unwrap()
            .unwrap();
    }
}
