use derivative::Derivative;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::binary::Binary;
use crate::coin::Coin;
use crate::errors::StdResult;
#[cfg(feature = "stargate")]
use crate::ibc::IbcMsg;
use crate::serde::to_binary;

use super::Empty;

/// Like CustomQuery for better type clarity.
/// Also makes it shorter to use as a trait bound.
pub trait CustomMsg: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema {}

impl CustomMsg for Empty {}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
// See https://github.com/serde-rs/serde/issues/1296 why we cannot add De-Serialize trait bounds to T
pub enum CosmosMsg<T = Empty> {
    Bank(BankMsg),
    // by default we use RawMsg, but a contract can override that
    // to call into more app-specific code (whatever they define)
    Custom(T),
    #[cfg(feature = "staking")]
    Staking(StakingMsg),
    #[cfg(feature = "staking")]
    Distribution(DistributionMsg),
    /// A Stargate message encoded the same way as a protobuf [Any](https://github.com/protocolbuffers/protobuf/blob/master/src/google/protobuf/any.proto).
    /// This is the same structure as messages in `TxBody` from [ADR-020](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-020-protobuf-transaction-encoding.md)
    #[cfg(feature = "stargate")]
    Stargate {
        /// this is the fully qualified msg path used for routing,
        /// e.g. /cosmos.bank.v1beta1.MsgSend
        /// NOTE: the type_url can be changed after a chain upgrade
        type_url: String,
        value: Binary,
    },
    #[cfg(feature = "stargate")]
    Ibc(IbcMsg),
    Wasm(WasmMsg),
    #[cfg(feature = "stargate")]
    Gov(GovMsg),
    FinalizeTx(Empty),
}

impl<T> CosmosMsg<T> {
    pub fn finalize_tx() -> Self {
        CosmosMsg::FinalizeTx(Empty {})
    }
}

/// The message types of the bank module.
///
/// See https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/bank/v1beta1/tx.proto
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BankMsg {
    /// Sends native tokens from the contract to the given address.
    ///
    /// This is translated to a [MsgSend](https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/bank/v1beta1/tx.proto#L19-L28).
    /// `from_address` is automatically filled with the current contract's address.
    Send {
        to_address: String,
        amount: Vec<Coin>,
    },
    /// This will burn the given coins from the contract's account.
    /// There is no Cosmos SDK message that performs this, but it can be done by calling the bank keeper.
    /// Important if a contract controls significant token supply that must be retired.
    Burn { amount: Vec<Coin> },
}

/// The message types of the staking module.
///
/// See https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/staking/v1beta1/tx.proto
#[cfg(feature = "staking")]
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StakingMsg {
    /// This is translated to a [MsgDelegate](https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/staking/v1beta1/tx.proto#L81-L90).
    /// `delegator_address` is automatically filled with the current contract's address.
    Delegate { validator: String, amount: Coin },
    /// This is translated to a [MsgUndelegate](https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/staking/v1beta1/tx.proto#L112-L121).
    /// `delegator_address` is automatically filled with the current contract's address.
    Undelegate { validator: String, amount: Coin },
    /// This is translated to a [MsgBeginRedelegate](https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/staking/v1beta1/tx.proto#L95-L105).
    /// `delegator_address` is automatically filled with the current contract's address.
    Redelegate {
        src_validator: String,
        dst_validator: String,
        amount: Coin,
    },
}

/// The message types of the distribution module.
///
/// See https://github.com/cosmos/cosmos-sdk/blob/v0.42.4/proto/cosmos/distribution/v1beta1/tx.proto
#[cfg(feature = "staking")]
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DistributionMsg {
    /// This is translated to a [MsgSetWithdrawAddress](https://github.com/cosmos/cosmos-sdk/blob/v0.42.4/proto/cosmos/distribution/v1beta1/tx.proto#L29-L37).
    /// `delegator_address` is automatically filled with the current contract's address.
    SetWithdrawAddress {
        /// The `withdraw_address`
        address: String,
    },
    /// This is translated to a [[MsgWithdrawDelegatorReward](https://github.com/cosmos/cosmos-sdk/blob/v0.42.4/proto/cosmos/distribution/v1beta1/tx.proto#L42-L50).
    /// `delegator_address` is automatically filled with the current contract's address.
    WithdrawDelegatorReward {
        /// The `validator_address`
        validator: String,
    },
}

