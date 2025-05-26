use cosmwasm_std::{
    entry_point, from_json, to_json_binary, wasm_execute, Binary, CosmosMsg, Deps, DepsMut, Empty,
    Env, Event, Ibc3ChannelOpenResponse, IbcAcknowledgement, IbcBasicResponse, IbcChannelCloseMsg,
    IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse, IbcMsg, IbcOrder,
    IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, MessageInfo,
    Never, QueryResponse, Reply, Response, StdError, StdResult, SubMsg, SubMsgResponse,
    SubMsgResult, WasmMsg,
};

use crate::msg::{
    AccountInfo, AccountResponse, AcknowledgementMsg, BalanceResponse, DispatchResponse,
    ExecuteMsg, InstantiateMsg, ListAccountsResponse, PacketMsg, QueryMsg, ReflectExecuteMsg,
    ReturnMsgsResponse, WhoAmIResponse,
};
use crate::state::{
    load_account, load_item, may_load_account, range_accounts, remove_account, save_account,
    save_item, Config, KEY_CONFIG, KEY_PENDING_CHANNEL,
};

pub const IBC_APP_VERSION: &str = "ibc-reflect-v1";
pub const RECEIVE_DISPATCH_ID: u64 = 1234;
pub const INIT_CALLBACK_ID: u64 = 7890;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    // we store the reflect_id for creating accounts later
    let cfg = Config {
        reflect_code_id: msg.reflect_code_id,
    };
    save_item(deps.storage, KEY_CONFIG, &cfg)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[entry_point]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::AsyncAck {
            channel_id,
            packet_sequence,
            ack,
        } => execute_async_ack(channel_id, packet_sequence.u64(), ack),
    }
}

fn execute_async_ack(
    channel_id: String,
    packet_sequence: u64,
    ack: IbcAcknowledgement,
) -> StdResult<Response> {
    Ok(Response::new().add_message(IbcMsg::WriteAcknowledgement {
        channel_id,
        packet_sequence,
        ack,
    }))
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> StdResult<Response> {
    match (reply.id, reply.result) {
        (RECEIVE_DISPATCH_ID, SubMsgResult::Err(err)) => {
            Ok(Response::new().set_data(encode_ibc_error(err)))
        }
        (INIT_CALLBACK_ID, SubMsgResult::Ok(response)) => handle_init_callback(deps, response),
        _ => Err(StdError::generic_err("invalid reply id or result")),
    }
}

// updated with https://github.com/CosmWasm/wasmd/pull/586 (emitted in keeper.Instantiate)
fn parse_contract_from_event(events: Vec<Event>) -> Option<String> {
    events
        .into_iter()
        .find(|e| e.ty == "instantiate")
        .and_then(|ev| {
            ev.attributes
                .into_iter()
                .find(|a| a.key == "_contract_address")
        })
        .map(|a| a.value)
}

pub fn handle_init_callback(deps: DepsMut, response: SubMsgResponse) -> StdResult<Response> {
    // we use storage to pass info from the caller to the reply
    let id: String = load_item(deps.storage, KEY_PENDING_CHANNEL)?;
    deps.storage.remove(KEY_PENDING_CHANNEL);

    // parse contract info from events
    let contract_addr = match parse_contract_from_event(response.events) {
        Some(addr) => deps.api.addr_validate(&addr),
        None => Err(StdError::generic_err(
            "No _contract_address found in callback events",
        )),
    }?;

    // store id -> contract_addr if it is empty
    // id comes from: `let chan_id = msg.endpoint.channel_id;` in `ibc_channel_connect`
    match may_load_account(deps.storage, &id)? {
        Some(_) => {
            return Err(StdError::generic_err(
                "Cannot register over an existing channel",
            ))
        }
        None => save_account(deps.storage, &id, &contract_addr)?,
    }

    Ok(Response::new().add_attribute("action", "execute_init_callback"))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Account { channel_id } => to_json_binary(&query_account(deps, channel_id)?),
        QueryMsg::ListAccounts {} => to_json_binary(&query_list_accounts(deps)?),
    }
}

pub fn query_account(deps: Deps, channel_id: String) -> StdResult<AccountResponse> {
    let account = load_account(deps.storage, &channel_id)?;
    Ok(AccountResponse {
        account: Some(account.into()),
    })
}

