use cosmwasm_std::{
    entry_point, DepsMut, Empty, Env, EurekaPacketReceiveMsg, IbcReceiveResponse, MessageInfo,
    Response, StdResult,
};

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> StdResult<Response> {
    Ok(Response::new())
}

#[entry_point]
pub fn eu_packet_receive(
    _deps: DepsMut,
    _env: Env,
    _msg: EurekaPacketReceiveMsg,
) -> StdResult<IbcReceiveResponse> {
    Ok(IbcReceiveResponse::without_ack())
}
