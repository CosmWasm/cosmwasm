use cosmwasm_std::{
    entry_point, from_binary, to_binary, wasm_execute, wasm_instantiate, CosmosMsg, Deps, DepsMut,
    Env, HandleResponse, HumanAddr, IbcAcknowledgement, IbcBasicResponse, IbcChannel, IbcOrder,
    IbcPacket, IbcReceiveResponse, InitResponse, MessageInfo, Order, QueryResponse, StdError,
    StdResult,
};

use crate::msg::{
    AccountInfo, AccountResponse, AcknowledgementMsg, HandleMsg, InitMsg, ListAccountsResponse,
    PacketMsg, QueryMsg, ReflectHandleMsg, ReflectInitMsg,
};
use crate::state::{accounts, accounts_read, config, Config};

pub const IBC_VERSION: &str = "ibc-reflect";

#[entry_point]
pub fn init(deps: DepsMut, _env: Env, _info: MessageInfo, msg: InitMsg) -> StdResult<InitResponse> {
    // we store the reflect_id for creating accounts later
    let cfg = Config {
        reflect_code_id: msg.reflect_code_id,
    };
    config(deps.storage).save(&cfg)?;

    Ok(InitResponse::default())
}

#[entry_point]
pub fn handle(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::InitCallback { id, contract_addr } => {
            handle_init_callback(deps, info, id, contract_addr)
        }
    }
}

pub fn handle_init_callback(
    deps: DepsMut,
    info: MessageInfo,
    id: String,
    contract_addr: HumanAddr,
) -> StdResult<HandleResponse> {
    // sanity check - the caller is registering itself
    if info.sender != contract_addr {
        return Err(StdError::generic_err("Must register self on callback"));
    }

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

    Ok(HandleResponse::default())
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
        account: Some(account),
    })
}

pub fn query_list_accounts(deps: Deps) -> StdResult<ListAccountsResponse> {
    let accounts: StdResult<Vec<_>> = accounts_read(deps.storage)
        .range(None, None, Order::Ascending)
        .map(|r| {
            let (k, account) = r?;
            Ok(AccountInfo {
                account,
                channel_id: String::from_utf8(k)?,
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
    let payload = ReflectInitMsg {
        callback_id: Some(chan_id),
    };
    let msg = wasm_instantiate(cfg.reflect_code_id, &payload, vec![], Some(label))?;
    Ok(IbcBasicResponse {
        messages: vec![msg.into()],
        attributes: vec![],
    })
}

#[entry_point]
/// we do nothing
/// TODO: remove the account from the lookup?
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    _channel: IbcChannel,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::default())
}

// pull this into one function so we can handle all StdErrors once
fn parse_receipt(deps: DepsMut, packet: IbcPacket) -> StdResult<(HumanAddr, Vec<CosmosMsg>)> {
    // which local channel did this packet come on
    let caller = packet.dest.channel_id;
    // what is the reflect contract here
    let reflect_addr = accounts(deps.storage).load(caller.as_bytes())?;

    // parse the message we got
    let msg: PacketMsg = from_binary(&packet.data)?;
    Ok((reflect_addr, msg.msgs))
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
    let (reflect_addr, msgs) = match parse_receipt(deps, packet) {
        Ok(m) => m,
        Err(_) => {
            let acknowledgement =
                to_binary(&AcknowledgementMsg::Err("invalid packet".to_string()))?;
            return Ok(IbcReceiveResponse {
                acknowledgement,
                messages: vec![],
                attributes: vec![],
            });
        }
    };

    // let them know we're fine
    let acknowledgement = to_binary(&AcknowledgementMsg::Ok(()))?;
    // create the message to re-dispatch to the reflect contract
    let reflect_msg = ReflectHandleMsg::ReflectMsg { msgs };
    let wasm_msg = wasm_execute(reflect_addr, &reflect_msg, vec![])?;
    // and we are golden
    Ok(IbcReceiveResponse {
        acknowledgement,
        messages: vec![wasm_msg.into()],
        attributes: vec![],
    })
}

#[entry_point]
/// we do nothing
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    _ack: IbcAcknowledgement,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::default())
}

