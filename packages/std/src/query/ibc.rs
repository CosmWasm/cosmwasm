#![cfg(feature = "stargate")]

use cosmwasm_schema::cw_serde;

use crate::ibc::IbcChannel;

/// These are queries to the various IBC modules to see the state of the contract's
/// IBC connection. These will return errors if the contract is not "ibc enabled"
#[non_exhaustive]
#[cw_serde]
#[derive(Eq)]
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

#[cw_serde]
#[derive(Eq)]
pub struct PortIdResponse {
    pub port_id: String,
}

#[cw_serde]
#[derive(Eq)]
pub struct ListChannelsResponse {
    pub channels: Vec<IbcChannel>,
}

#[cw_serde]
#[derive(Eq)]
pub struct ChannelResponse {
    pub channel: Option<IbcChannel>,
}
