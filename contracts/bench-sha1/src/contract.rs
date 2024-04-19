use cosmwasm_std::{entry_point, Api, DepsMut, Env, MessageInfo, Response, StdResult};
use sha1::{Digest, Sha1};

use crate::msg::{ExecuteMsg, InstantiateMsg};

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    let digest = match msg {
        ExecuteMsg::Wasm { len } => try_wasm_sha1(len),
        ExecuteMsg::Api { len } => try_api_sha1(deps.api, len).unwrap(),
    };
    Ok(Response::default().add_attribute("digest", format!("{:?}", digest)))
}

fn msg_len(len: u64) -> Vec<u8> {
    vec![42u8; len as usize]
}

fn try_wasm_sha1(len: u64) -> Vec<u8> {
    let msg = msg_len(len);
    let mut hasher = Sha1::new();
    hasher.update(&msg);
    hasher.finalize()[..].to_vec()
}

fn try_api_sha1(api: &dyn Api, len: u64) -> StdResult<Vec<u8>> {
    let msg = msg_len(len);
    Ok(api.sha1_calculate(&msg).unwrap().to_vec())
}
