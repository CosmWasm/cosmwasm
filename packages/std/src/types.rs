use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::addresses::HumanAddr;
use crate::coins::Coin;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Env {
    pub block: BlockInfo,
    pub contract: ContractInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
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
    /// #     contract: ContractInfo {
    /// #         address: HumanAddr::from("contract"),
    /// #     },
    /// # };
    /// let millis = (env.block.time * 1_000) + (env.block.time_nanos / 1_000_000);
    /// ```
    pub time_nanos: u64,
    pub chain_id: String,
}

/// Additional information from [MsgInstantiateContract] and [MsgExecuteContract], which is passed
/// along with the contract execution message into the `instantiate` and `execute` entry points.
///
/// It contains the essential info for authorization - identity of the call, and payment.
///
/// [MsgInstantiateContract]: https://github.com/CosmWasm/wasmd/blob/v0.15.0/x/wasm/internal/types/tx.proto#L47-L61
/// [MsgExecuteContract]: https://github.com/CosmWasm/wasmd/blob/v0.15.0/x/wasm/internal/types/tx.proto#L68-L78
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MessageInfo {
    /// The `sender` field from `MsgInstantiateContract` and `MsgExecuteContract`.
    /// You can think of this as the address that initiated the action (i.e. the message). What that
    /// means exactly heavily depends on the application.
    ///
    /// The x/wasm module ensures that the sender address signed the transaction or
    /// is otherwise authorized to send the message.
    ///
    /// Additional signers of the transaction that are either needed for other messages or contain unnecessary
    /// signatures are not propagated into the contract.
    pub sender: HumanAddr,
    /// The funds that are sent to the contract as part of `MsgInstantiateContract`
    /// or `MsgExecuteContract`. The transfer is processed in bank before the contract
    /// is executed such that the new balance is visible during contract execution.
    pub funds: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractInfo {
    pub address: HumanAddr,
}
