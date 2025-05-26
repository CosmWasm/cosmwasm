// The CosmosMsg variants are defined in results/cosmos_msg.rs
// The rest of the IBC related functionality is defined here

use core::cmp::{Ord, Ordering, PartialOrd};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::coin::Coin;
use crate::prelude::*;
use crate::results::{Attribute, CosmosMsg, Empty, Event, SubMsg};
use crate::StdResult;
use crate::{to_json_binary, Binary};
use crate::{Addr, Timestamp};

mod callbacks;
mod transfer_msg_builder;

pub use callbacks::*;
pub use transfer_msg_builder::*;

/// These are messages in the IBC lifecycle. Only usable by IBC-enabled contracts
/// (contracts that directly speak the IBC protocol via 6 entry points)
#[non_exhaustive]
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
#[serde(rename_all = "snake_case")]
pub enum IbcMsg {
    /// Sends bank tokens owned by the contract to the given address on another chain.
    /// The channel must already be established between the ibctransfer module on this chain
    /// and a matching module on the remote chain.
    /// We cannot select the port_id, this is whatever the local chain has bound the ibctransfer
    /// module to.
    Transfer {
        /// existing channel to send the tokens over
        channel_id: String,
        /// address on the remote chain to receive these tokens
        to_address: String,
        /// packet data only supports one coin
        /// https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/ibc/applications/transfer/v1/transfer.proto#L11-L20
        amount: Coin,
        /// when packet times out, measured on remote chain
        timeout: IbcTimeout,
        /// An optional memo. See the blog post
        /// ["Moving Beyond Simple Token Transfers"](https://medium.com/the-interchain-foundation/moving-beyond-simple-token-transfers-d42b2b1dc29b)
        /// for more information.
        ///
        /// There is no difference between setting this to `None` or an empty string.
        ///
        /// This field is only supported on chains with CosmWasm >= 2.0 and silently
        /// ignored on older chains.
        /// If you need support for both 1.x and 2.x chain with the same codebase,
        /// it is recommended to use `CosmosMsg::Stargate` with a custom MsgTransfer
        /// protobuf encoder instead.
        memo: Option<String>,
    },
    /// Sends an IBC packet with given data over the existing channel.
    /// Data should be encoded in a format defined by the channel version,
    /// and the module on the other side should know how to parse this.
    SendPacket {
        channel_id: String,
        data: Binary,
        /// when packet times out, measured on remote chain
        timeout: IbcTimeout,
    },
    /// Acknowledges a packet that this contract received over IBC.
    /// This allows acknowledging a packet that was not acknowledged yet in the `ibc_packet_receive` call.
    #[cfg(feature = "cosmwasm_2_1")]
    WriteAcknowledgement {
        /// Existing channel where the packet was received
        channel_id: String,
        /// Sequence number of the packet that was received
        packet_sequence: u64,
        /// The acknowledgement to send back
        ack: IbcAcknowledgement,
    },
    /// This will close an existing channel that is owned by this contract.
    /// Port is auto-assigned to the contract's IBC port
    CloseChannel { channel_id: String },
}

#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
pub struct IbcEndpoint {
    pub port_id: String,
    pub channel_id: String,
}

/// In IBC each package must set at least one type of timeout:
/// the timestamp or the block height. Using this rather complex enum instead of
/// two timeout fields we ensure that at least one timeout is set.
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
#[serde(rename_all = "snake_case")]
pub struct IbcTimeout {
    // use private fields to enforce the use of constructors, which ensure that at least one is set
    block: Option<IbcTimeoutBlock>,
    timestamp: Option<Timestamp>,
}

impl IbcTimeout {
    pub fn with_block(block: IbcTimeoutBlock) -> Self {
        IbcTimeout {
            block: Some(block),
            timestamp: None,
        }
    }

    pub fn with_timestamp(timestamp: Timestamp) -> Self {
        IbcTimeout {
            block: None,
            timestamp: Some(timestamp),
        }
    }

