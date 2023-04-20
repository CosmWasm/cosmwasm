use cosmwasm_std::{
    entry_point, to_binary, CosmosMsg, Deps, DepsMut, Env, IbcMsg, MessageInfo, Order,
    QueryResponse, Response, StdError, StdResult,
};

use crate::ibc::PACKET_LIFETIME;
use crate::ibc_msg::PacketMsg;
use crate::msg::{
    AccountInfo, AccountResponse, AdminResponse, ExecuteMsg, InstantiateMsg, ListAccountsResponse,
    QueryMsg,
};
use crate::state::{accounts, accounts_read, config, config_read, Config};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    // we store the reflect_id for creating accounts later
    let cfg = Config { admin: info.sender };
    config(deps.storage).save(&cfg)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateAdmin { admin } => handle_update_admin(deps, info, admin),
        ExecuteMsg::SendMsgs { channel_id, msgs } => {
            handle_send_msgs(deps, env, info, channel_id, msgs)
        }
        ExecuteMsg::CheckRemoteBalance { channel_id } => {
            handle_check_remote_balance(deps, env, info, channel_id)
        }
        ExecuteMsg::SendFunds {
            reflect_channel_id,
            transfer_channel_id,
        } => handle_send_funds(deps, env, info, reflect_channel_id, transfer_channel_id),
    }
}

pub fn handle_update_admin(
    deps: DepsMut,
    info: MessageInfo,
    new_admin: String,
) -> StdResult<Response> {
    // auth check
    let mut cfg = config(deps.storage).load()?;
    if info.sender != cfg.admin {
        return Err(StdError::generic_err("Only admin may set new admin"));
    }
    cfg.admin = deps.api.addr_validate(&new_admin)?;
    config(deps.storage).save(&cfg)?;

    Ok(Response::new()
        .add_attribute("action", "handle_update_admin")
        .add_attribute("new_admin", cfg.admin))
}

pub fn handle_send_msgs(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    channel_id: String,
    msgs: Vec<CosmosMsg>,
) -> StdResult<Response> {
    // auth check
    let cfg = config(deps.storage).load()?;
    if info.sender != cfg.admin {
        return Err(StdError::generic_err("Only admin may send messages"));
    }
    // ensure the channel exists (not found if not registered)
    accounts(deps.storage).load(channel_id.as_bytes())?;

    // construct a packet to send
    let packet = PacketMsg::Dispatch { msgs };
    let msg = IbcMsg::SendPacket {
        channel_id,
        data: to_binary(&packet)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "handle_send_msgs");
    Ok(res)
}

pub fn handle_check_remote_balance(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    channel_id: String,
) -> StdResult<Response> {
    // auth check
    let cfg = config(deps.storage).load()?;
    if info.sender != cfg.admin {
        return Err(StdError::generic_err("Only admin may send messages"));
    }
    // ensure the channel exists (not found if not registered)
    accounts(deps.storage).load(channel_id.as_bytes())?;

    // construct a packet to send
    let packet = PacketMsg::Balances {};
    let msg = IbcMsg::SendPacket {
        channel_id,
        data: to_binary(&packet)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "handle_check_remote_balance");
    Ok(res)
}

pub fn handle_send_funds(
    deps: DepsMut,
    env: Env,
    mut info: MessageInfo,
    reflect_channel_id: String,
    transfer_channel_id: String,
) -> StdResult<Response> {
    // intentionally no auth check

    // require some funds
    let amount = match info.funds.pop() {
        Some(coin) => coin,
        None => {
            return Err(StdError::generic_err(
                "you must send the coins you wish to ibc transfer",
            ))
        }
    };
    // if there are any more coins, reject the message
    if !info.funds.is_empty() {
        return Err(StdError::generic_err("you can only ibc transfer one coin"));
    }

    // load remote account
    let data = accounts(deps.storage).load(reflect_channel_id.as_bytes())?;
    let remote_addr = match data.remote_addr {
        Some(addr) => addr,
        None => {
            return Err(StdError::generic_err(
                "We don't have the remote address for this channel",
            ))
        }
    };

    // construct a packet to send
    let msg = IbcMsg::Transfer {
        channel_id: transfer_channel_id,
        to_address: remote_addr,
        amount,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
        memo: "",
    };

    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "handle_send_funds");
    Ok(res)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Admin {} => to_binary(&query_admin(deps)?),
        QueryMsg::Account { channel_id } => to_binary(&query_account(deps, channel_id)?),
        QueryMsg::ListAccounts {} => to_binary(&query_list_accounts(deps)?),
    }
}

fn query_account(deps: Deps, channel_id: String) -> StdResult<AccountResponse> {
    let account = accounts_read(deps.storage).load(channel_id.as_bytes())?;
    Ok(account.into())
}

fn query_list_accounts(deps: Deps) -> StdResult<ListAccountsResponse> {
    let accounts: StdResult<Vec<_>> = accounts_read(deps.storage)
        .range(None, None, Order::Ascending)
        .map(|r| {
            let (k, account) = r?;
            let channel_id = String::from_utf8(k)?;
            Ok(AccountInfo::convert(channel_id, account))
        })
        .collect();
    Ok(ListAccountsResponse {
        accounts: accounts?,
    })
}

fn query_admin(deps: Deps) -> StdResult<AdminResponse> {
    let Config { admin } = config_read(deps.storage).load()?;
    Ok(AdminResponse {
        admin: admin.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    const CREATOR: &str = "creator";

    #[test]
    fn instantiate_works() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = mock_info(CREATOR, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let admin = query_admin(deps.as_ref()).unwrap();
        assert_eq!(CREATOR, admin.admin.as_str());
    }
}
