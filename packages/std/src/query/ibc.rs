use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ibc::IbcChannel;
use crate::prelude::*;

/// These are queries to the various IBC modules to see the state of the contract's
/// IBC connection.
/// Most of these will return errors if the contract is not "ibc enabled".
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IbcQuery {
    /// Gets the Port ID the current contract is bound to.
    ///
    /// Returns a `PortIdResponse`.
    PortId {},
    /// Lists all channels that are bound to a given port.
    /// If `port_id` is omitted, this list all channels bound to the contract's port.
    ///
    /// Returns a `ListChannelsResponse`.
    #[deprecated = "Returns a potentially unbound number of results. If you think you have a valid usecase, please open an issue."]
    ListChannels { port_id: Option<String> },
    /// Lists all information for a (portID, channelID) pair.
    /// If port_id is omitted, it will default to the contract's own channel.
    /// (To save a PortId{} call)
    ///
    /// Returns a `ChannelResponse`.
    Channel {
        channel_id: String,
        port_id: Option<String>,
    },
    /// Queries whether the given channel supports IBC fees.
    /// If port_id is omitted, it will default to the contract's own channel.
    /// (To save a PortId{} call)
    ///
    /// Returns a `FeeEnabledChannelResponse`.
    #[cfg(feature = "cosmwasm_2_2")]
    #[deprecated(
        since = "2.2.3",
        note = "IBC fees have been removed from ibc-go `v10`, which is used in wasmd `v0.55.0`."
    )]
    FeeEnabledChannel {
        port_id: Option<String>,
        channel_id: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[non_exhaustive]
pub struct PortIdResponse {
    pub port_id: String,
}

impl_response_constructor!(PortIdResponse, port_id: String);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[non_exhaustive]
pub struct ListChannelsResponse {
    pub channels: Vec<IbcChannel>,
}

impl_response_constructor!(ListChannelsResponse, channels: Vec<IbcChannel>);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[non_exhaustive]
pub struct ChannelResponse {
    pub channel: Option<IbcChannel>,
}

impl_response_constructor!(ChannelResponse, channel: Option<IbcChannel>);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[non_exhaustive]
#[deprecated(
    since = "2.2.3",
    note = "IBC fees have been removed from ibc-go `v10`, which is used in wasmd `v0.55.0`."
)]
pub struct FeeEnabledChannelResponse {
    pub fee_enabled: bool,
}

impl_response_constructor!(FeeEnabledChannelResponse, fee_enabled: bool);
