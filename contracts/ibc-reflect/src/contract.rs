use cosmwasm_std::{
    attr, entry_point, from_slice, to_binary, wasm_execute, wasm_instantiate, BankMsg, Binary,
    ContractResult, CosmosMsg, Deps, DepsMut, Empty, Env, Event, IbcAcknowledgement,
    IbcBasicResponse, IbcChannel, IbcOrder, IbcPacket, IbcReceiveResponse, MessageInfo, Order,
    QueryResponse, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, SubcallResponse,
};

use crate::msg::{
    AccountInfo, AccountResponse, AcknowledgementMsg, BalancesResponse, DispatchResponse,
    InstantiateMsg, ListAccountsResponse, PacketMsg, QueryMsg, ReflectExecuteMsg,
    ReflectInstantiateMsg, WhoAmIResponse,
};
use crate::state::{accounts, accounts_read, config, pending_channel, Config};

pub const IBC_VERSION: &str = "ibc-reflect-v1";
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
    config(deps.storage).save(&cfg)?;

    Ok(Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![attr("action", "instantiate")],
        data: None,
    })
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> StdResult<Response> {
    match (reply.id, reply.result) {
        (RECEIVE_DISPATCH_ID, ContractResult::Err(err)) => Ok(Response {
            data: Some(encode_ibc_error(err)),
            ..Response::default()
        }),
        (INIT_CALLBACK_ID, ContractResult::Ok(response)) => handle_init_callback(deps, response),
        _ => Err(StdError::generic_err("invalid reply id or result")),
    }
}

// see https://github.com/CosmWasm/wasmd/blob/408bba14a5c6d583abe32ffb235a364130136298/x/wasm/keeper/msg_server.go#L63-L69
fn parse_contract_from_event(events: Vec<Event>) -> Option<String> {
    events
        .into_iter()
        .find(|e| e.kind == "message")
        .and_then(|ev| {
            ev.attributes
                .into_iter()
                .find(|a| a.key == "contract_address")
        })
        .map(|a| a.value)
}

pub fn handle_init_callback(deps: DepsMut, response: SubcallResponse) -> StdResult<Response> {
    // we use storage to pass info from the caller to the reply
    let id = pending_channel(deps.storage).load()?;
    pending_channel(deps.storage).remove();

    // parse contract info from events
    let contract_addr = match parse_contract_from_event(response.events) {
        Some(addr) => deps.api.addr_validate(&addr),
        None => Err(StdError::generic_err(
            "No contract_address found in callback events",
        )),
    }?;

    // store id -> contract_addr if it is empty
    // id comes from: `let chan_id = msg.endpoint.channel_id;` in `ibc_channel_connect`
    accounts(deps.storage).update(id.as_bytes(), |val| -> StdResult<_> {
        match val {
            Some(_) => Err(StdError::generic_err(
                "Cannot register over an existing channel",
            )),
            None => Ok(contract_addr),
        }
    })?;

    Ok(Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![attr("action", "execute_init_callback")],
        data: None,
    })
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Account { channel_id } => to_binary(&query_account(deps, channel_id)?),
        QueryMsg::ListAccounts {} => to_binary(&query_list_accounts(deps)?),
    }
}

pub fn query_account(deps: Deps, channel_id: String) -> StdResult<AccountResponse> {
    let account = accounts_read(deps.storage).load(channel_id.as_bytes())?;
    Ok(AccountResponse {
        account: Some(account.into()),
    })
}