pub fn query_list_accounts(deps: Deps) -> StdResult<ListAccountsResponse> {
    let accounts: StdResult<Vec<_>> = range_accounts(deps.storage)
        .map(|item| {
            let (key, account) = item?;
            Ok(AccountInfo {
                account: account.into(),
                channel_id: key,
            })
        })
        .collect();
    Ok(ListAccountsResponse {
        accounts: accounts?,
    })
}

#[entry_point]
/// enforces ordering and versioning constraints
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> StdResult<IbcChannelOpenResponse> {
    let channel = msg.channel();

    if channel.order != IbcOrder::Ordered {
        return Err(StdError::generic_err("Only supports ordered channels"));
    }

    // In ibcv3 we don't check the version string passed in the message
    // and only check the counterparty version.
    if let Some(counter_version) = msg.counterparty_version() {
        if counter_version != IBC_APP_VERSION {
            return Err(StdError::generic_err(format!(
                "Counterparty version must be `{IBC_APP_VERSION}`"
            )));
        }
    }

    // We return the version we need (which could be different than the counterparty version)
    Ok(Some(Ibc3ChannelOpenResponse {
        version: IBC_APP_VERSION.to_string(),
    }))
}

#[entry_point]
/// once it's established, we create the reflect contract
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> StdResult<IbcBasicResponse> {
    let channel = msg.channel();
    let cfg: Config = load_item(deps.storage, KEY_CONFIG)?;
    let chan_id = &channel.endpoint.channel_id;

    let msg = WasmMsg::Instantiate {
        admin: None,
        code_id: cfg.reflect_code_id,
        msg: b"{}".into(),
        funds: vec![],
        label: format!("ibc-reflect-{chan_id}"),
    };
    let msg = SubMsg::reply_on_success(msg, INIT_CALLBACK_ID);

    // store the channel id for the reply handler
    save_item(deps.storage, KEY_PENDING_CHANNEL, chan_id)?;

    Ok(IbcBasicResponse::new()
        .add_submessage(msg)
        .add_attribute("action", "ibc_connect")
        .add_attribute("channel_id", chan_id)
        .add_event(Event::new("ibc").add_attribute("channel", "connect")))
}

#[entry_point]
/// On closed channel, we delete the channel entry from accounts.
pub fn ibc_channel_close(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    let channel = msg.channel();
    // get contract address and remove lookup
    let channel_id = channel.endpoint.channel_id.as_str();
    remove_account(deps.storage, channel_id);

    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ibc_close")
        .add_attribute("channel_id", channel_id))
}

/// this is a no-op just to test how this integrates with wasmd
#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: Empty) -> StdResult<Response> {
    Ok(Response::default())
}

// this encode an error or error message into a proper acknowledgement to the receiver
fn encode_ibc_error(msg: impl Into<String>) -> Binary {
    // this cannot error, unwrap to keep the interface simple
    to_json_binary(&AcknowledgementMsg::<()>::Error(msg.into())).unwrap()
}

#[entry_point]
/// we look for the proper reflect contract to relay to and send the message
/// We cannot return any meaningful response value as we do not know the response value
/// of execution. We just return ok if we dispatched, error if we failed to dispatch
pub fn ibc_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, Never> {
    // put this in a closure so we can convert all error responses into acknowledgements
    (|| {
        let packet = msg.packet;
        // which local channel did this packet come on
        let caller = packet.dest.channel_id;
        let msg: PacketMsg = from_json(packet.data)?;
        match msg {
            PacketMsg::Dispatch { msgs } => receive_dispatch(deps, caller, msgs),
            PacketMsg::WhoAmI {} => receive_who_am_i(deps, caller),
            PacketMsg::Balance { denom } => receive_balance(deps, caller, denom),
            PacketMsg::Panic {} => execute_panic(),
            PacketMsg::ReturnErr { text } => execute_error(text),
            PacketMsg::ReturnMsgs { msgs } => execute_return_msgs(msgs),
            PacketMsg::NoAck {} => Ok(IbcReceiveResponse::without_ack()),
        }
    })()
    .or_else(|e| {
        // we try to capture all app-level errors and convert them into
        // acknowledgement packets that contain an error code.
        let acknowledgement = encode_ibc_error(format!("invalid packet: {e}"));
        Ok(IbcReceiveResponse::new(acknowledgement)
            .add_event(Event::new("ibc").add_attribute("packet", "receive")))
    })
}