    pub fn with_both(block: IbcTimeoutBlock, timestamp: Timestamp) -> Self {
        IbcTimeout {
            block: Some(block),
            timestamp: Some(timestamp),
        }
    }

    pub fn block(&self) -> Option<IbcTimeoutBlock> {
        self.block
    }

    pub fn timestamp(&self) -> Option<Timestamp> {
        self.timestamp
    }
}

impl From<Timestamp> for IbcTimeout {
    fn from(timestamp: Timestamp) -> IbcTimeout {
        IbcTimeout::with_timestamp(timestamp)
    }
}

impl From<IbcTimeoutBlock> for IbcTimeout {
    fn from(original: IbcTimeoutBlock) -> IbcTimeout {
        IbcTimeout::with_block(original)
    }
}

// These are various messages used in the callbacks

/// IbcChannel defines all information on a channel.
/// This is generally used in the hand-shake process, but can be queried directly.
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
#[non_exhaustive]
pub struct IbcChannel {
    pub endpoint: IbcEndpoint,
    pub counterparty_endpoint: IbcEndpoint,
    pub order: IbcOrder,
    /// Note: in ibcv3 this may be "", in the IbcOpenChannel handshake messages
    pub version: String,
    /// The connection upon which this channel was created. If this is a multi-hop
    /// channel, we only expose the first hop.
    pub connection_id: String,
}

impl IbcChannel {
    /// Construct a new IbcChannel.
    pub fn new(
        endpoint: IbcEndpoint,
        counterparty_endpoint: IbcEndpoint,
        order: IbcOrder,
        version: impl Into<String>,
        connection_id: impl Into<String>,
    ) -> Self {
        Self {
            endpoint,
            counterparty_endpoint,
            order,
            version: version.into(),
            connection_id: connection_id.into(),
        }
    }
}

/// IbcOrder defines if a channel is ORDERED or UNORDERED
/// Values come from https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/ibc/core/channel/v1/channel.proto#L69-L80
/// Naming comes from the protobuf files and go translations.
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
pub enum IbcOrder {
    #[serde(rename = "ORDER_UNORDERED")]
    Unordered,
    #[serde(rename = "ORDER_ORDERED")]
    Ordered,
}

/// IBCTimeoutHeight Height is a monotonically increasing data type
/// that can be compared against another Height for the purposes of updating and
/// freezing clients.
/// Ordering is (revision_number, timeout_height)
#[derive(
    Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
pub struct IbcTimeoutBlock {
    /// the version that the client is currently on
    /// (e.g. after resetting the chain this could increment 1 as height drops to 0)
    pub revision: u64,
    /// block height after which the packet times out.
    /// the height within the given revision
    pub height: u64,
}

impl IbcTimeoutBlock {
    pub fn is_zero(&self) -> bool {
        self.revision == 0 && self.height == 0
    }
}

impl PartialOrd for IbcTimeoutBlock {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IbcTimeoutBlock {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.revision.cmp(&other.revision) {
            Ordering::Equal => self.height.cmp(&other.height),
            other => other,
        }
    }
}

#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
#[non_exhaustive]
pub struct IbcPacket {
    /// The raw data sent from the other side in the packet
    pub data: Binary,
    /// identifies the channel and port on the sending chain.
    pub src: IbcEndpoint,
    /// identifies the channel and port on the receiving chain.
    pub dest: IbcEndpoint,
    /// The sequence number of the packet on the given channel
    pub sequence: u64,
    pub timeout: IbcTimeout,
}

impl IbcPacket {
    /// Construct a new IbcPacket.
    pub fn new(
        data: impl Into<Binary>,
        src: IbcEndpoint,
        dest: IbcEndpoint,
        sequence: u64,
        timeout: IbcTimeout,
    ) -> Self {
        Self {
            data: data.into(),
            src,
            dest,
            sequence,
            timeout,
        }
    }
}