pub fn query_list_accounts(deps: Deps) -> StdResult<ListAccountsResponse> {
    let accounts: StdResult<Vec<_>> = accounts_read(deps.storage)
        .range(None, None, Order::Ascending)
        .map(|item| {
            let (key, account) = item?;
            Ok(AccountInfo {
                account: account.into(),
                channel_id: String::from_utf8(key)?,
            })
        })
        .collect();
    Ok(ListAccountsResponse {
        accounts: accounts?,
    })
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
/// once it's established, we create the reflect contract
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    channel: IbcChannel,
) -> StdResult<IbcBasicResponse> {
    let cfg = config(deps.storage).load()?;
    let chan_id = channel.endpoint.channel_id;

    let label = format!("ibc-reflect-{}", &chan_id);
    let payload = ReflectInstantiateMsg {
        callback_id: Some(chan_id.clone()),
    };
    let msg = wasm_instantiate(cfg.reflect_code_id, &payload, vec![], label)?;

    let sub_msg = SubMsg {
        id: INIT_CALLBACK_ID,
        msg: msg.into(),
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    // store the channel id for the reply handler
    pending_channel(deps.storage).save(&chan_id)?;

    Ok(IbcBasicResponse {
        messages: vec![],
        submessages: vec![sub_msg],
        attributes: vec![attr("action", "ibc_connect"), attr("channel_id", chan_id)],
    })
}

#[entry_point]
/// On closed channel, we take all tokens from reflect contract to this contract.
/// We also delete the channel entry from accounts.
pub fn ibc_channel_close(
    deps: DepsMut,
    env: Env,
    channel: IbcChannel,
) -> StdResult<IbcBasicResponse> {
    // get contract address and remove lookup
    let channel_id = channel.endpoint.channel_id.as_str();
    let reflect_addr = accounts(deps.storage).load(channel_id.as_bytes())?;
    accounts(deps.storage).remove(channel_id.as_bytes());

    // transfer current balance if any (steal the money)
    let amount = deps.querier.query_all_balances(&reflect_addr)?;
    let messages: Vec<CosmosMsg<Empty>> = if !amount.is_empty() {
        let bank_msg = BankMsg::Send {
            to_address: env.contract.address.into(),
            amount,
        };
        let reflect_msg = ReflectExecuteMsg::ReflectMsg {
            msgs: vec![bank_msg.into()],
        };
        let wasm_msg = wasm_execute(reflect_addr, &reflect_msg, vec![])?;
        vec![wasm_msg.into()]
    } else {
        vec![]
    };
    let steal_funds = !messages.is_empty();

    Ok(IbcBasicResponse {
        submessages: vec![],
        messages,
        attributes: vec![
            attr("action", "ibc_close"),
            attr("channel_id", channel_id),
            attr("steal_funds", steal_funds),
        ],
    })
}

/// this is a no-op just to test how this integrates with wasmd
#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: Empty) -> StdResult<Response> {
    Ok(Response::default())
}

// this encode an error or error message into a proper acknowledgement to the recevier
fn encode_ibc_error<T: Into<String>>(msg: T) -> Binary {
    // this cannot error, unwrap to keep the interface simple
    to_binary(&AcknowledgementMsg::<()>::Err(msg.into())).unwrap()
}

#[entry_point]
/// we look for a the proper reflect contract to relay to and send the message
/// We cannot return any meaningful response value as we do not know the response value
/// of execution. We just return ok if we dispatched, error if we failed to dispatch
pub fn ibc_packet_receive(
    deps: DepsMut,
    _env: Env,
    packet: IbcPacket,
) -> StdResult<IbcReceiveResponse> {
    // put this in a closure so we can convert all error responses into acknowledgements
    (|| {
        // which local channel did this packet come on
        let caller = packet.dest.channel_id;
        let msg: PacketMsg = from_slice(&packet.data)?;
        match msg {
            PacketMsg::Dispatch { msgs } => receive_dispatch(deps, caller, msgs),
            PacketMsg::WhoAmI {} => receive_who_am_i(deps, caller),
            PacketMsg::Balances {} => receive_balances(deps, caller),
        }
    })()
    .or_else(|e| {
        // we try to capture all app-level errors and convert them into
        // acknowledgement packets that contain an error code.
        let acknowledgement = encode_ibc_error(format!("invalid packet: {}", e));
        Ok(IbcReceiveResponse {
            acknowledgement,
            submessages: vec![],
            messages: vec![],
            attributes: vec![],
        })
    })
}

