use cosmwasm_std::{
    entry_point, from_slice, to_binary, DepsMut, Env, IbcBasicResponse, IbcChannelCloseMsg,
    IbcChannelConnectMsg, IbcChannelOpenMsg, IbcMsg, IbcOrder, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, StdError, StdResult,
};

use crate::ibc_msg::{
    AcknowledgementMsg, BalancesResponse, DispatchResponse, PacketMsg, WhoAmIResponse,
};
use crate::state::{accounts, AccountData};

pub const IBC_APP_VERSION: &str = "ibc-reflect-v1";

// TODO: make configurable?
/// packets live one hour
pub const PACKET_LIFETIME: u64 = 60 * 60;

#[entry_point]
/// enforces ordering and versioing constraints
pub fn ibc_channel_open(_deps: DepsMut, _env: Env, msg: IbcChannelOpenMsg) -> StdResult<()> {
    let channel = msg.channel();

    if channel.order != IbcOrder::Ordered {
        return Err(StdError::generic_err("Only supports ordered channels"));
    }
    if channel.version.as_str() != IBC_APP_VERSION {
        return Err(StdError::generic_err(format!(
            "Must set version to `{}`",
            IBC_APP_VERSION
        )));
    }

    if let Some(counter_version) = msg.counterparty_version() {
        if counter_version != IBC_APP_VERSION {
            return Err(StdError::generic_err(format!(
                "Counterparty version must be `{}`",
                IBC_APP_VERSION
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
    msg: IbcChannelConnectMsg,
) -> StdResult<IbcBasicResponse> {
    let channel = msg.channel();

    let channel_id = &channel.endpoint.channel_id;

    // create an account holder the channel exists (not found if not registered)
    let data = AccountData::default();
    accounts(deps.storage).save(channel_id.as_bytes(), &data)?;

    // construct a packet to send
    let packet = PacketMsg::WhoAmI {};
    let msg = IbcMsg::SendPacket {
        channel_id: channel_id.clone(),
        data: to_binary(&packet)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    Ok(IbcBasicResponse::new()
        .add_message(msg)
        .add_attribute("action", "ibc_connect")
        .add_attribute("channel_id", channel_id))
}

#[entry_point]
/// On closed channel, simply delete the account from our local store
pub fn ibc_channel_close(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    let channel = msg.channel();

    // remove the channel
    let channel_id = &channel.endpoint.channel_id;
    accounts(deps.storage).remove(channel_id.as_bytes());

    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ibc_close")
        .add_attribute("channel_id", channel_id))
}

#[entry_point]
/// never should be called as the other side never sends packets
pub fn ibc_packet_receive(
    _deps: DepsMut,
    _env: Env,
    _packet: IbcPacketReceiveMsg,
) -> StdResult<IbcReceiveResponse> {
    Ok(IbcReceiveResponse::new()
        .set_ack(b"{}")
        .add_attribute("action", "ibc_packet_ack"))
}

#[entry_point]
pub fn ibc_packet_ack(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
    // which local channel was this packet send from
    let caller = msg.original_packet.src.channel_id;
    // we need to parse the ack based on our request
    let packet: PacketMsg = from_slice(&msg.original_packet.data)?;
    match packet {
        PacketMsg::Dispatch { .. } => {
            let res: AcknowledgementMsg<DispatchResponse> = from_slice(&msg.acknowledgement.data)?;
            acknowledge_dispatch(deps, caller, res)
        }
        PacketMsg::WhoAmI {} => {
            let res: AcknowledgementMsg<WhoAmIResponse> = from_slice(&msg.acknowledgement.data)?;
            acknowledge_who_am_i(deps, caller, res)
        }
        PacketMsg::Balances {} => {
            let res: AcknowledgementMsg<BalancesResponse> = from_slice(&msg.acknowledgement.data)?;
            acknowledge_balances(deps, env, caller, res)
        }
    }
}

// receive PacketMsg::Dispatch response
#[allow(clippy::unnecessary_wraps)]
fn acknowledge_dispatch(
    _deps: DepsMut,
    _caller: String,
    _ack: AcknowledgementMsg<DispatchResponse>,
) -> StdResult<IbcBasicResponse> {
    // TODO: actually handle success/error?
    Ok(IbcBasicResponse::new().add_attribute("action", "acknowledge_dispatch"))
}

// receive PacketMsg::WhoAmI response
// store address info in accounts info
fn acknowledge_who_am_i(
    deps: DepsMut,
    caller: String,
    ack: AcknowledgementMsg<WhoAmIResponse>,
) -> StdResult<IbcBasicResponse> {
    // ignore errors (but mention in log)
    let WhoAmIResponse { account } = match ack {
        AcknowledgementMsg::Ok(res) => res,
        AcknowledgementMsg::Err(e) => {
            return Ok(IbcBasicResponse::new()
                .add_attribute("action", "acknowledge_who_am_i")
                .add_attribute("error", e))
        }
    };

    accounts(deps.storage).update(caller.as_bytes(), |acct| -> StdResult<_> {
        match acct {
            Some(mut acct) => {
                // set the account the first time
                if acct.remote_addr.is_none() {
                    acct.remote_addr = Some(account);
                }
                Ok(acct)
            }
            None => Err(StdError::generic_err("no account to update")),
        }
    })?;

    Ok(IbcBasicResponse::new().add_attribute("action", "acknowledge_who_am_i"))
}

// receive PacketMsg::Balances response
fn acknowledge_balances(
    deps: DepsMut,
    env: Env,
    caller: String,
    ack: AcknowledgementMsg<BalancesResponse>,
) -> StdResult<IbcBasicResponse> {
    // ignore errors (but mention in log)
    let BalancesResponse { account, balances } = match ack {
        AcknowledgementMsg::Ok(res) => res,
        AcknowledgementMsg::Err(e) => {
            return Ok(IbcBasicResponse::new()
                .add_attribute("action", "acknowledge_balances")
                .add_attribute("error", e))
        }
    };

    accounts(deps.storage).update(caller.as_bytes(), |acct| -> StdResult<_> {
        match acct {
            Some(acct) => {
                if let Some(old_addr) = acct.remote_addr {
                    if old_addr != account {
                        return Err(StdError::generic_err(format!(
                            "remote account changed from {} to {}",
                            old_addr, account
                        )));
                    }
                }
                Ok(AccountData {
                    last_update_time: env.block.time,
                    remote_addr: Some(account),
                    remote_balance: balances,
                })
            }
            None => Err(StdError::generic_err("no account to update")),
        }
    })?;

    Ok(IbcBasicResponse::new().add_attribute("action", "acknowledge_balances"))
}

#[entry_point]
/// we just ignore these now. shall we store some info?
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_timeout"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{execute, instantiate, query};
    use crate::msg::{AccountResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_ibc_channel_connect_ack, mock_ibc_channel_open_init,
        mock_ibc_channel_open_try, mock_ibc_packet_ack, mock_info, MockApi, MockQuerier,
        MockStorage,
    };
    use cosmwasm_std::{coin, coins, BankMsg, CosmosMsg, IbcAcknowledgement, OwnedDeps};

    const CREATOR: &str = "creator";

    fn setup() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = mock_info(CREATOR, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        deps
    }

    // connect will run through the entire handshake to set up a proper connect and
    // save the account (tested in detail in `proper_handshake_flow`)
    fn connect(mut deps: DepsMut, channel_id: &str) {
        let handshake_open =
            mock_ibc_channel_open_init(channel_id, IbcOrder::Ordered, IBC_APP_VERSION);
        // first we try to open with a valid handshake
        ibc_channel_open(deps.branch(), mock_env(), handshake_open).unwrap();

        // then we connect (with counter-party version set)
        let handshake_connect =
            mock_ibc_channel_connect_ack(channel_id, IbcOrder::Ordered, IBC_APP_VERSION);
        let res = ibc_channel_connect(deps.branch(), mock_env(), handshake_connect).unwrap();

        // this should send a WhoAmI request, which is received some blocks later
        assert_eq!(1, res.messages.len());
        match &res.messages[0].msg {
            CosmosMsg::Ibc(IbcMsg::SendPacket {
                channel_id: packet_channel,
                ..
            }) => assert_eq!(packet_channel.as_str(), channel_id),
            o => panic!("Unexpected message: {:?}", o),
        };
    }

    fn who_am_i_response(deps: DepsMut, channel_id: &str, account: impl Into<String>) {
        let packet = PacketMsg::WhoAmI {};
        let response = AcknowledgementMsg::Ok(WhoAmIResponse {
            account: account.into(),
        });
        let ack = IbcAcknowledgement::encode_json(&response).unwrap();
        let msg = mock_ibc_packet_ack(channel_id, &packet, ack).unwrap();
        let res = ibc_packet_ack(deps, mock_env(), msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn enforce_version_in_handshake() {
        let mut deps = setup();

        let wrong_order =
            mock_ibc_channel_open_try("channel-12", IbcOrder::Unordered, IBC_APP_VERSION);
        ibc_channel_open(deps.as_mut(), mock_env(), wrong_order).unwrap_err();

        let wrong_version = mock_ibc_channel_open_try("channel-12", IbcOrder::Ordered, "reflect");
        ibc_channel_open(deps.as_mut(), mock_env(), wrong_version).unwrap_err();

        let valid_handshake =
            mock_ibc_channel_open_try("channel-12", IbcOrder::Ordered, IBC_APP_VERSION);
        ibc_channel_open(deps.as_mut(), mock_env(), valid_handshake).unwrap();
    }

    #[test]
    fn proper_handshake_flow() {
        // setup and connect handshake
        let mut deps = setup();
        let channel_id = "channel-1234";
        connect(deps.as_mut(), channel_id);

        // check for empty account
        let q = QueryMsg::Account {
            channel_id: channel_id.into(),
        };
        let r = query(deps.as_ref(), mock_env(), q).unwrap();
        let acct: AccountResponse = from_slice(&r).unwrap();
        assert!(acct.remote_addr.is_none());
        assert!(acct.remote_balance.is_empty());
        assert_eq!(0, acct.last_update_time.nanos());

        // now get feedback from WhoAmI packet
        let remote_addr = "account-789";
        who_am_i_response(deps.as_mut(), channel_id, remote_addr);

        // account should be set up
        let q = QueryMsg::Account {
            channel_id: channel_id.into(),
        };
        let r = query(deps.as_ref(), mock_env(), q).unwrap();
        let acct: AccountResponse = from_slice(&r).unwrap();
        assert_eq!(acct.remote_addr.unwrap(), remote_addr);
        assert!(acct.remote_balance.is_empty());
        assert_eq!(0, acct.last_update_time.nanos());
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
        let handle_msg = ExecuteMsg::SendMsgs {
            channel_id: channel_id.into(),
            msgs: msgs_to_dispatch,
        };
        let info = mock_info(CREATOR, &[]);
        let mut res = execute(deps.as_mut(), mock_env(), info, handle_msg).unwrap();
        assert_eq!(1, res.messages.len());
        let msg = match res.messages.swap_remove(0).msg {
            CosmosMsg::Ibc(IbcMsg::SendPacket {
                channel_id, data, ..
            }) => {
                let ack = IbcAcknowledgement::encode_json(&AcknowledgementMsg::Ok(())).unwrap();
                let mut msg = mock_ibc_packet_ack(&channel_id, &1, ack).unwrap();
                msg.original_packet.data = data;
                msg
            }
            o => panic!("Unexpected message: {:?}", o),
        };
        let res = ibc_packet_ack(deps.as_mut(), mock_env(), msg).unwrap();
        // no actions expected, but let's check the events to see it was dispatched properly
        assert_eq!(0, res.messages.len());
        assert_eq!(vec![("action", "acknowledge_dispatch")], res.attributes)
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
        let msg = ExecuteMsg::SendFunds {
            reflect_channel_id: "random-channel".into(),
            transfer_channel_id: transfer_channel_id.into(),
        };
        let info = mock_info(CREATOR, &coins(12344, "utrgd"));
        execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        // let's try with no sent funds in the message
        let msg = ExecuteMsg::SendFunds {
            reflect_channel_id: reflect_channel_id.into(),
            transfer_channel_id: transfer_channel_id.into(),
        };
        let info = mock_info(CREATOR, &[]);
        execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        // 3rd times the charm
        let msg = ExecuteMsg::SendFunds {
            reflect_channel_id: reflect_channel_id.into(),
            transfer_channel_id: transfer_channel_id.into(),
        };
        let info = mock_info(CREATOR, &coins(12344, "utrgd"));
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(1, res.messages.len());
        match &res.messages[0].msg {
            CosmosMsg::Ibc(IbcMsg::Transfer {
                channel_id,
                to_address,
                amount,
                timeout,
                memo,
            }) => {
                assert_eq!(transfer_channel_id, channel_id.as_str());
                assert_eq!(remote_addr, to_address.as_str());
                assert_eq!(&coin(12344, "utrgd"), amount);
                assert!(timeout.block().is_none());
                assert!(timeout.timestamp().is_some());
            }
            o => panic!("unexpected message: {:?}", o),
        }
    }
}
