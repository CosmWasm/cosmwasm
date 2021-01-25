use cosmwasm_std::{
    attr, entry_point, to_binary, CosmosMsg, Deps, DepsMut, Env, HandleResponse, HumanAddr, IbcMsg,
    InitResponse, MessageInfo, Order, QueryResponse, StdError, StdResult,
};

use crate::ibc::build_timeout_timestamp;
use crate::ibc_msg::PacketMsg;
use crate::msg::{
    AccountInfo, AccountResponse, AdminResponse, HandleMsg, InitMsg, ListAccountsResponse, QueryMsg,
};
use crate::state::{accounts, accounts_read, config, config_read, Config};

#[entry_point]
pub fn init(deps: DepsMut, _env: Env, info: MessageInfo, _msg: InitMsg) -> StdResult<InitResponse> {
    // we store the reflect_id for creating accounts later
    let cfg = Config { admin: info.sender };
    config(deps.storage).save(&cfg)?;

    Ok(InitResponse {
        data: None,
        messages: vec![],
        attributes: vec![attr("action", "init")],
    })
}

#[entry_point]
pub fn handle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::UpdateAdmin { admin } => handle_update_admin(deps, info, admin),
        HandleMsg::SendMsgs { channel_id, msgs } => {
            handle_send_msgs(deps, env, info, channel_id, msgs)
        }
        HandleMsg::CheckRemoteBalance { channel_id } => {
            handle_check_remote_balance(deps, env, info, channel_id)
        }
        HandleMsg::SendFunds {
            reflect_channel_id,
            transfer_channel_id,
        } => handle_send_funds(deps, env, info, reflect_channel_id, transfer_channel_id),
    }
}

pub fn handle_update_admin(
    deps: DepsMut,
    info: MessageInfo,
    new_admin: HumanAddr,
) -> StdResult<HandleResponse> {
    // auth check
    let mut cfg = config(deps.storage).load()?;
    if info.sender != cfg.admin {
        return Err(StdError::generic_err("Only admin may set new admin"));
    }
    cfg.admin = new_admin;
    config(deps.storage).save(&cfg)?;

    Ok(HandleResponse {
        messages: vec![],
        attributes: vec![
            attr("action", "handle_update_admin"),
            attr("new_admin", cfg.admin),
        ],
        data: None,
    })
}

pub fn handle_send_msgs(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    channel_id: String,
    msgs: Vec<CosmosMsg>,
) -> StdResult<HandleResponse> {
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
        timeout_block: None,
        timeout_timestamp: build_timeout_timestamp(&env.block),
    };

    Ok(HandleResponse {
        messages: vec![msg.into()],
        attributes: vec![attr("action", "handle_send_msgs")],
        data: None,
    })
}

pub fn handle_check_remote_balance(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    channel_id: String,
) -> StdResult<HandleResponse> {
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
        timeout_block: None,
        timeout_timestamp: build_timeout_timestamp(&env.block),
    };

    Ok(HandleResponse {
        messages: vec![msg.into()],
        attributes: vec![attr("action", "handle_check_remote_balance")],
        data: None,
    })
}

pub fn handle_send_funds(
    deps: DepsMut,
    env: Env,
    mut info: MessageInfo,
    reflect_channel_id: String,
    transfer_channel_id: String,
) -> StdResult<HandleResponse> {
    // intentionally no auth check

    // require some funds
    let amount = match info.sent_funds.pop() {
        Some(coin) => coin,
        None => {
            return Err(StdError::generic_err(
                "you must send the coins you wish to ibc transfer",
            ))
        }
    };
    // if there are any more coins, reject the message
    if !info.sent_funds.is_empty() {
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
        timeout_block: None,
        timeout_timestamp: build_timeout_timestamp(&env.block),
    };

    Ok(HandleResponse {
        messages: vec![msg.into()],
        attributes: vec![attr("action", "handle_send_funds")],
        data: None,
    })
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
    Ok(AdminResponse { admin })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    const CREATOR: &str = "creator";

    #[test]
    fn init_works() {
        let mut deps = mock_dependencies(&[]);
        let msg = InitMsg {};
        let info = mock_info(CREATOR, &[]);
        let res = init(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let admin = query_admin(deps.as_ref()).unwrap();
        assert_eq!(CREATOR, admin.admin.as_str());
    }
}