// processes PacketMsg::WhoAmI variant
fn receive_who_am_i(deps: DepsMut, caller: String) -> StdResult<IbcReceiveResponse> {
    let account = accounts(deps.storage).load(caller.as_bytes())?;
    let response = WhoAmIResponse {
        account: account.into(),
    };
    let acknowledgement = to_binary(&AcknowledgementMsg::Ok(response))?;
    // and we are golden
    Ok(IbcReceiveResponse {
        acknowledgement,
        submessages: vec![],
        messages: vec![],
        attributes: vec![attr("action", "receive_who_am_i")],
    })
}

// processes PacketMsg::Balances variant
fn receive_balances(deps: DepsMut, caller: String) -> StdResult<IbcReceiveResponse> {
    let account = accounts(deps.storage).load(caller.as_bytes())?;
    let balances = deps.querier.query_all_balances(&account)?;
    let response = BalancesResponse {
        account: account.into(),
        balances,
    };
    let acknowledgement = to_binary(&AcknowledgementMsg::Ok(response))?;
    // and we are golden
    Ok(IbcReceiveResponse {
        acknowledgement,
        submessages: vec![],
        messages: vec![],
        attributes: vec![attr("action", "receive_balances")],
    })
}

// processes PacketMsg::Dispatch variant
fn receive_dispatch(
    deps: DepsMut,
    caller: String,
    msgs: Vec<CosmosMsg>,
) -> StdResult<IbcReceiveResponse> {
    // what is the reflect contract here
    let reflect_addr = accounts(deps.storage).load(caller.as_bytes())?;

    // let them know we're fine
    let acknowledgement = to_binary(&AcknowledgementMsg::<DispatchResponse>::Ok(()))?;
    // create the message to re-dispatch to the reflect contract
    let reflect_msg = ReflectExecuteMsg::ReflectMsg { msgs };
    let wasm_msg = wasm_execute(reflect_addr, &reflect_msg, vec![])?;

    // we wrap it in a submessage to properly report errors
    let sub_msg = SubMsg {
        id: RECEIVE_DISPATCH_ID,
        msg: wasm_msg.into(),
        gas_limit: None,
        reply_on: ReplyOn::Error,
    };

    Ok(IbcReceiveResponse {
        acknowledgement,
        submessages: vec![sub_msg],
        messages: vec![],
        attributes: vec![attr("action", "receive_dispatch")],
    })
}

#[entry_point]
/// never should be called as we do not send packets
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    _ack: IbcAcknowledgement,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse {
        submessages: vec![],
        messages: vec![],
        attributes: vec![attr("action", "ibc_packet_ack")],
    })
}