fn binary_to_string(data: &Binary, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
    match std::str::from_utf8(data.as_slice()) {
        Ok(s) => fmt.write_str(s),
        Err(_) => write!(fmt, "{:?}", data),
    }
}

/// The message types of the wasm module.
///
/// See https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Derivative, PartialEq, Eq, JsonSchema)]
#[derivative(Debug)]
#[serde(rename_all = "snake_case")]
pub enum WasmMsg {
    /// Dispatches a call to another contract at a known address (with known ABI).
    ///
    /// This is translated to a [MsgExecuteContract](https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto#L68-L78).
    /// `sender` is automatically filled with the current contract's address.
    Execute {
        contract_addr: String,
        /// code_hash is the hex encoded hash of the code. This is used by Secret Network to harden against replaying the contract
        /// It is used to bind the request to a destination contract in a stronger way than just the contract address which can be faked
        code_hash: String,
        /// msg is the json-encoded ExecuteMsg struct (as raw Binary)
        #[derivative(Debug(format_with = "binary_to_string"))]
        msg: Binary,
        #[serde(rename = "send")]
        funds: Vec<Coin>,
    },
    /// Instantiates a new contracts from previously uploaded Wasm code.
    ///
    /// This is translated to a [MsgInstantiateContract](https://github.com/CosmWasm/wasmd/blob/v0.16.0-alpha1/x/wasm/internal/types/tx.proto#L47-L61).
    /// `sender` is automatically filled with the current contract's address.
    Instantiate {
        admin: Option<String>,
        code_id: u64,
        /// code_hash is the hex encoded hash of the code. This is used by Secret Network to harden against replaying the contract
        /// It is used to bind the request to a destination contract in a stronger way than just the contract address which can be faked
        code_hash: String,
        /// msg is the JSON-encoded InstantiateMsg struct (as raw Binary)
        #[derivative(Debug(format_with = "binary_to_string"))]
        msg: Binary,
        #[serde(rename = "send")]
        funds: Vec<Coin>,
        /// A human-readbale label for the contract, must be unique across all contracts
        label: String,
    },
    /// Migrates a given contracts to use new wasm code. Passes a MigrateMsg to allow us to
    /// customize behavior.
    ///
    /// Only the contract admin (as defined in wasmd), if any, is able to make this call.
    ///
    /// This is translated to a [MsgMigrateContract](https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto#L86-L96).
    /// `sender` is automatically filled with the current contract's address.
    Migrate {
        contract_addr: String,
        /// code_hash is the hex encoded hash of the **new** code. This is used by Secret Network to harden against replaying the contract
        /// It is used to bind the request to a destination contract in a stronger way than just the contract address which can be faked
        code_hash: String,
        /// the code_id of the **new** logic to place in the given contract
        code_id: u64,
        /// msg is the json-encoded MigrateMsg struct that will be passed to the new code
        #[derivative(Debug(format_with = "binary_to_string"))]
        msg: Binary,
    },
    /// Sets a new admin (for migrate) on the given contract.
    /// Fails if this contract is not currently admin of the target contract.
    UpdateAdmin {
        contract_addr: String,
        admin: String,
    },
    /// Clears the admin on the given contract, so no more migration possible.
    /// Fails if this contract is not currently admin of the target contract.
    ClearAdmin { contract_addr: String },
}

#[cfg(feature = "stargate")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovMsg {
    /// This maps directly to [MsgVote](https://github.com/cosmos/cosmos-sdk/blob/v0.42.5/proto/cosmos/gov/v1beta1/tx.proto#L46-L56) in the Cosmos SDK with voter set to the contract address.
    Vote { proposal_id: u64, vote: VoteOption },
}

#[cfg(feature = "stargate")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VoteOption {
    Yes,
    No,
    Abstain,
    NoWithVeto,
}


