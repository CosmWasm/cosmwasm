#![cfg(feature = "stargate")]
// The CosmosMsg variants are defined in results/cosmos_msg.rs
// The rest of the IBC related functionality is defined here

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::addresses::HumanAddr;
use crate::binary::Binary;
use crate::coins::Coin;
use crate::results::{Attribute, CosmosMsg};
use crate::types::Empty;

/// These are messages in the IBC lifecycle. Only usable by IBC-enabled contracts
/// (contracts that directly speak the IBC protocol via 6 entry points)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IbcMsg {
    /// Sends bank tokens owned by the contract to the given address on another chain.
    /// The channel must already be established between the ibctransfer module on this chain
    /// and a matching module on the remote chain.
    /// We cannot select the port_id, this is whatever the local chain has bound the ibctransfer
    /// module to.
    Ics20Transfer {
        /// exisiting channel to send the tokens over
        channel_id: String,
        /// address on the remote chain to receive these tokens
        to_address: HumanAddr,
        /// packet data only supports one coin
        /// https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/ibc/applications/transfer/v1/transfer.proto#L11-L20
        amount: Coin,
    },
    /// Sends an IBC packet with given data over the existing channel.
    /// Data should be encoded in a format defined by the channel version,
    /// and the module on the other side should know how to parse this.
    SendPacket {
        channel_id: String,
        data: Binary,
        timeout_height: IbcTimeoutHeight,
        timeout_timestamp: u64,
        version: u64,
    },
    /// This will close an existing channel that is owned by this contract.
    /// Port is auto-assigned to the contracts' ibc port
    CloseChannel { channel_id: String },
}

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
/// Values come from https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/ibc/core/channel/v1/channel.proto#L69-L80
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum IbcOrder {
    Unordered = 1,
    Ordered = 2,
}

// IBCTimeoutHeight Height is a monotonically increasing data type
// that can be compared against another Height for the purposes of updating and
// freezing clients.
// Ordering is (revision_number, timeout_height)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IbcTimeoutHeight {
    /// the version that the client is currently on
    /// (eg. after reseting the chain this could increment 1 as height drops to 0)
    pub revision_number: u64,
    /// block height after which the packet times out.
    /// the height within the given revision
    pub timeout_height: u64,
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
    pub timeout_height: IbcTimeoutHeight,
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

/// This is the return value for the majority of the ibc handlers.
/// That are able to dispatch messages / events on their own,
/// but have no meaningful return value to the calling code.
///
/// Callbacks that have return values (like receive_packet)
/// or that cannot redispatch messages (like the handshake callbacks)
/// will use other Response types
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IbcBasicResponse<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    pub messages: Vec<CosmosMsg<T>>,
    /// The attributes that will be emitted as part of a "wasm" event
    pub attributes: Vec<Attribute>,
}

impl<T> Default for IbcBasicResponse<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn default() -> Self {
        IbcBasicResponse {
            messages: vec![],
            attributes: vec![],
        }
    }
}

/// This is the return value for the majority of the ibc handlers.
/// That are able to dispatch messages / events on their own,
/// but have no meaningful return value to the calling code.
///
/// Callbacks that have return values (like receive_packet)
/// or that cannot redispatch messages (like the handshake callbacks)
/// will use other Response types
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IbcReceiveResponse<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    /// The bytes we return to the contract that sent the packet.
    /// This may represent a success or error of exection
    pub acknowledgement: Binary,
    pub messages: Vec<CosmosMsg<T>>,
    /// The attributes that will be emitted as part of a "wasm" event
    pub attributes: Vec<Attribute>,
}

impl<T> Default for IbcReceiveResponse<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn default() -> Self {
        IbcReceiveResponse {
            acknowledgement: Binary(vec![]),
            messages: vec![],
            attributes: vec![],
        }
    }
}
