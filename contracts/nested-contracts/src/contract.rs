#![allow(unreachable_code, clippy::diverging_sub_expression)]

use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:nested-contracts";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    inner_contract::contract::instantiate(deps, env, info, todo!()).unwrap();
    unimplemented!()
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    inner_contract::contract::execute(deps, env, info, todo!()).unwrap();
    unimplemented!()
}

#[entry_point]
pub fn query(deps: Deps, env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    inner_contract::contract::query(deps, env, todo!()).unwrap();
    unimplemented!()
}

#[cfg(test)]
mod tests {}