#[entry_point]
/// we do nothing
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _packet: IbcPacket,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{
        coins, from_slice, BankMsg, IbcEndpoint, IbcTimeoutHeight, OwnedDeps, WasmMsg,
    };
    use serde::Serialize;

    const CREATOR: &str = "creator";
    // code id of the reflect contract
    const REFLECT_ID: u64 = 101;
    // address of first reflect contract instance that we created
    const REFLECT_ADDR: &str = "reflect-acct-1";

    fn setup() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies(&[]);
        let msg = InitMsg {
            reflect_code_id: REFLECT_ID,
        };
        let info = mock_info(CREATOR, &[]);
        let res = init(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        deps
    }

    // this provides a minimal channel skeleton, which can be modified to set certain fields
    // TODO: something similar should be a helper in ibc.rs
    fn mock_channel(order: IbcOrder, version: &str) -> IbcChannel {
        IbcChannel {
            endpoint: IbcEndpoint {
                port_id: "my_port".to_string(),
                channel_id: "channel-5".to_string(),
            },
            counterparty_endpoint: IbcEndpoint {
                port_id: "their_port".to_string(),
                channel_id: "channel-7".to_string(),
            },
            order,
            version: version.to_string(),
            /// CounterpartyVersion can be None when not known this context, yet
            counterparty_version: None,
            /// The connection upon which this channel was created. If this is a multi-hop
            /// channel, we only expose the first hop.
            connection_id: "connection-2".to_string(),
        }
    }

    // this provides a minimal packet around custom data, which can be modified to set certain fields
    // TODO: something similar should be a helper in ibc.rs
    fn mock_ibc_packet<T: Serialize>(channel_id: &str, data: &T) -> IbcPacket {
        IbcPacket {
            data: to_binary(data).unwrap(),
            src: IbcEndpoint {
                port_id: "their-port".to_string(),
                channel_id: "channel-1234".to_string(),
            },
            dest: IbcEndpoint {
                port_id: "our-port".to_string(),
                channel_id: channel_id.into(),
            },
            sequence: 27,
            timeout_height: IbcTimeoutHeight {
                revision_number: 1,
                timeout_height: 12345678,
            },
            timeout_timestamp: 0,
            version: 1,
        }
    }

    // connect will run through the entire handshake to set up a proper connect and
    // save the account (tested in detail in `proper_handshake_flow`)
    fn connect<T: Into<HumanAddr>>(mut deps: DepsMut, channel_id: &str, account: T) {
        let account = account.into();
        // first we try to open with a valid handshake
        let mut valid_handshake = mock_channel(IbcOrder::Ordered, IBC_VERSION);
        valid_handshake.endpoint.channel_id = channel_id.into();
        ibc_channel_open(deps.branch(), mock_env(), valid_handshake.clone()).unwrap();

        // then we connect (with counter-party version set)
        valid_handshake.counterparty_version = Some(IBC_VERSION.to_string());
        ibc_channel_connect(deps.branch(), mock_env(), valid_handshake).unwrap();

        // which creates a reflect account. here we get the callback
        let handle_msg = HandleMsg::InitCallback {
            id: channel_id.into(),
            contract_addr: account.clone(),
        };
        let info = mock_info(account, &[]);
        handle(deps.branch(), mock_env(), info, handle_msg).unwrap();
    }

    #[test]
    fn init_works() {
        let mut deps = mock_dependencies(&[]);

        let msg = InitMsg {
            reflect_code_id: 17,
        };
        let info = mock_info("creator", &[]);
        let res = init(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len())
    }

    #[test]
    fn enforce_version_in_handshake() {
        let mut deps = setup();

        let wrong_order = mock_channel(IbcOrder::Unordered, IBC_VERSION);
        ibc_channel_open(deps.as_mut(), mock_env(), wrong_order).unwrap_err();

        let wrong_version = mock_channel(IbcOrder::Ordered, "reflect");
        ibc_channel_open(deps.as_mut(), mock_env(), wrong_version).unwrap_err();

        let valid_handshake = mock_channel(IbcOrder::Ordered, IBC_VERSION);
        ibc_channel_open(deps.as_mut(), mock_env(), valid_handshake).unwrap();
    }

    #[test]
    fn proper_handshake_flow() {
        let mut deps = setup();

        // first we try to open with a valid handshake
        let mut valid_handshake = mock_channel(IbcOrder::Ordered, IBC_VERSION);
        ibc_channel_open(deps.as_mut(), mock_env(), valid_handshake.clone()).unwrap();

        // then we connect (with counter-party version set)
        valid_handshake.counterparty_version = Some(IBC_VERSION.to_string());
        let res = ibc_channel_connect(deps.as_mut(), mock_env(), valid_handshake.clone()).unwrap();
        // and set up a reflect account
        assert_eq!(1, res.messages.len());
        let our_channel = valid_handshake.endpoint.channel_id.clone();
        if let CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id,
            msg,
            send,
            label,
        }) = &res.messages[0]
        {
            assert_eq!(&REFLECT_ID, code_id);
            assert_eq!(0, send.len());
            assert!(label.as_ref().unwrap().contains(&our_channel));
            // parse the message - should callback with proper channel_id
            let rmsg: ReflectInitMsg = from_binary(&msg).unwrap();
            assert_eq!(rmsg.callback_id, Some(our_channel.clone()));
        } else {
            panic!("invalid return message: {:?}", res.messages[0]);
        }

        // no accounts set yet
        let raw = query(deps.as_ref(), mock_env(), QueryMsg::ListAccounts {}).unwrap();
        let res: ListAccountsResponse = from_slice(&raw).unwrap();
        assert_eq!(0, res.accounts.len());

        // we get the callback from reflect
        let handle_msg = HandleMsg::InitCallback {
            id: our_channel.clone(),
            contract_addr: REFLECT_ADDR.into(),
        };
        let info = mock_info(REFLECT_ADDR, &[]);
        let res = handle(deps.as_mut(), mock_env(), info, handle_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // ensure this is now registered
        let raw = query(deps.as_ref(), mock_env(), QueryMsg::ListAccounts {}).unwrap();
        let res: ListAccountsResponse = from_slice(&raw).unwrap();
        assert_eq!(1, res.accounts.len());
        assert_eq!(
            &res.accounts[0],
            &AccountInfo {
                account: REFLECT_ADDR.into(),
                channel_id: our_channel.clone(),
            }
        );

        // and the account query also works
        let raw = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Account {
                channel_id: our_channel.clone(),
            },
        )
        .unwrap();
        let res: AccountResponse = from_slice(&raw).unwrap();
        assert_eq!(res.account.unwrap(), HumanAddr::from(REFLECT_ADDR));
    }

    #[test]
    fn handle_packet() {
        let mut deps = setup();

        let channel_id: &str = "channel-123";
        let account: &str = "acct-123";

        // receive a packet for an unregistered channel returns app-level error (not Result::Err)
        let ibc_msg = PacketMsg {
            msgs: vec![BankMsg::Send {
                to_address: "my-friend".into(),
                amount: coins(123456789, "uatom"),
            }
            .into()],
        };
        let packet = mock_ibc_packet(channel_id, &ibc_msg);
        let res = ibc_packet_receive(deps.as_mut(), mock_env(), packet.clone()).unwrap();
        // we didn't dispatch anything
        assert_eq!(0, res.messages.len());
        // acknowledgement is an error
        let ack: AcknowledgementMsg = from_slice(&res.acknowledgement).unwrap();
        assert_eq!(ack.unwrap_err(), "invalid packet");

        // register the channel
        connect(deps.as_mut(), channel_id, account);

        // receive a packet for an unregistered channel returns app-level error (not Result::Err)
        let packet = mock_ibc_packet(channel_id, &ibc_msg);
        let res = ibc_packet_receive(deps.as_mut(), mock_env(), packet.clone()).unwrap();

        // TODO: blocked on serde-json-wasm fix
        // see: https://github.com/CosmWasm/serde-json-wasm/issues/27
        // assert app-level success
        // let ack: AcknowledgementMsg = from_slice(&res.acknowledgement).unwrap();
        // ack.unwrap();

        // and we dispatch the BankMsg
        assert_eq!(1, res.messages.len());
        // parse the output, ensuring it matches
        if let CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr,
            msg,
            send,
        }) = &res.messages[0]
        {
            assert_eq!(account, contract_addr.as_str());
            assert_eq!(0, send.len());
            // parse the message - should callback with proper channel_id
            let rmsg: ReflectHandleMsg = from_slice(&msg).unwrap();
            assert_eq!(
                rmsg,
                ReflectHandleMsg::ReflectMsg {
                    msgs: ibc_msg.msgs.clone()
                }
            );
        } else {
            panic!("invalid return message: {:?}", res.messages[0]);
        }

        // invalid packet format on registered channel also returns app-level error
        let bad_data = InitMsg {
            reflect_code_id: 12345,
        };
        let packet = mock_ibc_packet(channel_id, &bad_data);
        let res = ibc_packet_receive(deps.as_mut(), mock_env(), packet.clone()).unwrap();
        // we didn't dispatch anything
        assert_eq!(0, res.messages.len());
        // acknowledgement is an error
        let ack: AcknowledgementMsg = from_slice(&res.acknowledgement).unwrap();
        assert_eq!(ack.unwrap_err(), "invalid packet");
    }
}
