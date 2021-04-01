use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;

pub const CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct State {
    pub verifier: Addr,
    pub beneficiary: Addr,
    pub funder: Addr,
}
