use cosmwasm_std::{
    attr, entry_point, from_slice, to_binary, BlockInfo, CosmosMsg, Deps, DepsMut, Env,
    HandleResponse, HumanAddr, IbcAcknowledgement, IbcBasicResponse, IbcChannel, IbcMsg, IbcOrder,
    IbcPacket, IbcReceiveResponse, InitResponse, MessageInfo, Order, QueryResponse, StdError,
    StdResult,
};

use crate::msg::{
    AccountInfo, AccountResponse, AcknowledgementMsg, AdminResponse, BalancesResponse,
    DispatchResponse, HandleMsg, InitMsg, ListAccountsResponse, PacketMsg, QueryMsg,
    WhoAmIResponse,
};
use crate::state::{accounts, accounts_read, config, config_read, AccountData, Config};

pub const IBC_VERSION: &str = "ibc-reflect-v1";

// TODO: make configurable?
/// packets live one houe
const PACKET_LIFETIME: u64 = 60 * 60;

fn build_timeout(block: &BlockInfo) -> Option<u64> {
    let timeout = block.time + PACKET_LIFETIME;
    let timeout_nanos = timeout * 1_000_000_000;
    Some(timeout_nanos)
}

#[entry_point]
pub fn init(deps: DepsMut, _env: Env, info: MessageInfo, _msg: InitMsg) -> StdResult<InitResponse> {
    // we store the reflect_id for creating accounts later
    let cfg = Config { admin: info.sender };
    config(deps.storage).save(&cfg)?;

    Ok(InitResponse {
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
        timeout_timestamp: build_timeout(&env.block),
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
        timeout_timestamp: build_timeout(&env.block),
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
    if info.sent_funds.is_empty() {
        return Err(StdError::generic_err(
            "you must send the coins you with to ibc transfer",
        ));
    } else if info.sent_funds.len() > 1 {
        return Err(StdError::generic_err("you can only ibc transfer one coin"));
    }
    let amount = info.sent_funds.swap_remove(0);

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
        timeout_timestamp: build_timeout(&env.block),
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

pub fn query_account(deps: Deps, channel_id: String) -> StdResult<AccountResponse> {
    let account = accounts_read(deps.storage).load(channel_id.as_bytes())?;
    Ok(account.into())
}

pub fn query_list_accounts(deps: Deps) -> StdResult<ListAccountsResponse> {
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

pub fn query_admin(deps: Deps) -> StdResult<AdminResponse> {
    let Config { admin } = config_read(deps.storage).load()?;
    Ok(AdminResponse { admin })
}

#[entry_point]
/// enforces ordering and versioing constraints
pub fn ibc_channel_open(_deps: DepsMut, _env: Env, channel: IbcChannel) -> StdResult<()> {
    if channel.order != IbcOrder::Ordered {
        return Err(StdError::generic_err("Only supports ordered channels"));
    }
    if channel.version.as_str() != IBC_VERSION {
        return Err(StdError::generic_err(format!(
            "Must set version to `{}`",
            IBC_VERSION
        )));
    }
    // TODO: do we need to check counterparty version as well?
    // This flow needs to be well documented
    if let Some(counter_version) = channel.counterparty_version {
        if counter_version.as_str() != IBC_VERSION {
            return Err(StdError::generic_err(format!(
                "Counterparty version must be `{}`",
                IBC_VERSION
            )));
        }
    }

    Ok(())
}

#[entry_point]
/// once it's established, we send a WhoAmI message
pub fn ibc_channel_connect(
    deps: DepsMut,
    env: Env,
    channel: IbcChannel,
) -> StdResult<IbcBasicResponse> {
    let channel_id = channel.endpoint.channel_id;

    // create an account holder the channel exists (not found if not registered)
    let data = AccountData::default();
    accounts(deps.storage).save(channel_id.as_bytes(), &data)?;

    // construct a packet to send
    let packet = PacketMsg::WhoAmI {};
    let msg = IbcMsg::SendPacket {
        channel_id: channel_id.clone(),
        data: to_binary(&packet)?,
        timeout_block: None,
        timeout_timestamp: build_timeout(&env.block),
    };

    Ok(IbcBasicResponse {
        messages: vec![msg.into()],
        attributes: vec![
            attr("action", "ibc_connect"),
            attr("channel_id", channel_id),
        ],
    })
}

#[entry_point]
/// On closed channel, simply delete the account from our local store
pub fn ibc_channel_close(
    deps: DepsMut,
    _env: Env,
    channel: IbcChannel,
) -> StdResult<IbcBasicResponse> {
    // remove the channel
    let channel_id = channel.endpoint.channel_id;
    accounts(deps.storage).remove(channel_id.as_bytes());

    Ok(IbcBasicResponse {
        messages: vec![],
        attributes: vec![attr("action", "ibc_close"), attr("channel_id", channel_id)],
    })
}

#[entry_point]
/// never should be called as the other side never sends packets
pub fn ibc_packet_receive(
    _deps: DepsMut,
    _env: Env,
    _packet: IbcPacket,
) -> StdResult<IbcReceiveResponse> {
    Ok(IbcReceiveResponse {
        acknowledgement: b"{}".into(),
        messages: vec![],
        attributes: vec![attr("action", "ibc_packet_ack")],
    })
}

#[entry_point]
pub fn ibc_packet_ack(
    deps: DepsMut,
    env: Env,
    ack: IbcAcknowledgement,
) -> StdResult<IbcBasicResponse> {
    // which local channel was this packet send from
    let caller = ack.original_packet.src.channel_id;
    // we need to parse the ack based on our request
    let msg: PacketMsg = from_slice(&ack.original_packet.data)?;
    match msg {
        PacketMsg::Dispatch { .. } => {
            let res: AcknowledgementMsg<DispatchResponse> = from_slice(&ack.acknowledgement)?;
            acknowledge_dispatch(deps, caller, res)
        }
        PacketMsg::WhoAmI {} => {
            let res: AcknowledgementMsg<WhoAmIResponse> = from_slice(&ack.acknowledgement)?;
            acknowledge_who_am_i(deps, caller, res)
        }
        PacketMsg::Balances {} => {
            let res: AcknowledgementMsg<BalancesResponse> = from_slice(&ack.acknowledgement)?;
            acknowledge_balances(deps, env, caller, res)
        }
    }
}

// receive PacketMsg::Dispatch response
fn acknowledge_dispatch(
    _deps: DepsMut,
    _caller: String,
    _ack: AcknowledgementMsg<DispatchResponse>,
) -> StdResult<IbcBasicResponse> {
    // TODO: actually handle success/error?
    Ok(IbcBasicResponse {
        messages: vec![],
        attributes: vec![attr("action", "acknowledge_dispatch")],
    })
}

// receive PacketMsg::WhoAmI response
// store address info in accounts info
fn acknowledge_who_am_i(
    deps: DepsMut,
    caller: String,
    ack: AcknowledgementMsg<WhoAmIResponse>,
) -> StdResult<IbcBasicResponse> {
    // ignore errors (but mention in log)
    let res: WhoAmIResponse = match ack {
        AcknowledgementMsg::Ok(res) => res,
        AcknowledgementMsg::Err(e) => {
            return Ok(IbcBasicResponse {
                messages: vec![],
                attributes: vec![attr("action", "acknowledge_who_am_i"), attr("error", e)],
            })
        }
    };

    accounts(deps.storage).update(caller.as_bytes(), |acct| -> StdResult<_> {
        match acct {
            Some(mut acct) => {
                // set the account the first time
                if acct.remote_addr.is_none() {
                    acct.remote_addr = Some(res.account);
                }
                Ok(acct)
            }
            None => Err(StdError::generic_err("no account to update")),
        }
    })?;

    Ok(IbcBasicResponse {
        messages: vec![],
        attributes: vec![attr("action", "acknowledge_who_am_i")],
    })
}

// receive PacketMsg::Balances response
fn acknowledge_balances(
    deps: DepsMut,
    env: Env,
    caller: String,
    ack: AcknowledgementMsg<BalancesResponse>,
) -> StdResult<IbcBasicResponse> {
    // ignore errors (but mention in log)
    let res: BalancesResponse = match ack {
        AcknowledgementMsg::Ok(res) => res,
        AcknowledgementMsg::Err(e) => {
            return Ok(IbcBasicResponse {
                messages: vec![],
                attributes: vec![attr("action", "acknowledge_balances"), attr("error", e)],
            })
        }
    };

    accounts(deps.storage).update(caller.as_bytes(), |acct| -> StdResult<_> {
        match acct {
            Some(acct) => {
                if let Some(old_addr) = &acct.remote_addr {
                    if old_addr != &res.account {
                        return Err(StdError::generic_err(format!(
                            "remote account changed from {} to {}",
                            old_addr, &res.account
                        )));
                    }
                }
                Ok(AccountData {
                    last_update_time: env.block.time,
                    remote_addr: Some(res.account),
                    remote_balance: res.balances,
                })
            }
            None => Err(StdError::generic_err("no account to update")),
        }
    })?;

    Ok(IbcBasicResponse {
        messages: vec![],
        attributes: vec![attr("action", "acknowledge_balances")],
    })
}

#[entry_point]
/// we just ignore these now. shall we store some info?
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _packet: IbcPacket,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse {
        messages: vec![],
        attributes: vec![attr("action", "ibc_packet_timeout")],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_ibc_channel, mock_ibc_packet_ack, mock_info, MockApi,
        MockQuerier, MockStorage,
    };
    use cosmwasm_std::{coin, coins, BankMsg, OwnedDeps};

    const CREATOR: &str = "creator";

    fn setup() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies(&[]);
        let msg = InitMsg {};
        let info = mock_info(CREATOR, &[]);
        let res = init(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        deps
    }

    // connect will run through the entire handshake to set up a proper connect and
    // save the account (tested in detail in `proper_handshake_flow`)
    fn connect(mut deps: DepsMut, channel_id: &str) {
        // open packet has no counterparty version, connect does
        let mut handshake_open = mock_ibc_channel(channel_id, IbcOrder::Ordered, IBC_VERSION);
        handshake_open.counterparty_version = None;
        // first we try to open with a valid handshake
        ibc_channel_open(deps.branch(), mock_env(), handshake_open).unwrap();

        // then we connect (with counter-party version set)
        let handshake_connect = mock_ibc_channel(channel_id, IbcOrder::Ordered, IBC_VERSION);
        let res = ibc_channel_connect(deps.branch(), mock_env(), handshake_connect).unwrap();

        // this should send a WhoAmI request, which is received some blocks later
        assert_eq!(1, res.messages.len());
        match &res.messages[0] {
            CosmosMsg::Ibc(IbcMsg::SendPacket {
                channel_id: packet_channel,
                ..
            }) => assert_eq!(packet_channel.as_str(), channel_id),
            o => panic!("Unexpected message: {:?}", o),
        };
    }

    fn who_am_i_response<T: Into<HumanAddr>>(deps: DepsMut, channel_id: &str, account: T) {
        let packet = PacketMsg::WhoAmI {};
        let response = AcknowledgementMsg::Ok(WhoAmIResponse {
            account: account.into(),
        });
        let ack = IbcAcknowledgement {
            acknowledgement: to_binary(&response).unwrap(),
            original_packet: mock_ibc_packet_ack(channel_id, &packet).unwrap(),
        };
        let res = ibc_packet_ack(deps, mock_env(), ack).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn init_works() {
        let deps = setup();
        let admin = query_admin(deps.as_ref()).unwrap();
        assert_eq!(CREATOR, admin.admin.as_str());
    }

    #[test]
    fn enforce_version_in_handshake() {
        let mut deps = setup();

        let wrong_order = mock_ibc_channel("channel-12", IbcOrder::Unordered, IBC_VERSION);
        ibc_channel_open(deps.as_mut(), mock_env(), wrong_order).unwrap_err();

        let wrong_version = mock_ibc_channel("channel-12", IbcOrder::Ordered, "reflect");
        ibc_channel_open(deps.as_mut(), mock_env(), wrong_version).unwrap_err();

        let valid_handshake = mock_ibc_channel("channel-12", IbcOrder::Ordered, IBC_VERSION);
        ibc_channel_open(deps.as_mut(), mock_env(), valid_handshake).unwrap();
    }

    #[test]
    fn proper_handshake_flow() {
        // setup and connect handshake
        let mut deps = setup();
        let channel_id = "channel-1234";
        connect(deps.as_mut(), channel_id);

        // check for empty account
        let acct = query_account(deps.as_ref(), channel_id.into()).unwrap();
        assert!(acct.remote_addr.is_none());
        assert!(acct.remote_balance.is_empty());
        assert_eq!(0, acct.last_update_time);

        // now get feedback from WhoAmI packet
        let remote_addr = "account-789";
        who_am_i_response(deps.as_mut(), channel_id, remote_addr);

        // account should be set up
        let acct = query_account(deps.as_ref(), channel_id.into()).unwrap();
        assert_eq!(acct.remote_addr.unwrap(), HumanAddr::from(remote_addr));
        assert!(acct.remote_balance.is_empty());
        assert_eq!(0, acct.last_update_time);
    }

    #[test]
    fn dispatch_message_send_and_ack() {
        let channel_id = "channel-1234";
        let remote_addr = "account-789";

        // init contract
        let mut deps = setup();
        // channel handshake
        connect(deps.as_mut(), channel_id);
        // get feedback from WhoAmI packet
        who_am_i_response(deps.as_mut(), channel_id, remote_addr);

        // try to dispatch a message
        let msgs_to_dispatch = vec![BankMsg::Send {
            to_address: "my-friend".into(),
            amount: coins(123456789, "uatom"),
        }
        .into()];
        let handle_msg = HandleMsg::SendMsgs {
            channel_id: channel_id.into(),
            msgs: msgs_to_dispatch.clone(),
        };
        let info = mock_info(CREATOR, &[]);
        let mut res = handle(deps.as_mut(), mock_env(), info, handle_msg).unwrap();
        assert_eq!(1, res.messages.len());
        let packet = match res.messages.swap_remove(0) {
            CosmosMsg::Ibc(IbcMsg::SendPacket {
                channel_id, data, ..
            }) => {
                let mut packet = mock_ibc_packet_ack(&channel_id, &1).unwrap();
                packet.data = data;
                packet
            }
            o => panic!("Unexpected message: {:?}", o),
        };

        // and handle the ack
        let ack = IbcAcknowledgement {
            acknowledgement: to_binary(&AcknowledgementMsg::Ok(())).unwrap(),
            original_packet: packet,
        };
        let res = ibc_packet_ack(deps.as_mut(), mock_env(), ack).unwrap();
        // no actions expected, but let's check the events to see it was dispatched properly
        assert_eq!(0, res.messages.len());
        assert_eq!(vec![attr("action", "acknowledge_dispatch")], res.attributes)
    }

    #[test]
    fn send_remote_funds() {
        let reflect_channel_id = "channel-1234";
        let remote_addr = "account-789";
        let transfer_channel_id = "transfer-2";

        // init contract
        let mut deps = setup();
        // channel handshake
        connect(deps.as_mut(), reflect_channel_id);
        // get feedback from WhoAmI packet
        who_am_i_response(deps.as_mut(), reflect_channel_id, remote_addr);

        // let's try to send funds to a channel that doesn't exist
        let msg = HandleMsg::SendFunds {
            reflect_channel_id: "random-channel".into(),
            transfer_channel_id: transfer_channel_id.into(),
        };
        let info = mock_info(CREATOR, &coins(12344, "utrgd"));
        handle(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        // let's try with no sent funds in the message
        let msg = HandleMsg::SendFunds {
            reflect_channel_id: reflect_channel_id.into(),
            transfer_channel_id: transfer_channel_id.into(),
        };
        let info = mock_info(CREATOR, &[]);
        handle(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        // 3rd times the charm
        let msg = HandleMsg::SendFunds {
            reflect_channel_id: reflect_channel_id.into(),
            transfer_channel_id: transfer_channel_id.into(),
        };
        let info = mock_info(CREATOR, &coins(12344, "utrgd"));
        let res = handle(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(1, res.messages.len());
        match &res.messages[0] {
            CosmosMsg::Ibc(IbcMsg::Transfer {
                channel_id,
                to_address,
                amount,
                timeout_block,
                timeout_timestamp,
            }) => {
                assert_eq!(transfer_channel_id, channel_id.as_str());
                assert_eq!(remote_addr, to_address.as_str());
                assert_eq!(&coin(12344, "utrgd"), amount);
                assert!(timeout_block.is_none());
                assert!(timeout_timestamp.is_some());
            }
            o => panic!("unexpected message: {:?}", o),
        }
    }
}
