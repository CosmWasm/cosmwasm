use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub ibc2_packet_receive_counter: u32,
}

pub const STATE_KEY: &[u8] = b"state";
