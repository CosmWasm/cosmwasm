use cosmwasm_std::{
    entry_point, from_json, to_json_vec, Binary, Deps, DepsMut, Empty, Env, Ibc2Msg,
    Ibc2PacketReceiveMsg, Ibc2PacketTimeoutMsg, Ibc2Payload, IbcAcknowledgement, IbcBasicResponse,
    IbcReceiveResponse, MessageInfo, QueryResponse, Response, StdAck, StdError, StdResult,
};

use crate::msg::{IbcPayload, QueryMsg};
use crate::state::{State, STATE_KEY};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> StdResult<Response> {
    deps.storage.set(
        STATE_KEY,
        &to_json_vec(&State {
            ibc2_packet_receive_counter: 0,
            ibc2_packet_timeout_counter: 0,
            last_source_client: "".to_owned(),
            last_packet_seq: 0,
        })?,
    );

    Ok(Response::new())
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::QueryState {} => {
            let data = deps
                .storage
                .get(STATE_KEY)
                .ok_or_else(|| StdError::generic_err("State not found."))?;
            Ok(Binary::from(data))
        }
    }
}

#[entry_point]
pub fn ibc2_packet_receive(
    deps: DepsMut,
    env: Env,
    msg: Ibc2PacketReceiveMsg,
) -> StdResult<IbcReceiveResponse> {
    let binary_payload = &msg.payload.value;
    let json_payload: IbcPayload = from_json(binary_payload)?;

    let data = deps
        .storage
        .get(STATE_KEY)
        .ok_or_else(|| StdError::generic_err("State not found."))?;
    let state: State = from_json(data)?;

    deps.storage.set(
        STATE_KEY,
        &to_json_vec(&State {
            ibc2_packet_receive_counter: state.ibc2_packet_receive_counter + 1,
            last_source_client: msg.source_client.clone(),
            last_packet_seq: msg.packet_sequence,
            ..state
        })?,
    );
    let new_payload = Ibc2Payload::new(
        msg.payload.destination_port,
        msg.payload.source_port,
        msg.payload.version,
        msg.payload.encoding,
        msg.payload.value,
    );
    let new_msg = Ibc2Msg::SendPacket {
        source_client: msg.source_client,
        payloads: vec![new_payload],
        timeout: env.block.time.plus_minutes(1_u64),
    };

    let resp = if json_payload.response_without_ack {
        IbcReceiveResponse::without_ack().add_attribute("action", "handle_increment")
    } else {
        IbcReceiveResponse::new(StdAck::success(b"\x01"))
            .add_message(new_msg)
            .add_attribute("action", "handle_increment")
    };

    if json_payload.send_async_ack_for_prev_msg {
        Ok(
            resp.add_message(cosmwasm_std::Ibc2Msg::WriteAcknowledgement {
                source_client: state.last_source_client,
                packet_sequence: state.last_packet_seq,
                ack: IbcAcknowledgement::new([1, 2, 3]),
            }),
        )
    } else {
        Ok(resp)
    }
}

#[entry_point]
pub fn ibc2_packet_timeout(
    deps: DepsMut,
    _env: Env,
    _msg: Ibc2PacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    let data = deps
        .storage
        .get(STATE_KEY)
        .ok_or_else(|| StdError::generic_err("State not found."))?;
    let state: State = from_json(data)?;

    deps.storage.set(
        STATE_KEY,
        &to_json_vec(&State {
            ibc2_packet_timeout_counter: state.ibc2_packet_timeout_counter + 1,
            ..state
        })?,
    );

    Ok(IbcBasicResponse::default())
}