/// Shortcut helper as the construction of WasmMsg::Instantiate can be quite verbose in contract code.
///
/// When using this, `admin` is always unset. If you need more flexibility, create the message directly.
pub fn wasm_instantiate(
    code_id: u64,
    code_hash: impl Into<String>,
    msg: &impl Serialize,
    funds: Vec<Coin>,
    label: String,
) -> StdResult<WasmMsg> {
    let payload = to_binary(msg)?;
    Ok(WasmMsg::Instantiate {
        admin: None,
        code_id,
        code_hash: code_hash.into(),
        msg: payload,
        funds,
        label,
    })
}

/// Shortcut helper as the construction of WasmMsg::Instantiate can be quite verbose in contract code
pub fn wasm_execute(
    contract_addr: impl Into<String>,
    code_hash: impl Into<String>,
    msg: &impl Serialize,
    funds: Vec<Coin>,
) -> StdResult<WasmMsg> {
    let payload = to_binary(msg)?;
    Ok(WasmMsg::Execute {
        contract_addr: contract_addr.into(),
        code_hash: code_hash.into(),
        msg: payload,
        funds,
    })
}

impl<T> From<BankMsg> for CosmosMsg<T> {
    fn from(msg: BankMsg) -> Self {
        CosmosMsg::Bank(msg)
    }
}

#[cfg(feature = "staking")]
impl<T> From<StakingMsg> for CosmosMsg<T> {
    fn from(msg: StakingMsg) -> Self {
        CosmosMsg::Staking(msg)
    }
}

#[cfg(feature = "staking")]
impl<T> From<DistributionMsg> for CosmosMsg<T> {
    fn from(msg: DistributionMsg) -> Self {
        CosmosMsg::Distribution(msg)
    }
}

impl<T> From<WasmMsg> for CosmosMsg<T> {
    fn from(msg: WasmMsg) -> Self {
        CosmosMsg::Wasm(msg)
    }
}

#[cfg(feature = "stargate")]
impl<T> From<IbcMsg> for CosmosMsg<T> {
    fn from(msg: IbcMsg) -> Self {
        CosmosMsg::Ibc(msg)
    }
}

#[cfg(feature = "stargate")]
impl<T> From<GovMsg> for CosmosMsg<T> {
    fn from(msg: GovMsg) -> Self {
        CosmosMsg::Gov(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coin, coins};

    #[test]
    fn from_bank_msg_works() {
        let to_address = String::from("you");
        let amount = coins(1015, "earth");
        let bank = BankMsg::Send { to_address, amount };
        let msg: CosmosMsg = bank.clone().into();
        match msg {
            CosmosMsg::Bank(msg) => assert_eq!(bank, msg),
            _ => panic!("must encode in Bank variant"),
        }
    }

    #[cosmwasm_schema::cw_serde]
    enum ExecuteMsg {
        Mint { coin: Coin },
    }

    #[test]
    fn wasm_msg_debug_decodes_binary_string_when_possible() {
        let msg = WasmMsg::Execute {
            contract_addr: "joe".to_string(),
            code_hash: "aaaa".to_string(),
            msg: to_binary(&ExecuteMsg::Mint {
                coin: coin(10, "BTC"),
            })
            .unwrap(),
            funds: vec![],
        };

        assert_eq!(
            format!("{:?}", msg),
            "Execute { contract_addr: \"joe\", code_hash: \"aaaa\", msg: {\"mint\":{\"coin\":{\"denom\":\"BTC\",\"amount\":\"10\"}}}, funds: [] }"
        );
    }

    #[test]
    fn wasm_msg_debug_dumps_binary_when_not_utf8() {
        let msg = WasmMsg::Execute {
            contract_addr: "joe".to_string(),
            code_hash: "aaaa".to_string(),
            msg: Binary::from([0, 159, 146, 150]),
            funds: vec![],
        };

        assert_eq!(
            format!("{:?}", msg),
            "Execute { contract_addr: \"joe\", code_hash: \"aaaa\", msg: Binary(009f9296), funds: [] }"
        );
    }
}
