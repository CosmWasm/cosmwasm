use cosmwasm_std::Ibc2PacketSendMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub ibc2_packet_ack_counter: u32,
    pub ibc2_packet_receive_counter: u32,
    pub ibc2_packet_timeout_counter: u32,
    pub last_source_client: String,
    pub last_packet_seq: u64,
    pub last_packet_sent: Option<Ibc2PacketSendMsg>,
}

pub const STATE_KEY: &[u8] = b"state";
