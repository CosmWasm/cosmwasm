#![cfg(feature = "stargate")]
// The CosmosMsg variants are defined in results/cosmos_msg.rs
// The rest of the IBC related functionality is defined here

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::{Ord, Ordering, PartialOrd};
use std::fmt;

use crate::binary::Binary;
use crate::coins::Coin;
use crate::results::{Attribute, CosmosMsg, Empty, SubMsg};

/// These are messages in the IBC lifecycle. Only usable by IBC-enabled contracts
/// (contracts that directly speak the IBC protocol via 6 entry points)
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IbcMsg {
    /// Sends bank tokens owned by the contract to the given address on another chain.
    /// The channel must already be established between the ibctransfer module on this chain
    /// and a matching module on the remote chain.
    /// We cannot select the port_id, this is whatever the local chain has bound the ibctransfer
    /// module to.
    Transfer {
        /// exisiting channel to send the tokens over
        channel_id: String,
        /// address on the remote chain to receive these tokens
        to_address: String,
        /// packet data only supports one coin
        /// https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/ibc/applications/transfer/v1/transfer.proto#L11-L20
        amount: Coin,
        /// block after which the packet times out.
        /// at least one of timeout_block, timeout_timestamp is required
        timeout_block: Option<IbcTimeoutBlock>,
        /// block timestamp (nanoseconds since UNIX epoch) after which the packet times out.
        /// See https://golang.org/pkg/time/#Time.UnixNano
        /// at least one of timeout_block, timeout_timestamp is required
        timeout_timestamp: Option<u64>,
    },
    /// Sends an IBC packet with given data over the existing channel.
    /// Data should be encoded in a format defined by the channel version,
    /// and the module on the other side should know how to parse this.
    SendPacket {
        channel_id: String,
        data: Binary,
        /// block height after which the packet times out.
        /// at least one of timeout_block, timeout_timestamp is required
        timeout_block: Option<IbcTimeoutBlock>,
        /// block timestamp (nanoseconds since UNIX epoch) after which the packet times out.
        /// See https://golang.org/pkg/time/#Time.UnixNano
        /// at least one of timeout_block, timeout_timestamp is required
        timeout_timestamp: Option<u64>,
    },
    /// This will close an existing channel that is owned by this contract.
    /// Port is auto-assigned to the contracts' ibc port
    CloseChannel { channel_id: String },
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

/// IbcOrder defines if a channel is ORDERED or UNORDERED
/// Values come from https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/ibc/core/channel/v1/channel.proto#L69-L80
/// Naming comes from the protobuf files and go translations.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct IbcTimeoutBlock {
    /// the version that the client is currently on
    /// (eg. after reseting the chain this could increment 1 as height drops to 0)
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
    /// block height after which the packet times out.
    /// at least one of timeout_block, timeout_timestamp is required
    pub timeout_block: Option<IbcTimeoutBlock>,
    /// block timestamp (nanoseconds since UNIX epoch) after which the packet times out.
    /// See https://golang.org/pkg/time/#Time.UnixNano
    /// at least one of timeout_block, timeout_timestamp is required
    pub timeout_timestamp: Option<u64>,
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
    /// Optional list of "subcalls" to make. These will be executed in order
    /// (and this contract's subcall_response entry point invoked)
    /// *before* any of the "fire and forget" messages get executed.
    pub submessages: Vec<SubMsg<T>>,
    /// After any submessages are processed, these are all dispatched in the host blockchain.
    /// If they all succeed, then the transaction is committed. If any fail, then the transaction
    /// and any local contract state changes are reverted.
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
            submessages: vec![],
            messages: vec![],
            attributes: vec![],
        }
    }
}

// This defines the return value on packet response processing.
// This "success" case should be returned even in application-level errors,
// Where the acknowledgement bytes contain an encoded error message to be returned to
// the calling chain. (Returning ContractResult::Err will abort processing of this packet
// and not inform the calling chain).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IbcReceiveResponse<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    /// The bytes we return to the contract that sent the packet.
    /// This may represent a success or error of exection
    pub acknowledgement: Binary,
    /// Optional list of "subcalls" to make. These will be executed in order
    /// (and this contract's subcall_response entry point invoked)
    /// *before* any of the "fire and forget" messages get executed.
    pub submessages: Vec<SubMsg<T>>,
    /// After any submessages are processed, these are all dispatched in the host blockchain.
    /// If they all succeed, then the transaction is committed. If any fail, then the transaction
    /// and any local contract state changes are reverted.
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
            submessages: vec![],
            messages: vec![],
            attributes: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json_wasm::to_string;

    #[test]
    // added this to check json format for go compat, as I was unsure how some messages are snake encoded
    fn serialize_msg() {
        let msg = IbcMsg::Transfer {
            channel_id: "channel-123".to_string(),
            to_address: "my-special-addr".into(),
            amount: Coin::new(12345678, "uatom"),
            timeout_block: None,
            timeout_timestamp: Some(1234567890),
        };
        let encoded = to_string(&msg).unwrap();
        let expected = r#"{"transfer":{"channel_id":"channel-123","to_address":"my-special-addr","amount":{"denom":"uatom","amount":"12345678"},"timeout_block":null,"timeout_timestamp":1234567890}}"#;
        assert_eq!(encoded.as_str(), expected);
    }

    #[test]
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
        assert!(epoch1a == epoch1a);
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
}
