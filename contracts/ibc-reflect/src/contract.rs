use cosmwasm_std::{
    entry_point, to_binary, DepsMut, Env, HandleResponse, HumanAddr, IbcAcknowledgement,
    IbcBasicResponse, IbcChannel, IbcOrder, IbcPacket, IbcReceiveResponse, InitResponse,
    MessageInfo, StdError, StdResult, WasmMsg,
};

use crate::msg::{HandleMsg, InitMsg, ReflectInitMsg};
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
pub fn ibc_channel_open(_deps: DepsMut, _env: Env, msg: IbcChannel) -> StdResult<()> {
    if msg.order != IbcOrder::Ordered {
        return Err(StdError::generic_err("Only supports ordered channels"));
    }
    if msg.version.as_str() != IBC_VERSION {
        return Err(StdError::generic_err(format!(
            "Must set version to `{}`",
            IBC_VERSION
        )));
    }

    Ok(())
}

#[entry_point]
/// once it's established, we create the reflect contract
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannel,
) -> StdResult<IbcBasicResponse> {
    let cfg = config(deps.storage).load()?;
    let chan_id = msg.endpoint.channel_id;
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
    _msg: IbcChannel,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::default())
}

#[entry_point]
/// we look for a the proper reflect contract to relay to and send the message
/// We cannot return any meaningful response value as we do not know the response value
/// of execution. We just return ok if we dispatched, error if we failed to dispatch
pub fn ibc_packet_receive(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacket,
) -> StdResult<IbcReceiveResponse> {
    // TODO
    Ok(IbcReceiveResponse::default())
}

#[entry_point]
/// we do nothing
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcAcknowledgement,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::default())
}

#[entry_point]
/// we do nothing
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacket,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, HumanAddr, StdError, Storage};

    #[test]
    fn init_fails() {
        let mut deps = mock_dependencies(&[]);

        let msg = InitMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));
        // we can just call .unwrap() to assert this was a success
        let res = init(deps.as_mut(), mock_env(), info, msg);
        match res.unwrap_err() {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "You can only use this contract for migrations")
            }
            _ => panic!("expected migrate error message"),
        }
    }
}