#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
#[non_exhaustive]
pub struct IbcAcknowledgement {
    pub data: Binary,
    // we may add more info here in the future (meta-data from the acknowledgement)
    // there have been proposals to extend this type in core ibc for future versions
}

impl IbcAcknowledgement {
    pub fn new(data: impl Into<Binary>) -> Self {
        IbcAcknowledgement { data: data.into() }
    }

    pub fn encode_json(data: &impl Serialize) -> StdResult<Self> {
        Ok(IbcAcknowledgement {
            data: to_json_binary(data)?,
        })
    }
}

/// The message that is passed into `ibc_channel_open`
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
#[serde(rename_all = "snake_case")]
pub enum IbcChannelOpenMsg {
    /// The ChanOpenInit step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
    OpenInit { channel: IbcChannel },
    /// The ChanOpenTry step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
    OpenTry {
        channel: IbcChannel,
        counterparty_version: String,
    },
}

impl IbcChannelOpenMsg {
    pub fn new_init(channel: IbcChannel) -> Self {
        Self::OpenInit { channel }
    }

    pub fn new_try(channel: IbcChannel, counterparty_version: impl Into<String>) -> Self {
        Self::OpenTry {
            channel,
            counterparty_version: counterparty_version.into(),
        }
    }

    pub fn channel(&self) -> &IbcChannel {
        match self {
            Self::OpenInit { channel } => channel,
            Self::OpenTry { channel, .. } => channel,
        }
    }

    pub fn counterparty_version(&self) -> Option<&str> {
        match self {
            Self::OpenTry {
                counterparty_version,
                ..
            } => Some(counterparty_version),
            _ => None,
        }
    }
}

impl From<IbcChannelOpenMsg> for IbcChannel {
    fn from(msg: IbcChannelOpenMsg) -> IbcChannel {
        match msg {
            IbcChannelOpenMsg::OpenInit { channel } => channel,
            IbcChannelOpenMsg::OpenTry { channel, .. } => channel,
        }
    }
}

/// This serializes either as `null` or a JSON object.
/// Within the response, a channel version can be specified.
/// If `null` is provided instead, the incoming channel version is accepted.
pub type IbcChannelOpenResponse = Option<Ibc3ChannelOpenResponse>;

#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
pub struct Ibc3ChannelOpenResponse {
    /// We can set the channel version to a different one than we were called with
    pub version: String,
}

/// The message that is passed into `ibc_channel_connect`
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
#[serde(rename_all = "snake_case")]
pub enum IbcChannelConnectMsg {
    /// The ChanOpenAck step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
    OpenAck {
        channel: IbcChannel,
        counterparty_version: String,
    },
    /// The ChanOpenConfirm step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
    OpenConfirm { channel: IbcChannel },
}

impl IbcChannelConnectMsg {
    pub fn new_ack(channel: IbcChannel, counterparty_version: impl Into<String>) -> Self {
        Self::OpenAck {
            channel,
            counterparty_version: counterparty_version.into(),
        }
    }

    pub fn new_confirm(channel: IbcChannel) -> Self {
        Self::OpenConfirm { channel }
    }

    pub fn channel(&self) -> &IbcChannel {
        match self {
            Self::OpenAck { channel, .. } => channel,
            Self::OpenConfirm { channel } => channel,
        }
    }

    pub fn counterparty_version(&self) -> Option<&str> {
        match self {
            Self::OpenAck {
                counterparty_version,
                ..
            } => Some(counterparty_version),
            _ => None,
        }
    }
}

impl From<IbcChannelConnectMsg> for IbcChannel {
    fn from(msg: IbcChannelConnectMsg) -> IbcChannel {
        match msg {
            IbcChannelConnectMsg::OpenAck { channel, .. } => channel,
            IbcChannelConnectMsg::OpenConfirm { channel } => channel,
        }
    }
}