// processes PacketMsg::WhoAmI variant
fn receive_who_am_i(deps: DepsMut, caller: String) -> StdResult<IbcReceiveResponse> {
    let account = load_account(deps.storage, &caller)?;
    let response = WhoAmIResponse {
        account: account.into(),
    };
    let acknowledgement = to_json_binary(&AcknowledgementMsg::Ok(response))?;
    // and we are golden
    Ok(IbcReceiveResponse::new(acknowledgement).add_attribute("action", "receive_who_am_i"))
}

// processes PacketMsg::Balance variant
fn receive_balance(deps: DepsMut, caller: String, denom: String) -> StdResult<IbcReceiveResponse> {
    let account = load_account(deps.storage, &caller)?;
    let balance = deps.querier.query_balance(&account, denom)?;
    let response = BalanceResponse {
        account: account.into(),
        balance,
    };
    let acknowledgement = to_json_binary(&AcknowledgementMsg::Ok(response))?;
    // and we are golden
    Ok(IbcReceiveResponse::new(acknowledgement).add_attribute("action", "receive_balance"))
}

// processes PacketMsg::Dispatch variant
fn receive_dispatch(
    deps: DepsMut,
    caller: String,
    msgs: Vec<CosmosMsg>,
) -> StdResult<IbcReceiveResponse> {
    // what is the reflect contract here
    let reflect_addr = load_account(deps.storage, &caller)?;

    // let them know we're fine
    let acknowledgement = to_json_binary(&AcknowledgementMsg::<DispatchResponse>::Ok(()))?;
    // create the message to re-dispatch to the reflect contract
    let reflect_msg = ReflectExecuteMsg::ReflectMsg { msgs };
    let wasm_msg = wasm_execute(reflect_addr, &reflect_msg, vec![])?;

    // we wrap it in a submessage to properly report errors
    let msg = SubMsg::reply_on_error(wasm_msg, RECEIVE_DISPATCH_ID);

    Ok(IbcReceiveResponse::new(acknowledgement)
        .add_submessage(msg)
        .add_attribute("action", "receive_dispatch"))
}

fn execute_panic() -> StdResult<IbcReceiveResponse> {
    panic!("This page intentionally faulted");
}

fn execute_error(text: String) -> StdResult<IbcReceiveResponse> {
    Err(StdError::generic_err(text))
}

fn execute_return_msgs(msgs: Vec<CosmosMsg>) -> StdResult<IbcReceiveResponse> {
    let acknowledgement = to_json_binary(&AcknowledgementMsg::<ReturnMsgsResponse>::Ok(()))?;

    Ok(IbcReceiveResponse::new(acknowledgement)
        .add_messages(msgs)
        .add_attribute("action", "receive_dispatch"))
}

#[entry_point]
/// never should be called as we do not send packets
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_ack"))
}

