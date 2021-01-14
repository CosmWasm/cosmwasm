use cosmwasm_std::{
    entry_point, from_binary, to_binary, CosmosMsg, DepsMut, Env, HandleResponse, HumanAddr,
    IbcAcknowledgement, IbcBasicResponse, IbcChannel, IbcOrder, IbcPacket, IbcReceiveResponse,
    InitResponse, MessageInfo, StdError, StdResult, WasmMsg,
};

use crate::msg::{
    AcknowledgementMsg, HandleMsg, InitMsg, PacketMsg, ReflectHandleMsg, ReflectInitMsg,
};
use crate::state::{accounts, config, Config};

const IBC_VERSION: &str = "ibc-reflect";

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

    let payload = to_binary(&ReflectInitMsg {
        callback_id: Some(chan_id),
    })?;
    let msg = WasmMsg::Instantiate {
        code_id: cfg.reflect_code_id,
        msg: payload,
        send: vec![],
        label: Some(label),
    };

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
    let wasm_msg = WasmMsg::Execute {
        contract_addr: reflect_addr,
        msg: to_binary(&reflect_msg)?,
        send: vec![],
    };
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
    use cosmwasm_std::{IbcEndpoint, OwnedDeps};

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

        // we get the callback from reflect
        let handle_msg = HandleMsg::InitCallback {
            id: our_channel.clone(),
            contract_addr: REFLECT_ADDR.into(),
        };
        let info = mock_info(REFLECT_ADDR, &[]);
        let res = handle(deps.as_mut(), mock_env(), info, handle_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // ensure this is now registered
    }
}
