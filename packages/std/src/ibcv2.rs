use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Addr, Binary, Timestamp};

/// Payload value should be encoded in a format defined by the channel version,
/// and the module on the other side should know how to parse this.
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct IBCv2Payload {
    /// The port id on the chain where the packet is sent from.
    pub source_port: String,
    /// The port id on the chain where the packet is sent to.
    pub destination_port: String,
    /// Version of the receiving contract.
    pub version: String,
    /// Encoding used to serialize the [IBCv2Payload::value].
    pub encoding: String,
    /// Encoded payload data.
    pub value: Binary,
}

/// These are messages in the IBC lifecycle using the new IBCv2 approach.
/// Only usable by IBCv2-enabled contracts
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IBCv2Msg {
    /// Sends an IBCv2 packet with given payloads over the existing channel.
    SendPacket {
        channel_id: String,
        timeout: Timestamp,
        payloads: Vec<IBCv2Payload>,
    },
}

/// The message that is passed into `ibcv2_packet_receive`
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[non_exhaustive]
pub struct IBCv2PacketReceiveMsg {
    pub payload: IBCv2Payload,
    pub relayer: Addr,
    pub source_client: String,
}

impl IBCv2PacketReceiveMsg {
    pub fn new(payload: IBCv2Payload, relayer: Addr, source_client: String) -> Self {
        Self {
            payload,
            relayer,
            source_client,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::to_string;

    use crate::IBCv2Payload;

    #[test]
    fn ibcv2_payload_serialize() {
        let packet = IBCv2Payload {
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
