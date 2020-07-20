use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::coins::Coin;
use crate::encoding::Binary;

// Added Eq and Hash to allow this to be a key in a HashMap (MockQuerier)
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, JsonSchema, Hash)]
pub struct HumanAddr(pub String);

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct CanonicalAddr(pub Binary);

impl HumanAddr {
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for HumanAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl From<&str> for HumanAddr {
    fn from(addr: &str) -> Self {
        HumanAddr(addr.to_string())
    }
}

impl From<&HumanAddr> for HumanAddr {
    fn from(addr: &HumanAddr) -> Self {
        HumanAddr(addr.0.to_string())
    }
}

impl From<&&HumanAddr> for HumanAddr {
    fn from(addr: &&HumanAddr) -> Self {
        HumanAddr(addr.0.to_string())
    }
}

impl From<String> for HumanAddr {
    fn from(addr: String) -> Self {
        HumanAddr(addr)
    }
}

impl CanonicalAddr {
    pub fn as_slice(&self) -> &[u8] {
        &self.0.as_slice()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for CanonicalAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct Env {
    pub block: BlockInfo,
    pub message: MessageInfo,
    pub contract: ContractInfo,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct BlockInfo {
    pub height: u64,
    // time is seconds since epoch begin (Jan. 1, 1970)
    pub time: u64,
    pub chain_id: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct MessageInfo {
    /// The `sender` field from the wasm/store-code, wasm/instantiate or wasm/execute message.
    /// You can think of this as the address that initiated the action (i.e. the message). What that
    /// means exactly heavily depends on the application.
    ///
    /// The x/wasm module ensures that the sender address signed the transaction.
    /// Additional signers of the transaction that are either needed for other messages or contain unnecessary
    /// signatures are not propagated into the contract.
    ///
    /// There is a discussion to open up this field to multiple initiators, which you're welcome to join
    /// if you have a specific need for that feature: https://github.com/CosmWasm/cosmwasm/issues/293
    pub sender: HumanAddr,
    pub sent_funds: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct ContractInfo {
    pub address: HumanAddr,
}

/// An empty struct that serves as a placeholder in different places,
/// such as contracts that don't set a custom message.
///
/// It is designed to be expressable in correct JSON and JSON Schema but
/// contains no meaningful data. Previously we used enums without cases,
/// but those cannot represented as valid JSON Schema (https://github.com/CosmWasm/cosmwasm/issues/451)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Empty {}

#[cfg(test)]
mod test {
    use super::*;

    use crate::serde::{from_slice, to_vec};

    #[test]
    fn empty_can_be_instantiated_serialized_and_deserialized() {
        let instance = Empty {};
        let serialized = to_vec(&instance).unwrap();
        assert_eq!(serialized, b"{}");

        let deserialized: Empty = from_slice(b"{}").unwrap();
        assert_eq!(deserialized, instance);

        let deserialized: Empty = from_slice(b"{\"stray\":\"data\"}").unwrap();
        assert_eq!(deserialized, instance);
    }
}
