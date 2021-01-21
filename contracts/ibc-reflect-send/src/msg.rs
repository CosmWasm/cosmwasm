#![allow(clippy::field_reassign_with_default)] // see https://github.com/CosmWasm/cosmwasm/issues/685

use cosmwasm_std::{Coin, ContractResult, CosmosMsg, Empty, HumanAddr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::ChannelInfo;

/// InitMsg just needs to know the code_id of a reflect contract to spawn sub-accounts
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    /// Changes the admin
    UpdateAdmin {
        admin: HumanAddr,
    },
    SendMsgs {
        channel_id: String,
        // Note: we don't handle custom messages on remote chains
        msgs: Vec<CosmosMsg<Empty>>,
    },
    CheckRemoteBalance {
        channel_id: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Returns current admin
    Admin {},
    // Shows all open channels (incl. remote info)
    ListChannels {},
    // Get info for one channel
    GetChannel { channel_id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AdminResponse {
    pub admin: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ListChannelsResponse {
    pub channels: Vec<ChannelInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AccountInfo {
    pub channel_id: String,
    /// in normal cases, it should be set, but there is a delay between binding
    /// the channel and making a query and in that time it is empty
    pub remote_addr: Option<HumanAddr>,
    pub remote_balance: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetChannelResponse {
    /// in normal cases, it should be set, but there is a delay between binding
    /// the channel and making a query and in that time it is empty
    pub remote_addr: Option<HumanAddr>,
    pub remote_balance: Vec<Coin>,
}

impl From<ChannelInfo> for GetChannelResponse {
    fn from(input: ChannelInfo) -> Self {
        GetChannelResponse {
            remote_addr: input.remote_addr,
            remote_balance: input.remote_balance,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PacketMsg {
    Dispatch { msgs: Vec<CosmosMsg> },
    WhoAmI {},
    Balances {},
}

/// All acknowledgements are wrapped in `ContractResult`.
/// The success value depends on the PacketMsg variant.
pub type AcknowledgementMsg<T> = ContractResult<T>;

/// This is the success response we send on ack for PacketMsg::Dispatch.
/// Just acknowledge success or error
pub type DispatchResponse = ();

/// This is the success response we send on ack for PacketMsg::WhoAmI.
/// Return the caller's account address on the remote chain
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhoAmIResponse {
    pub account: HumanAddr,
}

/// This is the success response we send on ack for PacketMsg::Balance.
/// Just acknowledge success or error
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BalancesResponse {
    pub account: HumanAddr,
    pub balances: Vec<Coin>,
}