/// The message that is passed into `ibc_channel_close`
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
#[serde(rename_all = "snake_case")]
pub enum IbcChannelCloseMsg {
    /// The ChanCloseInit step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
    CloseInit { channel: IbcChannel },
    /// The ChanCloseConfirm step from https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management
    CloseConfirm { channel: IbcChannel }, // pub channel: IbcChannel,
}

impl IbcChannelCloseMsg {
    pub fn new_init(channel: IbcChannel) -> Self {
        Self::CloseInit { channel }
    }

    pub fn new_confirm(channel: IbcChannel) -> Self {
        Self::CloseConfirm { channel }
    }

    pub fn channel(&self) -> &IbcChannel {
        match self {
            Self::CloseInit { channel } => channel,
            Self::CloseConfirm { channel } => channel,
        }
    }
}

impl From<IbcChannelCloseMsg> for IbcChannel {
    fn from(msg: IbcChannelCloseMsg) -> IbcChannel {
        match msg {
            IbcChannelCloseMsg::CloseInit { channel } => channel,
            IbcChannelCloseMsg::CloseConfirm { channel } => channel,
        }
    }
}

/// The message that is passed into `ibc_packet_receive`
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
#[non_exhaustive]
pub struct IbcPacketReceiveMsg {
    pub packet: IbcPacket,
    pub relayer: Addr,
}

impl IbcPacketReceiveMsg {
    pub fn new(packet: IbcPacket, relayer: Addr) -> Self {
        Self { packet, relayer }
    }
}

/// The message that is passed into `ibc_packet_ack`
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
#[non_exhaustive]
pub struct IbcPacketAckMsg {
    pub acknowledgement: IbcAcknowledgement,
    pub original_packet: IbcPacket,
    pub relayer: Addr,
}

impl IbcPacketAckMsg {
    pub fn new(
        acknowledgement: IbcAcknowledgement,
        original_packet: IbcPacket,
        relayer: Addr,
    ) -> Self {
        Self {
            acknowledgement,
            original_packet,
            relayer,
        }
    }
}

/// The message that is passed into `ibc_packet_timeout`
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
#[non_exhaustive]
pub struct IbcPacketTimeoutMsg {
    pub packet: IbcPacket,
    pub relayer: Addr,
}

impl IbcPacketTimeoutMsg {
    pub fn new(packet: IbcPacket, relayer: Addr) -> Self {
        Self { packet, relayer }
    }
}

/// This is the return value for the majority of the ibc handlers.
/// That are able to dispatch messages / events on their own,
/// but have no meaningful return value to the calling code.
///
/// Callbacks that have return values (like receive_packet)
/// or that cannot redispatch messages (like the handshake callbacks)
/// will use other Response types
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
#[non_exhaustive]
pub struct IbcBasicResponse<T = Empty> {
    /// Optional list of messages to pass. These will be executed in order.
    /// If the ReplyOn member is set, they will invoke this contract's `reply` entry point
    /// after execution. Otherwise, they act like "fire and forget".
    /// Use `SubMsg::new` to create messages with the older "fire and forget" semantics.
    pub messages: Vec<SubMsg<T>>,
    /// The attributes that will be emitted as part of a `wasm` event.
    ///
    /// More info about events (and their attributes) can be found in [*Cosmos SDK* docs].
    ///
    /// [*Cosmos SDK* docs]: https://docs.cosmos.network/main/learn/advanced/events
    pub attributes: Vec<Attribute>,
    /// Extra, custom events separate from the main `wasm` one. These will have
    /// `wasm-` prepended to the type.
    ///
    /// More info about events can be found in [*Cosmos SDK* docs].
    ///
    /// [*Cosmos SDK* docs]: https://docs.cosmos.network/main/learn/advanced/events
    pub events: Vec<Event>,
}

// Custom implementation in order to implement it for all `T`, even if `T` is not `Default`.
impl<T> Default for IbcBasicResponse<T> {
    fn default() -> Self {
        IbcBasicResponse {
            messages: vec![],
            attributes: vec![],
            events: vec![],
        }
    }
}

