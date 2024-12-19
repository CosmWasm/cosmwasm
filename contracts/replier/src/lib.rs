use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, to_json_vec, Binary, Deps, DepsMut, Env, MessageInfo,
    QueryResponse, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, WasmMsg,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const SET_DATA_IN_EXEC_AND_REPLY_FLAG: u64 = 0x100;
const RETURN_ORDER_IN_REPLY_FLAG: u64 = 0x200;
const REPLY_ERROR_FLAG: u64 = 0x400;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct ExecuteMsg {
    msg_id: u8,
    set_data_in_exec_and_reply: bool,
    return_order_in_reply: bool,
    exec_error: bool,
    reply_error: bool,
    reply_on_never: bool,
    messages: Vec<ExecuteMsg>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}

pub const CONFIG_KEY: &[u8] = b"config";
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub order: Vec<u8>,
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    deps.storage
        .set(CONFIG_KEY, &to_json_vec(&State { order: vec![] })?);
    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    let data = deps.storage.get(CONFIG_KEY).unwrap();
    let mut config: State = from_json(data)?;
    if msg.msg_id <= 1 {
        config.order.clear();
    }
    config.order.extend([0xEE, msg.msg_id]);
    deps.storage.set(CONFIG_KEY, &to_json_vec(&config)?);

    let mut resp = Response::new();

    if msg.set_data_in_exec_and_reply {
        resp = resp.set_data(Binary::new(vec![0xEE, msg.msg_id]));
    }

    if msg.exec_error {
        return Err(StdError::generic_err(format!(
            "Err in exec msg_id: {}",
            msg.msg_id
        )));
    }

    for next_msg in msg.messages {
        let wasm_msg = WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_json_binary(&next_msg).unwrap(),
            funds: vec![],
        };
        let mut msg_id: u64 = msg.msg_id.into();
        if msg.set_data_in_exec_and_reply {
            msg_id |= SET_DATA_IN_EXEC_AND_REPLY_FLAG;
        }
        if msg.return_order_in_reply {
            msg_id |= RETURN_ORDER_IN_REPLY_FLAG;
        }
        if msg.reply_error {
            msg_id |= REPLY_ERROR_FLAG;
        }

        let submsg = SubMsg {
            id: msg_id,
            payload: Binary::default(),
            msg: wasm_msg.into(),
            gas_limit: None,
            reply_on: if next_msg.reply_on_never {
                ReplyOn::Never
            } else {
                ReplyOn::Always
            },
        };
        resp = resp.add_submessage(submsg);
    }
    Ok(resp)
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<QueryResponse> {
    Ok(QueryResponse::new(vec![]))
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    let msg_id = msg.id & 0xFF;
    let should_set_data = msg.id & SET_DATA_IN_EXEC_AND_REPLY_FLAG != 0;
    let should_set_order = msg.id & RETURN_ORDER_IN_REPLY_FLAG != 0;
    let should_return_error = msg.id & REPLY_ERROR_FLAG != 0;

    let data = deps.storage.get(CONFIG_KEY).unwrap();
    let mut config: State = from_json(data)?;
    config.order.extend([0xBB, msg_id as u8]);
    deps.storage.set(CONFIG_KEY, &to_json_vec(&config)?);

    if should_return_error {
        return Err(StdError::generic_err(format!(
            "Err in reply msg_id: {}",
            msg_id
        )));
    }

    let result = if msg.result.is_ok() {
        msg.result.unwrap()
    } else {
        return Ok(Response::new());
    };

    if should_set_order {
        Ok(Response::new().set_data(Binary::new(config.order)))
    } else if should_set_data {
        Ok(Response::new().set_data(Binary::new(
            result
                .msg_responses
                .into_iter()
                .flat_map(|resp| resp.value.as_slice().to_vec())
                .chain([0xBB, msg_id as u8])
                .collect(),
        )))
    } else {
        Ok(Response::new())
    }
}
