use cosmwasm_std::{
    entry_point, to_json_binary, to_json_string, Binary, Deps, DepsMut, Empty, Env,
    IbcBasicResponse, IbcCallbackData, IbcMsg, IbcSourceChainCallbackMsg, IbcSrcCallback,
    IbcTimeout, MessageInfo, Response, StdError, StdResult,
};

use crate::msg::{ExecuteMsg, QueryMsg};
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
    let transfer_msg = match &*info.funds {
        [coin] if !coin.amount.is_zero() => IbcMsg::Transfer {
            to_address: msg.to_address,
            timeout: IbcTimeout::with_timestamp(
                env.block.time.plus_seconds(msg.timeout_seconds.u64()),
            ),
            channel_id: msg.channel_id.clone(),
            amount: coin.clone(),
            memo: Some(to_json_string(&IbcCallbackData::source(IbcSrcCallback {
                address: env.contract.address.clone(),
                gas_limit: None,
            }))?),
        },
        _ => {
            return Err(StdError::generic_err(
                "Must send exactly one denom to trigger ics-20 transfer",
            ))
        }
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
pub fn ibc_source_chain_callback(
    deps: DepsMut,
    _env: Env,
    msg: IbcSourceChainCallbackMsg,
) -> StdResult<IbcBasicResponse> {
    let mut counts = load_stats(deps.storage)?;

    match msg {
        IbcSourceChainCallbackMsg::Acknowledgement(ack) => {
            // increment the counter
            counts.ibc_ack_callbacks.push(ack);
        }
        IbcSourceChainCallbackMsg::Timeout(timeout) => {
            // increment the counter
            counts.ibc_timeout_callbacks.push(timeout);
        }
    }

    save_stats(deps.storage, &counts)?;

    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_source_chain_callback"))
}
