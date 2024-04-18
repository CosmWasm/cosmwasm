//! This module contains types for the IBC callbacks defined in
//! [ADR-8](https://github.com/cosmos/ibc-go/blob/main/docs/architecture/adr-008-app-caller-cbs.md).

use cosmwasm_core::Binary;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Addr, IbcPacket, IbcPacketAckMsg, IbcPacketTimeoutMsg, Uint64};

/// This is just a type representing the data that has to be sent with the IBC message to receive
/// callbacks. It should be serialized and sent with the IBC message.
/// The specific field and format to send it in can vary depending on the IBC message,
/// but is usually the `memo` field by convention.
///
/// See [`IbcSourceChainCallback`] for more details.
///
/// # Example
///
/// ```rust
/// use cosmwasm_std::{
///     to_json_string, Coin, IbcCallbackData, IbcMsg, IbcSrcCallback, IbcTimeout, Response,
///     Timestamp,
/// };
///
/// # use cosmwasm_std::testing::mock_env;
/// # let env = mock_env();
///
/// let _transfer = IbcMsg::Transfer {
///     to_address: "cosmos1example".to_string(),
///     channel_id: "channel-0".to_string(),
///     amount: Coin::new(10u32, "ucoin"),
///     timeout: Timestamp::from_seconds(12345).into(),
///     memo: Some(to_json_string(&IbcCallbackData::source(IbcSrcCallback {
///         address: env.contract.address,
///         gas_limit: None,
///     })).unwrap()),
/// };
/// ```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct IbcCallbackData {
    // using private fields to force use of the constructors
    #[serde(skip_serializing_if = "Option::is_none")]
    src_callback: Option<IbcSrcCallback>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dest_callback: Option<IbcDstCallback>,
}

impl IbcCallbackData {
    /// Use this if you want to execute callbacks on both the source and destination chain.
    pub fn both(src_callback: IbcSrcCallback, dest_callback: IbcDstCallback) -> Self {
        IbcCallbackData {
            src_callback: Some(src_callback),
            dest_callback: Some(dest_callback),
        }
    }

    /// Use this if you want to execute callbacks on the source chain, but not the destination chain.
    pub fn source(src_callback: IbcSrcCallback) -> Self {
        IbcCallbackData {
            src_callback: Some(src_callback),
            dest_callback: None,
        }
    }

    /// Use this if you want to execute callbacks on the destination chain, but not the source chain.
    pub fn destination(dest_callback: IbcDstCallback) -> Self {
        IbcCallbackData {
            src_callback: None,
            dest_callback: Some(dest_callback),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct IbcSrcCallback {
    /// The source chain address that should receive the callback.
    /// For CosmWasm contracts, this *must* be `env.contract.address`.
    /// Other addresses are not allowed and will effectively be ignored.
    pub address: Addr,
    /// Optional gas limit for the callback (in Cosmos SDK gas units)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_limit: Option<Uint64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct IbcDstCallback {
    /// The destination chain address that should receive the callback.
    pub address: String,
    /// Optional gas limit for the callback (in Cosmos SDK gas units)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_limit: Option<Uint64>,
}

/// The type of IBC source chain callback that is being called.
///
/// IBC source chain callbacks are needed for cases where your contract triggers the sending of an
/// IBC packet through some other message (i.e. not through [`IbcMsg::SendPacket`]) and needs to
/// know whether or not the packet was successfully received on the other chain.
/// A prominent example is the [`IbcMsg::Transfer`] message. Without callbacks, you cannot know
/// whether the transfer was successful or not.
///
/// Note that there are some prerequisites that need to be fulfilled to receive source chain callbacks:
/// - The contract must implement the `ibc_source_chain_callback` entrypoint.
/// - The IBC application in the source chain must have support for the callbacks middleware.
/// - You have to add serialized [`IbcCallbackData`] to a specific field of the message.
///   For `IbcMsg::Transfer`, this is the `memo` field and it needs to be json-encoded.
/// - The receiver of the callback must also be the sender of the message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IbcSourceChainCallbackMsg {
    Acknowledgement(IbcPacketAckMsg),
    Timeout(IbcPacketTimeoutMsg),
}

/// The message type of the IBC destination chain callback.
///
/// The IBC destination chain callback is needed for cases where someone triggers the sending of an
/// IBC packet through some other message (i.e. not through [`IbcMsg::SendPacket`]) and
/// your contract needs to know that it received this.
/// A prominent example is the [`IbcMsg::Transfer`] message. Without callbacks, you cannot know
/// that someone sent you IBC coins.
///
/// The callback is called after the packet was acknowledged on the destination chain, as follows:
/// - If the acknowledgement is synchronous (i.e. returned immediately when the packet is received),
///   the callback is called only if the acknowledgement was successful.
/// - If the acknowledgement is asynchronous (i.e. written later using `WriteAcknowledgement`),
///   the callback is called regardless of the success of the acknowledgement.
///
/// Note that there are some prerequisites that need to be fulfilled to receive source chain callbacks:
/// - The contract must implement the `ibc_destination_chain_callback` entrypoint.
/// - The IBC application in the destination chain must have support for the callbacks middleware.
/// - You have to add serialized [`IbcCallbackData`] to a specific field of the message.
///   For `IbcMsg::Transfer`, this is the `memo` field and it needs to be json-encoded.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct IbcDestinationChainCallbackMsg {
    pub packet: IbcPacket,
    pub ack: IbcFullAcknowledgement,
}

/// The acknowledgement written by the module on the destination chain.
/// It is different from the [`crate::IbcAcknowledgement`] as it can be unsuccessful.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct IbcFullAcknowledgement {
    /// The acknowledgement data returned by the module.
    pub data: Binary,
    /// Whether the acknowledgement was successful or not.
    pub success: bool,
}

#[cfg(test)]
mod tests {
    use crate::to_json_string;

    use super::*;

    #[test]
    fn ibc_callback_data_serialization() {
        let mut data = IbcCallbackData::both(
            IbcSrcCallback {
                address: Addr::unchecked("src_address"),
                gas_limit: Some(123u64.into()),
            },
            IbcDstCallback {
                address: "dst_address".to_string(),
                gas_limit: Some(1234u64.into()),
            },
        );

        // both
        let json = to_json_string(&data).unwrap();
        assert_eq!(
            json,
            r#"{"src_callback":{"address":"src_address","gas_limit":"123"},"dest_callback":{"address":"dst_address","gas_limit":"1234"}}"#
        );

        // dst only, without gas limit
        let mut src = data.src_callback.take().unwrap();
        data.dest_callback.as_mut().unwrap().gas_limit = None;
        let json = to_json_string(&data).unwrap();
        assert_eq!(json, r#"{"dest_callback":{"address":"dst_address"}}"#);

        // source only, without gas limit
        src.gas_limit = None;
        data.src_callback = Some(src);
        data.dest_callback = None;
        let json = to_json_string(&data).unwrap();
        assert_eq!(json, r#"{"src_callback":{"address":"src_address"}}"#);
    }
}
