use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Addr, Binary, IbcAcknowledgement, Timestamp};

/// Payload value should be encoded in a format defined by the channel version,
/// and the module on the other side should know how to parse this.
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Ibc2Payload {
    /// The port id on the chain where the packet is sent from.
    pub source_port: String,
    /// The port id on the chain where the packet is sent to.
    pub destination_port: String,
    /// Version of the receiving contract.
    pub version: String,
    /// Encoding used to serialize the [Ibc2Payload::value].
    pub encoding: String,
    /// Encoded payload data.
    pub value: Binary,
}

impl Ibc2Payload {
    pub fn new(
        source_port: String,
        destination_port: String,
        version: String,
        encoding: String,
        value: Binary,
    ) -> Self {
        Self {
            source_port,
            destination_port,
            version,
            encoding,
            value,
        }
    }
}

/// These are messages in the IBC lifecycle using the new Ibc2 approach.
/// Only usable by Ibc2-enabled contracts
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Ibc2Msg {
    /// Sends an Ibc2 packet with given payloads over the existing channel.
    SendPacket {
        source_client: String,
        timeout: Timestamp,
        payloads: Vec<Ibc2Payload>,
    },
    /// Acknowledges a packet that this contract received over IBC.
    /// This allows acknowledging a packet that was not acknowledged yet in the `ibc2_packet_receive` call.
    WriteAcknowledgement {
        /// Existing channel where the packet was received
        source_client: String,
        /// Sequence number of the packet that was received
        packet_sequence: u64,
        /// The acknowledgement to send back
        ack: IbcAcknowledgement,
    },
}

/// The message that is passed into `ibc2_packet_receive`
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[non_exhaustive]
pub struct Ibc2PacketReceiveMsg {
    pub payload: Ibc2Payload,
    pub relayer: Addr,
    pub source_client: String,
    pub packet_sequence: u64,
}

impl Ibc2PacketReceiveMsg {
    pub fn new(
        payload: Ibc2Payload,
        relayer: Addr,
        source_client: String,
        packet_sequence: u64,
    ) -> Self {
        Self {
            payload,
            relayer,
            source_client,
            packet_sequence,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::to_string;

    use crate::Ibc2Payload;

    #[test]
    fn ibc2_payload_serialize() {
        let packet = Ibc2Payload {
            source_port: "sending-contractr-port".to_string(),
            destination_port: "receiving-contract-port".to_string(),
            version: "v1".to_string(),
            encoding: "json".to_string(),
            value: b"foo".into(),
        };
        let expected = r#"{"source_port":"sending-contractr-port","destination_port":"receiving-contract-port","version":"v1","encoding":"json","value":"Zm9v"}"#;
        assert_eq!(to_string(&packet).unwrap(), expected);
    }
}
