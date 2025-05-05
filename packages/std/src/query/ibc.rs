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
    //
    // ListChannels was removed in CosmWasm 3 due to potentially unbound number of results.
    // See https://github.com/CosmWasm/cosmwasm/issues/2223
    //
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
pub struct ChannelResponse {
    pub channel: Option<IbcChannel>,
}

impl_response_constructor!(ChannelResponse, channel: Option<IbcChannel>);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[non_exhaustive]
pub struct FeeEnabledChannelResponse {
    pub fee_enabled: bool,
}

impl_response_constructor!(FeeEnabledChannelResponse, fee_enabled: bool);