impl<T> IbcBasicResponse<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an attribute included in the main `wasm` event.
    pub fn add_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push(Attribute::new(key, value));
        self
    }

    /// This creates a "fire and forget" message, by using `SubMsg::new()` to wrap it,
    /// and adds it to the list of messages to process.
    pub fn add_message(mut self, msg: impl Into<CosmosMsg<T>>) -> Self {
        self.messages.push(SubMsg::new(msg));
        self
    }

    /// This takes an explicit SubMsg (creates via e.g. `reply_on_error`)
    /// and adds it to the list of messages to process.
    pub fn add_submessage(mut self, msg: SubMsg<T>) -> Self {
        self.messages.push(msg);
        self
    }

    /// Adds an extra event to the response, separate from the main `wasm` event
    /// that is always created.
    ///
    /// The `wasm-` prefix will be appended by the runtime to the provided type
    /// of event.
    pub fn add_event(mut self, event: Event) -> Self {
        self.events.push(event);
        self
    }

    /// Bulk add attributes included in the main `wasm` event.
    ///
    /// Anything that can be turned into an iterator and yields something
    /// that can be converted into an `Attribute` is accepted.
    ///
    /// ## Examples
    ///
    /// ```
    /// use cosmwasm_std::{attr, IbcBasicResponse};
    ///
    /// let attrs = vec![
    ///     ("action", "reaction"),
    ///     ("answer", "42"),
    ///     ("another", "attribute"),
    /// ];
    /// let res: IbcBasicResponse = IbcBasicResponse::new().add_attributes(attrs.clone());
    /// assert_eq!(res.attributes, attrs);
    /// ```
    pub fn add_attributes<A: Into<Attribute>>(
        mut self,
        attrs: impl IntoIterator<Item = A>,
    ) -> Self {
        self.attributes.extend(attrs.into_iter().map(A::into));
        self
    }

    /// Bulk add "fire and forget" messages to the list of messages to process.
    ///
    /// ## Examples
    ///
    /// ```
    /// use cosmwasm_std::{CosmosMsg, IbcBasicResponse};
    ///
    /// fn make_response_with_msgs(msgs: Vec<CosmosMsg>) -> IbcBasicResponse {
    ///     IbcBasicResponse::new().add_messages(msgs)
    /// }
    /// ```
    pub fn add_messages<M: Into<CosmosMsg<T>>>(self, msgs: impl IntoIterator<Item = M>) -> Self {
        self.add_submessages(msgs.into_iter().map(SubMsg::new))
    }

    /// Bulk add explicit SubMsg structs to the list of messages to process.
    ///
    /// ## Examples
    ///
    /// ```
    /// use cosmwasm_std::{SubMsg, IbcBasicResponse};
    ///
    /// fn make_response_with_submsgs(msgs: Vec<SubMsg>) -> IbcBasicResponse {
    ///     IbcBasicResponse::new().add_submessages(msgs)
    /// }
    /// ```
    pub fn add_submessages(mut self, msgs: impl IntoIterator<Item = SubMsg<T>>) -> Self {
        self.messages.extend(msgs);
        self
    }

    /// Bulk add custom events to the response. These are separate from the main
    /// `wasm` event.
    ///
    /// The `wasm-` prefix will be appended by the runtime to the provided types
    /// of events.
    pub fn add_events(mut self, events: impl IntoIterator<Item = Event>) -> Self {
        self.events.extend(events);
        self
    }
}

