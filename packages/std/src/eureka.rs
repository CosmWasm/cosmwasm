use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Binary, Timestamp};

/// Payload value should be encoded in a format defined by the channel version,
/// and the module on the other side should know how to parse this.
#[non_exhaustive]
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
#[serde(rename_all = "snake_case")]
pub struct EurekaPayload {
    /// The port id on the chain where the packet is sent to (external chain).
    pub destination_port: String,
    /// Version of the receiving contract.
    pub version: String,
    /// Encoding used to serialize the [EurekaPayload::value].
    pub encoding: String,
    /// Encoded payload data.
    pub value: Binary,
}

/// These are messages in the IBC lifecycle using the new Eureka approach. Only usable by IBC-enabled contracts
#[non_exhaustive]
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
#[serde(rename_all = "snake_case")]
pub enum EurekaMsg {
    /// Sends an IBC packet with given payloads over the existing channel.
    SendPacket {
        /// existing channel to send the tokens over
        channel_id: String,
        timeout: Timestamp,
        payloads: Vec<EurekaPayload>,
    },
}

#[cfg(test)]
mod tests {
    use serde_json::to_string;

    use crate::EurekaPayload;

    #[test]
    fn eureka_payload_serialize() {
        let packet = EurekaPayload {
            destination_port: "receiving-contract-port".to_string(),
            version: "v1".to_string(),
            encoding: "json".to_string(),
            value: b"foo".into(),
        };
        let expected = r#"{"destination_port":"receiving-contract-port","version":"v1","encoding":"json","value":"Zm9v"}"#;
        assert_eq!(to_string(&packet).unwrap(), expected);
    }
}
