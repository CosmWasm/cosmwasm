use cosmwasm_std::{
    entry_point, from_json, to_json_binary, to_json_vec, Binary, Deps, DepsMut, Empty, Env,
    Ibc2Msg, Ibc2PacketReceiveMsg, Ibc2Payload, IbcReceiveResponse, MessageInfo, QueryResponse,
    Response, StdError, StdResult,
};

use crate::msg::{ExecuteMsg, PacketMsg, QueryMsg};
use crate::state::{State, STATE_KEY};

pub const PACKET_LIFETIME: u64 = 60 * 60;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> StdResult<Response> {
    deps.storage
        .set(STATE_KEY, &to_json_vec(&State::default())?);

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
pub fn execute(
    _deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Increment {
            channel_id,
            destination_port,
        } => handle_increment(env, channel_id, destination_port),
    }
}

pub fn handle_increment(
    env: Env,
    channel_id: String,
    destination_port: String,
) -> StdResult<Response> {
    // construct a packet to send
    let packet = PacketMsg::Increment {};
    let payload = Ibc2Payload::new(
        env.contract.address.to_string(),
        destination_port,
        "v1".to_owned(),
        "json".to_owned(),
        to_json_binary(&packet)?,
    );
    let msg = Ibc2Msg::SendPacket {
        channel_id,
        payloads: vec![payload],
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME),
    };

    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "handle_increment");
    Ok(res)
}

#[entry_point]
pub fn ibc2_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: Ibc2PacketReceiveMsg,
) -> StdResult<IbcReceiveResponse> {
    let msg: PacketMsg = from_json(msg.payload.value)?;

    match msg {
        PacketMsg::Increment {} => {
            let data = deps
                .storage
                .get(STATE_KEY)
                .ok_or_else(|| StdError::generic_err("State not found."))?;
            let state: State = from_json(data)?;
            deps.storage.set(
                STATE_KEY,
                &to_json_vec(&State {
                    ibc2_packet_receive_counter: state.ibc2_packet_receive_counter + 1,
                })?,
            );
            Ok(IbcReceiveResponse::new([1, 2, 3]))
        }
    }
}
