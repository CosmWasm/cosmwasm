use cosmwasm_std::{
    entry_point, instantiate2_address, to_json_binary, Attribute, Binary, CodeInfoResponse,
    ContractInfoResponse, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg,
};

use crate::errors::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Spread {
            parent_path,
            levels,
        } => execute_spread(deps, env, info, parent_path, levels),
    }
}

/// Basic reproduction number
const R0: u32 = 2;

pub fn execute_spread(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    parent_path: String,
    levels: u32,
) -> Result<Response, ContractError> {
    if levels == 0 {
        return Ok(Response::new());
    }

    let creator = deps.api.addr_canonicalize(env.contract.address.as_str())?;
    let ContractInfoResponse { code_id, .. } = deps
        .querier
        .query_wasm_contract_info(env.contract.address)?;
    let CodeInfoResponse { checksum, .. } = deps.querier.query_wasm_code_info(code_id)?;

    let mut msgs = Vec::<WasmMsg>::new();
    let mut attributes = Vec::<Attribute>::new();
    for i in 0..R0 {
        let path = format!("{parent_path}/{i}");
        let label = format!("Instance {path}");
        let salt = Binary::from(path.as_bytes());

        attributes.push(Attribute::new(format!("path{i}"), path.clone()));

        let address =
            deps.api
                .addr_humanize(&instantiate2_address(checksum.as_ref(), &creator, &salt)?)?;
        attributes.push(Attribute::new(
            format!("predicted_address{i}"),
            address.clone(),
        ));

        msgs.push(WasmMsg::Instantiate2 {
            admin: None,
            code_id,
            label,
            msg: to_json_binary(&InstantiateMsg {})?,
            funds: vec![],
            salt,
        });

        // we know the address of the newly instantiated contract, so let's execute it right away
        msgs.push(WasmMsg::Execute {
            contract_addr: address.into(),
            msg: to_json_binary(&ExecuteMsg::Spread {
                parent_path: path,
                levels: levels - 1,
            })?,
            funds: vec![],
        });
    }

    Ok(Response::new()
        .add_attributes(attributes)
        .add_messages(msgs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};

    const CREATOR: &str = "creator";

    #[test]
    fn instantiate_works() {
        let mut deps = mock_dependencies();
        let creator = deps.api.addr_make(CREATOR);
        let msg = InstantiateMsg {};
        let info = message_info(&creator, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 0);
    }
}
