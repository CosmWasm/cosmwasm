use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::addresses::HumanAddr;
use crate::coins::Coin;

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct Env {
    pub block: BlockInfo,
    pub message: MessageInfo,
    pub contract: ContractInfo,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct BlockInfo {
    pub height: u64,
    /// Absolute time of the block creation in seconds since the UNIX epoch (00:00:00 on 1970-01-01 UTC).
    ///
    /// The source of this is the [BFT Time in Tendermint](https://docs.tendermint.com/master/spec/consensus/bft-time.html),
    /// converted from nanoseconds to second precision by truncating the fractioal part.
    pub time: u64,
    /// The fractional part of the block time in nanoseconds since `time` (0 to 999999999).
    /// Add this to `time` if you need a high precision block time.
    ///
    /// # Examples
    ///
    /// Using chrono:
    ///
    /// ```
    /// # use cosmwasm_std::{BlockInfo, ContractInfo, Env, HumanAddr, MessageInfo};
    /// # let env = Env {
    /// #     block: BlockInfo {
    /// #         height: 12_345,
    /// #         time: 1_571_797_419,
    /// #         time_nanos: 879305533,
    /// #         chain_id: "cosmos-testnet-14002".to_string(),
    /// #     },
    /// #     message: MessageInfo {
    /// #         sender: HumanAddr::from("alice"),
    /// #         sent_funds: vec![],
    /// #     },
    /// #     contract: ContractInfo {
    /// #         address: HumanAddr::from("contract"),
    /// #     },
    /// # };
    /// # extern crate chrono;
    /// use chrono::NaiveDateTime;
    /// let dt = NaiveDateTime::from_timestamp(env.block.time as i64, env.block.time_nanos as u32);
    /// ```
    ///
    /// Creating a simple millisecond-precision timestamp (as used in JavaScript):
    ///
    /// ```
    /// # use cosmwasm_std::{BlockInfo, ContractInfo, Env, HumanAddr, MessageInfo};
    /// # let env = Env {
    /// #     block: BlockInfo {
    /// #         height: 12_345,
    /// #         time: 1_571_797_419,
    /// #         time_nanos: 879305533,
    /// #         chain_id: "cosmos-testnet-14002".to_string(),
    /// #     },
    /// #     message: MessageInfo {
    /// #         sender: HumanAddr::from("alice"),
    /// #         sent_funds: vec![],
    /// #     },
    /// #     contract: ContractInfo {
    /// #         address: HumanAddr::from("contract"),
    /// #     },
    /// # };
    /// let millis = (env.block.time * 1_000) + (env.block.time_nanos / 1_000_000);
    /// ```
    pub time_nanos: u64,
    pub chain_id: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct MessageInfo {
    /// The `sender` field from the `wasm/MsgStoreCode`, `wasm/MsgInstantiateContract`, `wasm/MsgMigrateContract`
    /// or `wasm/MsgExecuteContract` message.
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