/// This defines the return value on packet response processing.
/// This "success" case should be returned even in application-level errors,
/// Where the acknowledgement bytes contain an encoded error message to be returned to
/// the calling chain. (Returning ContractResult::Err will abort processing of this packet
/// and not inform the calling chain).
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
#[non_exhaustive]
pub struct IbcReceiveResponse<T = Empty> {
    /// The bytes we return to the contract that sent the packet.
    /// This may represent a success or error of execution.
    /// In case of `None`, no acknowledgement is written.
    pub acknowledgement: Option<Binary>,
    /// Optional list of messages to pass. These will be executed in order.
    /// If the ReplyOn member is set, they will invoke this contract's `reply` entry point
    /// after execution. Otherwise, they act like "fire and forget".
    /// Use `call` or `msg.into()` to create messages with the older "fire and forget" semantics.
    pub messages: Vec<SubMsg<T>>,
    /// The attributes that will be emitted as part of a "wasm" event.
    ///
    /// More info about events (and their attributes) can be found in [*Cosmos SDK* docs].
    ///
    /// [*Cosmos SDK* docs]: https://docs.cosmos.network/main/learn/advanced/events
    pub attributes: Vec<Attribute>,
    /// Extra, custom events separate from the main `wasm` one. These will have
    /// `wasm-` prepended to the type.
    ///
    /// More info about events can be found in [*Cosmos SDK* docs].
    ///
    /// [*Cosmos SDK* docs]: https://docs.cosmos.network/main/learn/advanced/events
    pub events: Vec<Event>,
}

impl<T> IbcReceiveResponse<T> {
    /// Create a new response with the given acknowledgement.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use cosmwasm_std::{
    /// #     Storage, Api, Querier, DepsMut, Deps, entry_point, Env, StdError, MessageInfo,
    /// #     Never, Response, QueryResponse, StdAck, IbcPacketReceiveMsg
    /// # };
    /// use cosmwasm_std::IbcReceiveResponse;
    ///
    /// #[entry_point]
    /// pub fn ibc_packet_receive(
    ///     deps: DepsMut,
    ///     env: Env,
    ///     msg: IbcPacketReceiveMsg,
    /// ) -> Result<IbcReceiveResponse, Never> {
    ///     // ...
    ///
    ///     // 0x01 is a FungibleTokenPacketSuccess from ICS-20.
    ///     Ok(IbcReceiveResponse::new(StdAck::success(b"\x01")))
    /// }
    /// ```
    pub fn new(ack: impl Into<Binary>) -> Self {
        Self {
            acknowledgement: Some(ack.into()),
            messages: vec![],
            attributes: vec![],
            events: vec![],
        }
    }

    /// Creates a new response without an acknowledgement.
    ///
    /// This allows you to send the acknowledgement asynchronously later using
    /// [`IbcMsg::WriteAcknowledgement`][self::IbcMsg#variant.WriteAcknowledgement].
    /// If you want to send the acknowledgement immediately, use [`IbcReceiveResponse::new`].
    ///
    /// ## Examples
    ///
    /// ```
    /// # use cosmwasm_std::{
    /// #     Storage, Api, Querier, DepsMut, Deps, entry_point, Env, StdError, MessageInfo,
    /// #     Never, Response, QueryResponse, StdAck, IbcPacketReceiveMsg
    /// # };
    /// use cosmwasm_std::IbcReceiveResponse;
    ///
    /// #[entry_point]
    /// pub fn ibc_packet_receive(
    ///     deps: DepsMut,
    ///     env: Env,
    ///     msg: IbcPacketReceiveMsg,
    /// ) -> Result<IbcReceiveResponse, Never> {
    ///     // ...
    ///
    ///     Ok(IbcReceiveResponse::without_ack())
    /// }
    /// ```
    pub fn without_ack() -> Self {
        Self {
            acknowledgement: None,
            messages: vec![],
            attributes: vec![],
            events: vec![],
        }
    }

