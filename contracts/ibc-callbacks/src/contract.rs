use cosmwasm_std::{
    entry_point, to_json_binary, to_json_string, Addr, Binary, Deps, DepsMut, Empty, Env,
    IbcBasicResponse, IbcMsg, IbcPacketAckMsg, IbcPacketTimeoutMsg, IbcTimeout, MessageInfo,
    Response, StdError, StdResult,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
            timeout: IbcTimeout::with_timestamp(env.block.time.plus_seconds(5)), // TODO: how much time?
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
// #[entry_point]
pub fn ibc_source_chain_callback(
    deps: DepsMut,
    _env: Env,
    msg: IbcSourceChainCallback,
) -> StdResult<IbcBasicResponse> {
    let mut counts = load_stats(deps.storage)?;

    match msg {
        IbcSourceChainCallback::Acknowledgement(ack) => {
            // increment the counter
            counts.ibc_ack_callbacks.push(ack);
        }
        IbcSourceChainCallback::Timeout(timeout) => {
            // increment the counter
            counts.ibc_timeout_callbacks.push(timeout);
        }
    }

    save_stats(deps.storage, &counts)?;

    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_source_chain_callback"))
}

// TODO: move all the types below to cosmwasm-std when the everything is ready

/// This is just a type representing the data that has to be sent with the IBC message to receive
/// callbacks. It should be serialized to JSON and sent with the IBC message.
/// The specific field to send it in can vary depending on the IBC message,
/// but is usually the `memo` field by convention.
///
/// See [`IbcSourceChainCallback`] for more details.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct IbcCallbackData {
    // using private fields to force use of the constructors
    src_callback: Option<IbcSrcCallback>,
    dst_callback: Option<IbcDstCallback>,
}

impl IbcCallbackData {
    /// Use this if you want to execute callbacks on both the source and destination chain.
    ///
    /// In the first prototype, we only support receiving callbacks on the source chain.
    pub fn both(src_callback: IbcSrcCallback, dst_callback: IbcDstCallback) -> Self {
        IbcCallbackData {
            src_callback: Some(src_callback),
            dst_callback: Some(dst_callback),
        }
    }

    /// Use this if you want to execute callbacks on the source chain, but not the destination chain.
    pub fn source(src_callback: IbcSrcCallback) -> Self {
        IbcCallbackData {
            src_callback: Some(src_callback),
            dst_callback: None,
        }
    }

    /// Use this if you want to execute callbacks on the destination chain, but not the source chain.
    ///
    /// In the first prototype, we only support receiving callbacks on the source chain.
    pub fn destination(dst_callback: IbcDstCallback) -> Self {
        IbcCallbackData {
            src_callback: None,
            dst_callback: Some(dst_callback),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct IbcSrcCallback {
    /// The source chain address that should receive the callback.
    /// You probably want to put `env.contract.address` here.
    pub address: Addr,
    /// Optional gas limit for the callback (in Cosmos SDK gas units)
    pub gas_limit: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct IbcDstCallback {
    /// The destination chain address that should receive the callback.
    pub address: String,
    /// Optional gas limit for the callback (in Cosmos SDK gas units)
    pub gas_limit: Option<u64>,
}

/// The type of IBC callback that is being called.
///
/// IBC callbacks are needed for cases where your contract triggers the sending of an IBC packet
/// through some other message (i.e. not through [`IbcMsg::SendPacket`]) and needs to know whether
/// or not the packet was successfully received on the other chain.
/// A prominent example is the [`IbcMsg::Transfer`] message. Without callbacks, you cannot know
/// whether the transfer was successful or not.
///
/// Note that there are some prerequisites that need to be fulfilled to receive source chain callbacks:
/// - The contract must implement the `ibc_source_chain_callback` entrypoint.
/// - The module that sends the packet must be wrapped by an `IBCMiddleware`
///   (i.e. the source chain needs to support callbacks for the message you are sending).
/// - You have to add json-encoded [`IbcCallbackData`] to a specific field of the message.
///   For `IbcMsg::Transfer`, this is the `memo` field.
/// - The receiver of the callback must also be the sender of the message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum IbcSourceChainCallback {
    Acknowledgement(IbcPacketAckMsg),
    Timeout(IbcPacketTimeoutMsg),
    // TODO: should this be non-exhaustive?
}
