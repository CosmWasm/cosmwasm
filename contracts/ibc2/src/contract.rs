use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    entry_point, from_json, to_json_vec, Binary, Deps, DepsMut, Empty, Env, Ibc2PacketReceiveMsg,
    IbcAcknowledgement, IbcReceiveResponse, MessageInfo, QueryResponse, Response, StdError,
    StdResult,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    ibc2_packet_receive_counter: u32,
    last_channel_id: String,
    last_packet_seq: u64,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(State)]
    QueryState {},
}

#[cw_serde]
pub struct IbcPayload {
    response_without_ack: bool,
    send_async_ack_for_prev_msg: bool,
}

const STATE_KEY: &[u8] = b"state";

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
pub fn ibc2_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: Ibc2PacketReceiveMsg,
) -> StdResult<IbcReceiveResponse> {
    let binary_payload = msg.payload.value;
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
            last_channel_id: msg.source_client,
            last_packet_seq: msg.packet_sequence,
        })?,
    );

    let resp = if json_payload.response_without_ack {
        IbcReceiveResponse::without_ack()
    } else {
        IbcReceiveResponse::new([1, 2, 3])
    };

    if json_payload.send_async_ack_for_prev_msg {
        Ok(
            resp.add_message(cosmwasm_std::Ibc2Msg::WriteAcknowledgement {
                channel_id: state.last_channel_id,
                packet_sequence: state.last_packet_seq,
                ack: IbcAcknowledgement::new([1, 2, 3]),
            }),
        )
    } else {
        Ok(resp)
    }
}
