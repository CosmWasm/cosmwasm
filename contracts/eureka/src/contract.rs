use cosmwasm_std::{
    entry_point, DepsMut, Env, EurekaPacketReceiveMsg, IbcReceiveResponse, MessageInfo, Response,
    StdResult,
};

use crate::error::EurekaError;
use crate::msg::InstantiateMsg;

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, EurekaError> {
    // This adds some unrelated event attribute for testing purposes
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