#[entry_point]
/// never should be called as we do not send packets
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
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, mock_ibc_channel_close_init,
        mock_ibc_channel_connect_ack, mock_ibc_channel_open_init, mock_ibc_channel_open_try,
        mock_ibc_packet_recv, mock_wasmd_attr, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{attr, coin, coins, from_json, BankMsg, OwnedDeps, WasmMsg};

    const CREATOR: &str = "creator";
    // code id of the reflect contract
    const REFLECT_ID: u64 = 101;
    // address of first reflect contract instance that we created
    const REFLECT_ADDR: &str = "reflect-acct-1";

    fn setup() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();
        let creator = deps.api.addr_make(CREATOR);
        let msg = InstantiateMsg {
            reflect_code_id: REFLECT_ID,
        };
        let info = message_info(&creator, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 0);
        deps
    }

    fn fake_events(reflect_addr: &str) -> Vec<Event> {
        let event = Event::new("instantiate").add_attributes(vec![
            attr("code_id", "17"),
            // We have to force this one to avoid the debug assertion against _
            mock_wasmd_attr("_contract_address", reflect_addr),
        ]);
        vec![event]
    }

    // connect will run through the entire handshake to set up a proper connect and
    // save the account (tested in detail in `proper_handshake_flow`)
    fn connect(mut deps: DepsMut, channel_id: &str, account: impl Into<String>) {
        let account: String = account.into();

        let handshake_open =
            mock_ibc_channel_open_init(channel_id, IbcOrder::Ordered, IBC_APP_VERSION);
        // first we try to open with a valid handshake
        ibc_channel_open(deps.branch(), mock_env(), handshake_open).unwrap();

        // then we connect (with counter-party version set)
        let handshake_connect =
            mock_ibc_channel_connect_ack(channel_id, IbcOrder::Ordered, IBC_APP_VERSION);
        let res = ibc_channel_connect(deps.branch(), mock_env(), handshake_connect).unwrap();
        assert_eq!(1, res.messages.len());
        assert_eq!(1, res.events.len());
        assert_eq!(
            Event::new("ibc").add_attribute("channel", "connect"),
            res.events[0]
        );
        let id = res.messages[0].id;
        let payload = res.messages[0].payload.clone();

        // fake a reply and ensure this works
        #[allow(deprecated)]
        let response = Reply {
            id,
            payload,
            gas_used: 1234567,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: fake_events(&account),
                msg_responses: vec![],
                data: None,
            }),
        };
        reply(deps.branch(), mock_env(), response).unwrap();
    }

    #[test]
    fn instantiate_works() {
        let mut deps = mock_dependencies();
        let creator = deps.api.addr_make(CREATOR);

        let msg = InstantiateMsg {
            reflect_code_id: 17,
        };
        let info = message_info(&creator, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len())
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
        let mut deps = setup();
        let channel_id = "channel-1234";
        let reflect_addr = deps.api.addr_make(REFLECT_ADDR);

        // first we try to open with a valid handshake
        let handshake_open =
            mock_ibc_channel_open_init(channel_id, IbcOrder::Ordered, IBC_APP_VERSION);
        ibc_channel_open(deps.as_mut(), mock_env(), handshake_open).unwrap();

        // then we connect (with counter-party version set)
        let handshake_connect =
            mock_ibc_channel_connect_ack(channel_id, IbcOrder::Ordered, IBC_APP_VERSION);
        let res = ibc_channel_connect(deps.as_mut(), mock_env(), handshake_connect).unwrap();
        // and set up a reflect account
        assert_eq!(1, res.messages.len());
        let id = res.messages[0].id;
        let payload = res.messages[0].payload.clone();
        if let CosmosMsg::Wasm(WasmMsg::Instantiate {
            admin,
            code_id,
            msg: _,
            funds,
            label,
        }) = &res.messages[0].msg
        {
            assert_eq!(*admin, None);
            assert_eq!(*code_id, REFLECT_ID);
            assert_eq!(funds.len(), 0);
            assert!(label.contains(channel_id));
        } else {
            panic!("invalid return message: {:?}", res.messages[0]);
        }

        // no accounts set yet
        let raw = query(deps.as_ref(), mock_env(), QueryMsg::ListAccounts {}).unwrap();
        let res: ListAccountsResponse = from_json(raw).unwrap();
        assert_eq!(0, res.accounts.len());

        // fake a reply and ensure this works
        #[allow(deprecated)]
        let response = Reply {
            id,
            payload,
            gas_used: 1234567,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: fake_events(reflect_addr.as_str()),
                msg_responses: vec![],
                data: None,
            }),
        };
        reply(deps.as_mut(), mock_env(), response).unwrap();

        // ensure this is now registered
        let raw = query(deps.as_ref(), mock_env(), QueryMsg::ListAccounts {}).unwrap();
        let res: ListAccountsResponse = from_json(raw).unwrap();
        assert_eq!(1, res.accounts.len());
        assert_eq!(
            &res.accounts[0],
            &AccountInfo {
                account: reflect_addr.to_string(),
                channel_id: channel_id.to_string(),
            }
        );

        // and the account query also works
        let raw = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Account {
                channel_id: channel_id.to_string(),
            },
        )
        .unwrap();
        let res: AccountResponse = from_json(raw).unwrap();
        assert_eq!(res.account.unwrap(), reflect_addr.as_str());
    }

    #[test]
    fn handle_dispatch_packet() {
        let mut deps = setup();

        let channel_id = "channel-123";
        let account = deps.api.addr_make("acct-123");

        // receive a packet for an unregistered channel returns app-level error (not Result::Err)
        let msgs_to_dispatch = vec![BankMsg::Send {
            to_address: "my-friend".into(),
            amount: coins(123456789, "uatom"),
        }
        .into()];
        let ibc_msg = PacketMsg::Dispatch {
            msgs: msgs_to_dispatch.clone(),
        };
        let msg = mock_ibc_packet_recv(channel_id, &ibc_msg).unwrap();
        let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
        // we didn't dispatch anything
        assert_eq!(0, res.messages.len());
        assert_eq!(1, res.events.len());
        assert_eq!(
            Event::new("ibc").add_attribute("packet", "receive"),
            res.events[0]
        );
        // acknowledgement is an error
        let ack: AcknowledgementMsg<DispatchResponse> =
            from_json(res.acknowledgement.unwrap()).unwrap();
        assert_eq!(
            ack.unwrap_err(),
            "invalid packet: account channel-123 not found"
        );

        // register the channel
        connect(deps.as_mut(), channel_id, &account);

        // receive a packet for an unregistered channel returns app-level error (not Result::Err)
        let msg = mock_ibc_packet_recv(channel_id, &ibc_msg).unwrap();
        let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();

        // assert app-level success
        let ack: AcknowledgementMsg<()> = from_json(res.acknowledgement.unwrap()).unwrap();
        ack.unwrap();

        // and we dispatch the BankMsg via submessage
        assert_eq!(1, res.messages.len());
        assert_eq!(RECEIVE_DISPATCH_ID, res.messages[0].id);

        // parse the output, ensuring it matches
        if let CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            msg,
            funds,
        }) = &res.messages[0].msg
        {
            assert_eq!(account.as_str(), contract_addr);
            assert_eq!(0, funds.len());
            // parse the message - should callback with proper channel_id
            let rmsg: ReflectExecuteMsg = from_json(msg).unwrap();
            assert_eq!(
                rmsg,
                ReflectExecuteMsg::ReflectMsg {
                    msgs: msgs_to_dispatch
                }
            );
        } else {
            panic!("invalid return message: {:?}", res.messages[0]);
        }

        // invalid packet format on registered channel also returns app-level error
        let bad_data = InstantiateMsg {
            reflect_code_id: 12345,
        };
        let msg = mock_ibc_packet_recv(channel_id, &bad_data).unwrap();
        let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
        // we didn't dispatch anything
        assert_eq!(0, res.messages.len());
        // acknowledgement is an error
        let ack: AcknowledgementMsg<DispatchResponse> =
            from_json(res.acknowledgement.unwrap()).unwrap();
        assert_eq!(ack.unwrap_err(), "invalid packet: Error parsing into type ibc_reflect::msg::PacketMsg: unknown variant `reflect_code_id`, expected one of `dispatch`, `who_am_i`, `balance`, `panic`, `return_err`, `return_msgs`, `no_ack` at line 1 column 18");
    }

    #[test]
    fn check_close_channel() {
        let mut deps = setup();

        let channel_id = "channel-123";
        let account = deps.api.addr_make("acct-123");

        // register the channel
        connect(deps.as_mut(), channel_id, &account);
        // assign it some funds
        let funds = vec![coin(123456, "uatom"), coin(7654321, "tgrd")];
        deps.querier.bank.update_balance(&account, funds.clone());

        // channel should be listed and have balance
        let raw = query(deps.as_ref(), mock_env(), QueryMsg::ListAccounts {}).unwrap();
        let res: ListAccountsResponse = from_json(raw).unwrap();
        assert_eq!(1, res.accounts.len());

        // close the channel
        let channel = mock_ibc_channel_close_init(channel_id, IbcOrder::Ordered, IBC_APP_VERSION);
        let res = ibc_channel_close(deps.as_mut(), mock_env(), channel).unwrap();
        assert_eq!(res.messages.len(), 0);

        // and removes the account lookup
        let raw = query(deps.as_ref(), mock_env(), QueryMsg::ListAccounts {}).unwrap();
        let res: ListAccountsResponse = from_json(raw).unwrap();
        assert_eq!(0, res.accounts.len());
    }
}
