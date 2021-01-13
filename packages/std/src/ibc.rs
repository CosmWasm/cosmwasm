#![cfg(feature = "stargate")]
// The CosmosMsg variants are defined in results/cosmos_msg.rs
// The rest of the IBC related functionality is defined here

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::binary::Binary;
use crate::results::{Attribute, CosmosMsg};
use crate::types::Empty;

/// These are queries to the various IBC modules to see the state of the contract's
/// IBC connection. These will return errors if the contract is not "ibc enabled"
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IbcQuery {
    /// Gets the Port ID the current contract is bound to.
    /// Returns PortIdResponse
    PortId {},
    /// Lists all (portID, channelID) pairs that are bound to a given port
    /// If port_id is omitted, list all channels bound to the contract's port.
    /// Returns ListChannelsResponse.
    ListChannels { port_id: Option<String> },
    /// Lists all information for a (portID, channelID) pair.
    /// If port_id is omitted, it will default to the contract's own channel.
    /// (To save a PortId{} call)
    /// Returns ChannelResponse.
    Channel {
        channel_id: String,
        port_id: Option<String>,
    },
    // TODO: Add more
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PortIdResponse {
    pub port_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ListChannelsResponse {
    pub channels: Vec<IbcEndpoint>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ChannelResponse {
    pub channel: IbcChannel,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IbcEndpoint {
    pub port_id: String,
    pub channel_id: String,
}

// These are various messages used in the callbacks

/// IbcChannel defines all information on a channel.
/// This is generally used in the hand-shake process, but can be queried directly.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IbcChannel {
    pub endpoint: IbcEndpoint,
    pub counterparty_endpoint: IbcEndpoint,
    pub order: IbcOrder,
    pub version: String,
    /// CounterpartyVersion can be None when not known this context, yet
    pub counterparty_version: Option<String>,
    /// The connection upon which this channel was created. If this is a multi-hop
    /// channel, we only expose the first hop.
    pub connection_id: String,
}

// TODO: check what representation we want here for encoding - string or number
/// IbcOrder defines if a channel is ORDERED or UNORDERED
#[repr(u8)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum IbcOrder {
    Unordered = 1,
    Ordered = 2,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IbcPacket {
    /// The raw data send from the other side in the packet
    pub data: Binary,
    /// identifies the channel and port on the sending chain.
    pub src: IbcEndpoint,
    /// identifies the channel and port on the receiving chain.
    pub dest: IbcEndpoint,
    /// The sequence number of the packet on the given channel
    pub sequence: u64,
    /// block height after which the packet times out
    pub timeout_height: u64,
    /// block timestamp (in nanoseconds) after which the packet times out
    pub timeout_timestamp: u64,
    // the version that the client is currently on
    pub version: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IbcAcknowledgement {
    pub acknowledgement: Binary,
    pub original_packet: IbcPacket,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IbcConnectResponse<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    pub messages: Vec<CosmosMsg<T>>,
    /// The attributes that will be emitted as part of a "wasm" event
    pub attributes: Vec<Attribute>,
}

impl<T> Default for IbcConnectResponse<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn default() -> Self {
        IbcConnectResponse {
            messages: vec![],
            attributes: vec![],
        }
    }
}

// type IBCPacketReceiveResponse struct {
//     // Acknowledgement contains the data to acknowledge the ibc packet execution
//     Acknowledgement []byte `json:"acknowledgement"`
//     // Messages comes directly from the contract and is it's request for action
//     Messages []CosmosMsg `json:"messages,omitempty"`
//     // log message to return over abci interface
//     Attributes []cosmwasmv1.EventAttribute `json:"attributes"`
// }
//
// type IBCPacketAcknowledgementResponse struct {
//     Messages   []CosmosMsg                 `json:"messages"`
//     Attributes []cosmwasmv1.EventAttribute `json:"attributes"`
// }
//
// type IBCPacketTimeoutResponse struct {
//     Messages   []CosmosMsg                 `json:"messages"`
//     Attributes []cosmwasmv1.EventAttribute `json:"attributes"`
// }
//
// type IBCChannelOpenResponse struct {
//     // Success contains a boolean if the channel would be accepted
//     Success bool `json:"result"`
//     // Reason optional description why it was not accepted
//     Reason string `json:"reason"`
// }
//
// type IBCChannelConnectResponse struct {
//     Messages   []CosmosMsg                 `json:"messages"`
//     Attributes []cosmwasmv1.EventAttribute `json:"attributes"`
// }
//
// type IBCChannelCloseResponse struct {
//     Messages   []CosmosMsg                 `json:"messages"`
//     Attributes []cosmwasmv1.EventAttribute `json:"attributes"`
// }