    /// Add an attribute included in the main `wasm` event.
    pub fn add_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push(Attribute::new(key, value));
        self
    }

    /// This creates a "fire and forget" message, by using `SubMsg::new()` to wrap it,
    /// and adds it to the list of messages to process.
    pub fn add_message(mut self, msg: impl Into<CosmosMsg<T>>) -> Self {
        self.messages.push(SubMsg::new(msg));
        self
    }

    /// This takes an explicit SubMsg (creates via e.g. `reply_on_error`)
    /// and adds it to the list of messages to process.
    pub fn add_submessage(mut self, msg: SubMsg<T>) -> Self {
        self.messages.push(msg);
        self
    }

    /// Adds an extra event to the response, separate from the main `wasm` event
    /// that is always created.
    ///
    /// The `wasm-` prefix will be appended by the runtime to the provided type
    /// of event.
    pub fn add_event(mut self, event: Event) -> Self {
        self.events.push(event);
        self
    }

    /// Bulk add attributes included in the main `wasm` event.
    ///
    /// Anything that can be turned into an iterator and yields something
    /// that can be converted into an `Attribute` is accepted.
    ///
    /// ## Examples
    ///
    /// ```
    /// use cosmwasm_std::{attr, IbcReceiveResponse, StdAck};
    ///
    /// let attrs = vec![
    ///     ("action", "reaction"),
    ///     ("answer", "42"),
    ///     ("another", "attribute"),
    /// ];
    /// let res: IbcReceiveResponse = IbcReceiveResponse::new(StdAck::success(b"\x01")).add_attributes(attrs.clone());
    /// assert_eq!(res.attributes, attrs);
    /// ```
    pub fn add_attributes<A: Into<Attribute>>(
        mut self,
        attrs: impl IntoIterator<Item = A>,
    ) -> Self {
        self.attributes.extend(attrs.into_iter().map(A::into));
        self
    }

    /// Bulk add "fire and forget" messages to the list of messages to process.
    ///
    /// ## Examples
    ///
    /// ```
    /// use cosmwasm_std::{CosmosMsg, IbcReceiveResponse, StdAck};
    ///
    /// fn make_response_with_msgs(msgs: Vec<CosmosMsg>) -> IbcReceiveResponse {
    ///     IbcReceiveResponse::new(StdAck::success(b"\x01")).add_messages(msgs)
    /// }
    /// ```
    pub fn add_messages<M: Into<CosmosMsg<T>>>(self, msgs: impl IntoIterator<Item = M>) -> Self {
        self.add_submessages(msgs.into_iter().map(SubMsg::new))
    }

    /// Bulk add explicit SubMsg structs to the list of messages to process.
    ///
    /// ## Examples
    ///
    /// ```
    /// use cosmwasm_std::{SubMsg, StdAck, IbcReceiveResponse};
    ///
    /// fn make_response_with_submsgs(msgs: Vec<SubMsg>) -> IbcReceiveResponse {
    ///     IbcReceiveResponse::new(StdAck::success(b"\x01")).add_submessages(msgs)
    /// }
    /// ```
    pub fn add_submessages(mut self, msgs: impl IntoIterator<Item = SubMsg<T>>) -> Self {
        self.messages.extend(msgs);
        self
    }

    /// Bulk add custom events to the response. These are separate from the main
    /// `wasm` event.
    ///
    /// The `wasm-` prefix will be appended by the runtime to the provided types
    /// of events.
    pub fn add_events(mut self, events: impl IntoIterator<Item = Event>) -> Self {
        self.events.extend(events);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::to_json_string;

    #[test]
    // added this to check json format for go compat, as I was unsure how some messages are snake encoded
    fn serialize_msg() {
        let msg = IbcMsg::Transfer {
            channel_id: "channel-123".to_string(),
            to_address: "my-special-addr".into(),
            amount: Coin::new(12345678u128, "uatom"),
            timeout: IbcTimeout::with_timestamp(Timestamp::from_nanos(1234567890)),
            memo: None,
        };
        let encoded = to_json_string(&msg).unwrap();
        let expected = r#"{"transfer":{"channel_id":"channel-123","to_address":"my-special-addr","amount":{"denom":"uatom","amount":"12345678"},"timeout":{"block":null,"timestamp":"1234567890"},"memo":null}}"#;
        assert_eq!(encoded.as_str(), expected);
    }

    #[test]
    fn ibc_timeout_serialize() {
        let timestamp = IbcTimeout::with_timestamp(Timestamp::from_nanos(684816844));
        let expected = r#"{"block":null,"timestamp":"684816844"}"#;
        assert_eq!(to_json_string(&timestamp).unwrap(), expected);

        let block = IbcTimeout::with_block(IbcTimeoutBlock {
            revision: 12,
            height: 129,
        });
        let expected = r#"{"block":{"revision":12,"height":129},"timestamp":null}"#;
        assert_eq!(to_json_string(&block).unwrap(), expected);

        let both = IbcTimeout::with_both(
            IbcTimeoutBlock {
                revision: 12,
                height: 129,
            },
            Timestamp::from_nanos(684816844),
        );
        let expected = r#"{"block":{"revision":12,"height":129},"timestamp":"684816844"}"#;
        assert_eq!(to_json_string(&both).unwrap(), expected);
    }

    #[test]
    #[allow(clippy::eq_op)]
    fn ibc_timeout_block_ord() {
        let epoch1a = IbcTimeoutBlock {
            revision: 1,
            height: 1000,
        };
        let epoch1b = IbcTimeoutBlock {
            revision: 1,
            height: 3000,
        };
        let epoch2a = IbcTimeoutBlock {
            revision: 2,
            height: 500,
        };
        let epoch2b = IbcTimeoutBlock {
            revision: 2,
            height: 2500,
        };

        // basic checks
        assert_eq!(epoch1a, epoch1a);
        assert!(epoch1a < epoch1b);
        assert!(epoch1b > epoch1a);
        assert!(epoch2a > epoch1a);
        assert!(epoch2b > epoch1a);

        // ensure epoch boundaries are correctly handled
        assert!(epoch1b > epoch1a);
        assert!(epoch2a > epoch1b);
        assert!(epoch2b > epoch2a);
        assert!(epoch2b > epoch1b);
        // and check the inverse compare
        assert!(epoch1a < epoch1b);
        assert!(epoch1b < epoch2a);
        assert!(epoch2a < epoch2b);
        assert!(epoch1b < epoch2b);
    }

    #[test]
    fn ibc_packet_serialize() {
        let packet = IbcPacket {
            data: b"foo".into(),
            src: IbcEndpoint {
                port_id: "their-port".to_string(),
                channel_id: "channel-1234".to_string(),
            },
            dest: IbcEndpoint {
                port_id: "our-port".to_string(),
                channel_id: "chan33".into(),
            },
            sequence: 27,
            timeout: IbcTimeout::with_both(
                IbcTimeoutBlock {
                    revision: 1,
                    height: 12345678,
                },
                Timestamp::from_nanos(4611686018427387904),
            ),
        };
        let expected = r#"{"data":"Zm9v","src":{"port_id":"their-port","channel_id":"channel-1234"},"dest":{"port_id":"our-port","channel_id":"chan33"},"sequence":27,"timeout":{"block":{"revision":1,"height":12345678},"timestamp":"4611686018427387904"}}"#;
        assert_eq!(to_json_string(&packet).unwrap(), expected);

        let no_timestamp = IbcPacket {
            data: b"foo".into(),
            src: IbcEndpoint {
                port_id: "their-port".to_string(),
                channel_id: "channel-1234".to_string(),
            },
            dest: IbcEndpoint {
                port_id: "our-port".to_string(),
                channel_id: "chan33".into(),
            },
            sequence: 27,
            timeout: IbcTimeout::with_block(IbcTimeoutBlock {
                revision: 1,
                height: 12345678,
            }),
        };
        let expected = r#"{"data":"Zm9v","src":{"port_id":"their-port","channel_id":"channel-1234"},"dest":{"port_id":"our-port","channel_id":"chan33"},"sequence":27,"timeout":{"block":{"revision":1,"height":12345678},"timestamp":null}}"#;
        assert_eq!(to_json_string(&no_timestamp).unwrap(), expected);
    }
}
