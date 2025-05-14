use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::coin::Coin;
use crate::prelude::*;
use crate::Binary;
use crate::{Addr, Timestamp};

use crate::utils::impl_hidden_constructor;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Env {
    pub block: BlockInfo,
    /// Information on the transaction this message was executed in.
    /// The field is unset when the `MsgExecuteContract`/`MsgInstantiateContract`/`MsgMigrateContract`
    /// is not executed as part of a transaction.
    pub transaction: Option<TransactionInfo>,
    pub contract: ContractInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[non_exhaustive]
pub struct TransactionInfo {
    /// The position of this transaction in the block. The first
    /// transaction has index 0.
    ///
    /// This allows you to get a unique transaction identifier in this chain
    /// using the pair (`env.block.height`, `env.transaction.index`).
    ///
    pub index: u32,

    /// Checksum of the transaction.
    ///
    /// If the blockchain's CosmWasm version is below 3.0, this field
    /// will default to being empty.
    #[serde(default)]
    pub transaction_hash: Binary,
}

impl_hidden_constructor!(TransactionInfo, index: u32, transaction_hash: Binary);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct BlockInfo {
    /// The height of a block is the number of blocks preceding it in the blockchain.
    pub height: u64,
    /// Absolute time of the block creation in seconds since the UNIX epoch (00:00:00 on 1970-01-01 UTC).
    ///
    /// The source of this is the [BFT Time in Tendermint](https://github.com/tendermint/tendermint/blob/58dc1726/spec/consensus/bft-time.md),
    /// which has the same nanosecond precision as the `Timestamp` type.
    ///
    /// # Examples
    ///
    /// Using chrono:
    ///
    /// ```
    /// # use cosmwasm_std::{Addr, Binary, BlockInfo, ContractInfo, Env, MessageInfo, Timestamp, TransactionInfo};
    /// # let env = Env {
    /// #     block: BlockInfo {
    /// #         height: 12_345,
    /// #         time: Timestamp::from_nanos(1_571_797_419_879_305_533),
    /// #         chain_id: "cosmos-testnet-14002".to_string(),
    /// #     },
    /// #     transaction: Some(TransactionInfo::new(3, Binary::new(vec![0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8]))),
    /// #     contract: ContractInfo {
    /// #         address: Addr::unchecked("contract"),
    /// #     },
    /// # };
    /// # extern crate chrono;
    /// use chrono::NaiveDateTime;
    /// let seconds = env.block.time.seconds();
    /// let nsecs = env.block.time.subsec_nanos();
    /// let dt = NaiveDateTime::from_timestamp(seconds as i64, nsecs as u32);
    /// ```
    ///
    /// Creating a simple millisecond-precision timestamp (as used in JavaScript):
    ///
    /// ```
    /// # use cosmwasm_std::{Addr, Binary, BlockInfo, ContractInfo, Env, MessageInfo, Timestamp, TransactionInfo};
    /// # let env = Env {
    /// #     block: BlockInfo {
    /// #         height: 12_345,
    /// #         time: Timestamp::from_nanos(1_571_797_419_879_305_533),
    /// #         chain_id: "cosmos-testnet-14002".to_string(),
    /// #     },
    /// #     transaction: Some(TransactionInfo::new(3, Binary::new(vec![0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8]))),
    /// #     contract: ContractInfo {
    /// #         address: Addr::unchecked("contract"),
    /// #     },
    /// # };
    /// let millis = env.block.time.nanos() / 1_000_000;
    /// ```
    pub time: Timestamp,
    pub chain_id: String,
}

/// Additional information from [MsgInstantiateContract] and [MsgExecuteContract], which is passed
/// along with the contract execution message into the `instantiate` and `execute` entry points.
///
/// It contains the essential info for authorization - identity of the call, and payment.
///
/// [MsgInstantiateContract]: https://github.com/CosmWasm/wasmd/blob/v0.15.0/x/wasm/internal/types/tx.proto#L47-L61
/// [MsgExecuteContract]: https://github.com/CosmWasm/wasmd/blob/v0.15.0/x/wasm/internal/types/tx.proto#L68-L78
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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
    pub sender: Addr,
    /// The funds that are sent to the contract as part of `MsgInstantiateContract`
    /// or `MsgExecuteContract`. The transfer is processed in bank before the contract
    /// is executed such that the new balance is visible during contract execution.
    pub funds: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ContractInfo {
    pub address: Addr,
}

/// The structure contains additional information related to the
/// contract's migration procedure - the sender address and
/// the contract's migrate version currently stored on the blockchain.
/// The `old_migrate_version` is optional, since there is no guarantee
/// that the currently stored contract's binary contains that information.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateInfo {
    /// Address of the sender.
    ///
    /// This is the `sender` field from [`MsgMigrateContract`](https://github.com/CosmWasm/wasmd/blob/v0.53.0/proto/cosmwasm/wasm/v1/tx.proto#L217-L233).
    pub sender: Addr,
    /// Migrate version of the previous contract. It's optional, since
    /// adding the version number to the binary is not a mandatory feature.
    pub old_migrate_version: Option<u64>,
}
