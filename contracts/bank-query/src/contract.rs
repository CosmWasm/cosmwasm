use cosmwasm_std::{
    entry_point, to_binary, AllBalanceResponse, BalanceResponse, BankQuery, Binary, Deps, DepsMut,
    Empty, Env, MessageInfo, QueryRequest, Response, StdError, StdResult, SupplyResponse,
};

use crate::msg::QueryMsg;

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
pub fn execute(_deps: DepsMut, _env: Env, _info: MessageInfo, _msg: Empty) -> StdResult<Response> {
    Err(StdError::generic_err("unimplemented"))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Supply { denom } => {
            let res: SupplyResponse = deps
                .querier
                .query(&QueryRequest::Bank(BankQuery::Supply { denom }))?;

            to_binary(&res)
        }
        QueryMsg::Balance { address, denom } => {
            let res: BalanceResponse = deps
                .querier
                .query(&QueryRequest::Bank(BankQuery::Balance { address, denom }))?;

            to_binary(&res)
        }
        QueryMsg::AllBalances { address } => {
            let res: AllBalanceResponse = deps
                .querier
                .query(&QueryRequest::Bank(BankQuery::AllBalances { address }))?;

            to_binary(&res)
        }
    }
}