#[entry_point]
/// never should be called as we do not send packets
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _packet: IbcPacket,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse {
        submessages: vec![],
        messages: vec![],
        attributes: vec![attr("action", "ibc_packet_timeout")],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_ibc_channel, mock_ibc_packet_recv, mock_info, MockApi,
        MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
    };
    use cosmwasm_std::{coin, coins, from_slice, BankMsg, OwnedDeps, WasmMsg};

    const CREATOR: &str = "creator";
    // code id of the reflect contract
    const REFLECT_ID: u64 = 101;
    // address of first reflect contract instance that we created
    const REFLECT_ADDR: &str = "reflect-acct-1";

    fn setup() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg {
            reflect_code_id: REFLECT_ID,
        };
        let info = mock_info(CREATOR, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        deps
    }

    fn fake_events(reflect_addr: &str) -> Vec<Event> {
        let event = Event {
            kind: "message".into(),
            attributes: vec![
                attr("module", "wasm"),
                attr("signer", MOCK_CONTRACT_ADDR),
                attr("code_id", "17"),
                attr("contract_address", reflect_addr),
            ],
        };
        vec![event]
    }

    // connect will run through the entire handshake to set up a proper connect and
    // save the account (tested in detail in `proper_handshake_flow`)
    fn connect<T: Into<String>>(mut deps: DepsMut, channel_id: &str, account: T) {
        let account: String = account.into();

        // open packet has no counterparty versin, connect does
        // TODO: validate this with alpe
        let mut handshake_open = mock_ibc_channel(channel_id, IbcOrder::Ordered, IBC_VERSION);
        handshake_open.counterparty_version = None;
        // first we try to open with a valid handshake
        ibc_channel_open(deps.branch(), mock_env(), handshake_open).unwrap();

        // then we connect (with counter-party version set)
        let handshake_connect = mock_ibc_channel(channel_id, IbcOrder::Ordered, IBC_VERSION);
        let res = ibc_channel_connect(deps.branch(), mock_env(), handshake_connect).unwrap();
        assert_eq!(1, res.submessages.len());
        let id = res.submessages[0].id;

        // fake a reply and ensure this works
        let response = Reply {
            id,
            result: ContractResult::Ok(SubcallResponse {
                events: fake_events(&account),
                data: None,
            }),
        };
        reply(deps.branch(), mock_env(), response).unwrap();
    }

    #[test]
    fn instantiate_works() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            reflect_code_id: 17,
        };
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len())
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
        let mut deps = setup();
        let channel_id = "channel-1234";

        // first we try to open with a valid handshake
        let mut handshake_open = mock_ibc_channel(channel_id, IbcOrder::Ordered, IBC_VERSION);
        handshake_open.counterparty_version = None;
        ibc_channel_open(deps.as_mut(), mock_env(), handshake_open).unwrap();

        // then we connect (with counter-party version set)
        let handshake_connect = mock_ibc_channel(channel_id, IbcOrder::Ordered, IBC_VERSION);
        let res = ibc_channel_connect(deps.as_mut(), mock_env(), handshake_connect).unwrap();
        // and set up a reflect account
        assert_eq!(1, res.submessages.len());
        let id = res.submessages[0].id;
        if let CosmosMsg::Wasm(WasmMsg::Instantiate {
            admin,
            code_id,
            msg,
            send,
            label,
        }) = &res.submessages[0].msg
        {
            assert_eq!(*admin, None);
            assert_eq!(*code_id, REFLECT_ID);
            assert_eq!(send.len(), 0);
            assert!(label.contains(channel_id));
            // parse the message - should callback with proper channel_id
            let rmsg: ReflectInstantiateMsg = from_slice(&msg).unwrap();
            assert_eq!(rmsg.callback_id, Some(channel_id.to_string()));
        } else {
            panic!("invalid return message: {:?}", res.messages[0]);
        }

        // no accounts set yet
        let raw = query(deps.as_ref(), mock_env(), QueryMsg::ListAccounts {}).unwrap();
        let res: ListAccountsResponse = from_slice(&raw).unwrap();
        assert_eq!(0, res.accounts.len());

        // fake a reply and ensure this works
        let response = Reply {
            id,
            result: ContractResult::Ok(SubcallResponse {
                events: fake_events(&REFLECT_ADDR),
                data: None,
            }),
        };
        reply(deps.as_mut(), mock_env(), response).unwrap();

        // ensure this is now registered
        let raw = query(deps.as_ref(), mock_env(), QueryMsg::ListAccounts {}).unwrap();
        let res: ListAccountsResponse = from_slice(&raw).unwrap();
        assert_eq!(1, res.accounts.len());
        assert_eq!(
            &res.accounts[0],
            &AccountInfo {
                account: REFLECT_ADDR.into(),
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
        let res: AccountResponse = from_slice(&raw).unwrap();
        assert_eq!(res.account.unwrap(), REFLECT_ADDR);
    }

    #[test]
    fn handle_dispatch_packet() {
        let mut deps = setup();

        let channel_id = "channel-123";
        let account = "acct-123";

        // receive a packet for an unregistered channel returns app-level error (not Result::Err)
        let msgs_to_dispatch = vec![BankMsg::Send {
            to_address: "my-friend".into(),
            amount: coins(123456789, "uatom"),
        }
        .into()];
        let ibc_msg = PacketMsg::Dispatch {
            msgs: msgs_to_dispatch.clone(),
        };
        let packet = mock_ibc_packet_recv(channel_id, &ibc_msg).unwrap();
        let res = ibc_packet_receive(deps.as_mut(), mock_env(), packet).unwrap();
        // we didn't dispatch anything
        assert_eq!(0, res.messages.len());
        // acknowledgement is an error
        let ack: AcknowledgementMsg<DispatchResponse> = from_slice(&res.acknowledgement).unwrap();
        assert_eq!(
            ack.unwrap_err(),
            "invalid packet: cosmwasm_std::addresses::Addr not found"
        );

        // register the channel
        connect(deps.as_mut(), channel_id, account);

        // receive a packet for an unregistered channel returns app-level error (not Result::Err)
        let packet = mock_ibc_packet_recv(channel_id, &ibc_msg).unwrap();
        let res = ibc_packet_receive(deps.as_mut(), mock_env(), packet).unwrap();

        // assert app-level success
        let ack: AcknowledgementMsg<()> = from_slice(&res.acknowledgement).unwrap();
        ack.unwrap();

        // and we dispatch the BankMsg via submessage
        assert_eq!(0, res.messages.len());
        assert_eq!(1, res.submessages.len());
        assert_eq!(RECEIVE_DISPATCH_ID, res.submessages[0].id);

        // parse the output, ensuring it matches
        if let CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            msg,
            send,
        }) = &res.submessages[0].msg
        {
            assert_eq!(account, contract_addr.as_str());
            assert_eq!(0, send.len());
            // parse the message - should callback with proper channel_id
            let rmsg: ReflectExecuteMsg = from_slice(&msg).unwrap();
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
        let packet = mock_ibc_packet_recv(channel_id, &bad_data).unwrap();
        let res = ibc_packet_receive(deps.as_mut(), mock_env(), packet).unwrap();
        // we didn't dispatch anything
        assert_eq!(0, res.submessages.len());
        // acknowledgement is an error
        let ack: AcknowledgementMsg<DispatchResponse> = from_slice(&res.acknowledgement).unwrap();
        assert_eq!(ack.unwrap_err(), "invalid packet: Error parsing into type ibc_reflect::msg::PacketMsg: unknown variant `reflect_code_id`, expected one of `dispatch`, `who_am_i`, `balances`");
    }

    #[test]
    fn check_close_channel() {
        let mut deps = setup();

        let channel_id = "channel-123";
        let account = "acct-123";

        // register the channel
        connect(deps.as_mut(), channel_id, account);
        // assign it some funds
        let funds = vec![coin(123456, "uatom"), coin(7654321, "tgrd")];
        deps.querier.update_balance(account, funds.clone());

        // channel should be listed and have balance
        let raw = query(deps.as_ref(), mock_env(), QueryMsg::ListAccounts {}).unwrap();
        let res: ListAccountsResponse = from_slice(&raw).unwrap();
        assert_eq!(1, res.accounts.len());
        let balance = deps.as_ref().querier.query_all_balances(account).unwrap();
        assert_eq!(funds, balance);

        // close the channel
        let channel = mock_ibc_channel(channel_id, IbcOrder::Ordered, IBC_VERSION);
        let res = ibc_channel_close(deps.as_mut(), mock_env(), channel).unwrap();

        // it pulls out all money from the reflect contract
        assert_eq!(1, res.messages.len());
        if let CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr, msg, ..
        }) = &res.messages[0]
        {
            assert_eq!(contract_addr.as_str(), account);
            let reflect: ReflectExecuteMsg = from_slice(msg).unwrap();
            match reflect {
                ReflectExecuteMsg::ReflectMsg { msgs } => {
                    assert_eq!(1, msgs.len());
                    assert_eq!(
                        &msgs[0],
                        &BankMsg::Send {
                            to_address: MOCK_CONTRACT_ADDR.into(),
                            amount: funds
                        }
                        .into()
                    )
                }
            }
        } else {
            panic!("Unexpected message: {:?}", &res.messages[0]);
        }

        // and removes the account lookup
        let raw = query(deps.as_ref(), mock_env(), QueryMsg::ListAccounts {}).unwrap();
        let res: ListAccountsResponse = from_slice(&raw).unwrap();
        assert_eq!(0, res.accounts.len());
    }
}
