use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Empty, Env, IbcBasicResponse,
    IbcDestinationCallbackMsg, IbcDstCallback, IbcSourceCallbackMsg, IbcSrcCallback, IbcTimeout,
    MessageInfo, Response, StdError, StdResult, TransferMsgBuilder,
};

use crate::msg::{CallbackType, ExecuteMsg, QueryMsg};
use crate::state::{load_stats, save_stats, CallbackStats};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> StdResult<Response> {
    // initialize counts
    let counts = CallbackStats::default();
    save_stats(deps.storage, &counts)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

/// Sends an `IbcMsg::Transfer` to the given `to_address` on the given `channel_id`.
#[entry_point]
pub fn execute(
    _deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Transfer {
            to_address,
            channel_id,
            timeout_seconds,
            callback_type,
        } => execute_transfer(
            env,
            info,
            to_address,
            channel_id,
            timeout_seconds,
            callback_type,
        ),
    }
}

fn execute_transfer(
    env: Env,
    info: MessageInfo,
    to_address: String,
    channel_id: String,
    timeout_seconds: u32,
    callback_type: CallbackType,
) -> StdResult<Response> {
    let src_callback = IbcSrcCallback {
        address: env.contract.address,
        gas_limit: None,
    };
    let dst_callback = IbcDstCallback {
        address: to_address.clone(),
        gas_limit: None,
    };
    let coin = match &*info.funds {
        [coin] if !coin.amount.is_zero() => coin,
        _ => {
            return Err(StdError::generic_err(
                "Must send exactly one denom to trigger ics-20 transfer",
            ))
        }
    };

    let builder = TransferMsgBuilder::new(
        channel_id,
        to_address.clone(),
        coin.clone(),
        IbcTimeout::with_timestamp(env.block.time.plus_seconds(timeout_seconds as u64)),
    );
    let transfer_msg = match callback_type {
        CallbackType::Both => builder
            .with_src_callback(src_callback)
            .with_dst_callback(dst_callback)
            .build(),
        CallbackType::Src => builder.with_src_callback(src_callback).build(),
        CallbackType::Dst => builder.with_dst_callback(dst_callback).build(),
    };

    Ok(Response::new()
        .add_message(transfer_msg)
        .add_attribute("action", "execute"))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::CallbackStats {} => to_json_binary(&load_stats(deps.storage)?),
    }
}

/// This is the entrypoint that is called by the source chain when a callbacks-enabled IBC message
/// is acknowledged or times out.
#[entry_point]
pub fn ibc_source_callback(
    deps: DepsMut,
    _env: Env,
    msg: IbcSourceCallbackMsg,
) -> StdResult<IbcBasicResponse> {
    let mut counts = load_stats(deps.storage)?;

    match msg {
        IbcSourceCallbackMsg::Acknowledgement(ack) => {
            // save the ack
            counts.ibc_ack_callbacks.push(ack);
        }
        IbcSourceCallbackMsg::Timeout(timeout) => {
            // save the timeout
            counts.ibc_timeout_callbacks.push(timeout);
        }
    }

    save_stats(deps.storage, &counts)?;

    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_source_callback"))
}

#[entry_point]
pub fn ibc_destination_callback(
    deps: DepsMut,
    _env: Env,
    msg: IbcDestinationCallbackMsg,
) -> StdResult<IbcBasicResponse> {
    let mut counts = load_stats(deps.storage)?;

    // save the receive
    counts.ibc_destination_callbacks.push(msg);

    save_stats(deps.storage, &counts)?;

    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_destination_callback"))
}
