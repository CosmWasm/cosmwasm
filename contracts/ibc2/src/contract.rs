use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    entry_point, from_json, to_json_vec, Binary, Deps, DepsMut, Empty, Env,
    Ibc2PacketReceiveMsg, IbcReceiveResponse, MessageInfo, QueryResponse, Response, StdError,
    StdResult,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    ibc2_packet_receive_counter: u32,
    ibc2_timeout_counter: u32,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(State)]
    QueryState {},
    #[returns(u32)]
    QueryTimeoutCounter {},
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
        QueryMsg::QueryTimeoutCounter {} => {
            let data = deps
                .storage
                .get(STATE_KEY)
                .ok_or_else(|| StdError::generic_err("State not found."))?;
            let state: State = from_json(&data)?;
            Ok(Binary::from(to_json_vec(&state.ibc2_timeout_counter)?))
        }
    }
}

#[entry_point]
pub fn ibc2_packet_receive(
    deps: DepsMut,
    _env: Env,
    _msg: Ibc2PacketReceiveMsg,
) -> StdResult<IbcReceiveResponse> {
    let data = deps
        .storage
        .get(STATE_KEY)
        .ok_or_else(|| StdError::generic_err("State not found."))?;
    let state: State = from_json(data)?;
    deps.storage.set(
        STATE_KEY,
        &to_json_vec(&State {
            ibc2_packet_receive_counter: state.ibc2_packet_receive_counter + 1,
            ibc2_timeout_counter: state.ibc2_timeout_counter,
        })?,
    );

    Ok(IbcReceiveResponse::new([1, 2, 3]))
}

#[entry_point]
pub fn ibc2_timeout(
    deps: DepsMut,
    _env: Env,
    _msg: Ibc2PacketReceiveMsg,
) -> StdResult<IbcReceiveResponse> {
    let data = deps
        .storage
        .get(STATE_KEY)
        .ok_or_else(|| StdError::generic_err("State not found."))?;
    let state: State = from_json(data)?;
    deps.storage.set(
        STATE_KEY,
        &to_json_vec(&State {
            ibc2_packet_receive_counter: state.ibc2_packet_receive_counter,
            ibc2_timeout_counter: state.ibc2_timeout_counter + 1,
        })?,
    );

    Ok(IbcReceiveResponse::new([1, 2, 3]))
}
