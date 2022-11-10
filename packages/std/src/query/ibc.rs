#![cfg(feature = "stargate")]

use serde::{Deserialize, Serialize};

use crate::ibc::IbcChannel;

use alloc::string::String;
use alloc::vec::Vec;

/// These are queries to the various IBC modules to see the state of the contract's
/// IBC connection. These will return errors if the contract is not "ibc enabled"
#[non_exhaustive]
#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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
    // TODO: Add more
}

#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PortIdResponse {
    pub port_id: String,
}

#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ListChannelsResponse {
    pub channels: Vec<IbcChannel>,
}

#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ChannelResponse {
    pub channel: Option<IbcChannel>,
}
