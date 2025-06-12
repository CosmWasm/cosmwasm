#![allow(deprecated)]

use core::fmt;
use derive_more::Debug;
use dyn_partial_eq::{dyn_partial_eq, DynPartialEq};
use erased_serde::serialize_trait_object;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

use crate::coin::Coin;
#[cfg(feature = "stargate")]
use crate::ibc::IbcMsg;
#[cfg(feature = "ibc2")]
use crate::ibc2::Ibc2Msg;
use crate::prelude::*;
#[cfg(all(feature = "stargate", feature = "cosmwasm_1_2"))]
use crate::Decimal;
use crate::StdResult;
use crate::{to_json_binary, Binary};

use super::Empty;

/// A trait for custom message types which are embedded in `CosmosMsg::Custom(..)`.
/// Those are messages that the contract and the chain need
/// to agree on in advance as the chain must be able to deserialize and execute them.
///
/// Custom messages are always JSON-encoded when sent from the contract to the environment.
///
/// This trait is similar to [`CustomQuery`](crate::CustomQuery) for better type clarity and
/// makes it shorter to use as a trait bound. It does not require fields or functions to be implemented.
///
/// An alternative approach is using [`CosmosMsg::Any`][crate::CosmosMsg#variant.Any]
/// which provides more flexibility but offers less type-safety.
///
/// ## Examples
///
/// Some real-world examples of such custom message types are
/// [TgradeMsg](https://github.com/confio/poe-contracts/blob/v0.17.1/packages/bindings/src/msg.rs#L13),
/// [ArchwayMsg](https://github.com/archway-network/arch3.rs/blob/bindings/v0.2.1/packages/bindings/src/msg.rs#L22) or
/// [NeutronMsg](https://github.com/neutron-org/neutron-sdk/blob/v0.11.0/packages/neutron-sdk/src/bindings/msg.rs#L33).
///
/// ```
/// use cosmwasm_schema::cw_serde;
/// use cosmwasm_std::CustomQuery;
///
/// #[cw_serde]
/// pub enum MyMsg {
///    // ...
/// }
///
/// impl CustomQuery for MyMsg {}
/// ```
#[dyn_partial_eq]
pub trait CustomMsg: erased_serde::Serialize + fmt::Debug {}

serialize_trait_object!(CustomMsg);

#[derive(Serialize, Clone, Debug, DynPartialEq)]
pub struct CustomMsgContainer(pub Rc<dyn CustomMsg>);

impl PartialEq for CustomMsgContainer {
    fn eq(&self, other: &Self) -> bool {
        self.0.box_eq(other.0.as_any())
    }
}

impl<C: CustomMsg + 'static> From<C> for CustomMsgContainer {
    fn from(msg: C) -> Self {
        CustomMsgContainer(Rc::new(msg))
    }
}

impl CustomMsg for Empty {}

impl<'de> Deserialize<'de> for CustomMsgContainer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize into a Box<dyn Test> by deserializing into a concrete type
        // that implements the Test trait.
        let concrete: UnknownCustomMsg = Deserialize::deserialize(deserializer)?;
        Ok(Self(Rc::new(concrete)))
    }
}

/// Helper type that allows us to deserialize any custom message without knowing its type.
/// We need that because we need to deserialize [`CosmosMsg`] in `cosmwasm-vm`
#[derive(Serialize, Deserialize, Clone, Debug, DynPartialEq, PartialEq, Eq, JsonSchema)]
struct UnknownCustomMsg(serde_json::Value);

impl CustomMsg for UnknownCustomMsg {}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
// See https://github.com/serde-rs/serde/issues/1296 why we cannot add De-Serialize trait bounds to T
pub enum CosmosMsg {
    Bank(BankMsg),
    // by default we use RawMsg, but a contract can override that
    // to call into more app-specific code (whatever they define)
    Custom(CustomMsgContainer),
    #[cfg(feature = "staking")]
    Staking(StakingMsg),
    #[cfg(feature = "staking")]
    Distribution(DistributionMsg),
    /// This is the same structure as messages in `TxBody` from [ADR-020](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-020-protobuf-transaction-encoding.md)
    #[cfg(feature = "stargate")]
    #[deprecated = "Use `CosmosMsg::Any` instead (if you only target CosmWasm 2+ chains)"]
    Stargate {
        type_url: String,
        value: Binary,
    },
    /// `CosmosMsg::Any` replaces the "stargate message" – a message wrapped
    /// in a [protobuf Any](https://protobuf.dev/programming-guides/proto3/#any)
    /// that is supported by the chain. It behaves the same as
    /// `CosmosMsg::Stargate` but has a better name and slightly improved syntax.
    ///
    /// This is feature-gated at compile time with `cosmwasm_2_0` because
    /// a chain running CosmWasm < 2.0 cannot process this.
    #[cfg(feature = "cosmwasm_2_0")]
    Any(AnyMsg),
    #[cfg(feature = "stargate")]
    Ibc(IbcMsg),
    Wasm(WasmMsg),
    #[cfg(feature = "stargate")]
    Gov(GovMsg),
    #[cfg(feature = "ibc2")]
    Ibc2(Ibc2Msg),
}

// TODO: make this easier to maintain
impl PartialEq for CosmosMsg {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bank(l0), Self::Bank(r0)) => l0 == r0,
            (Self::Custom(l0), Self::Custom(r0)) => l0.box_eq(r0.as_any()),
            #[cfg(feature = "staking")]
            (Self::Staking(l0), Self::Staking(r0)) => l0 == r0,
            #[cfg(feature = "staking")]
            (Self::Distribution(l0), Self::Distribution(r0)) => l0 == r0,
            #[cfg(feature = "stargate")]
            (
                Self::Stargate {
                    type_url: l_type_url,
                    value: l_value,
                },
                Self::Stargate {
                    type_url: r_type_url,
                    value: r_value,
                },
            ) => l_type_url == r_type_url && l_value == r_value,
            #[cfg(feature = "cosmwasm_2_0")]
            (Self::Any(l0), Self::Any(r0)) => l0 == r0,
            #[cfg(feature = "stargate")]
            (Self::Ibc(l0), Self::Ibc(r0)) => l0 == r0,
            (Self::Wasm(l0), Self::Wasm(r0)) => l0 == r0,
            #[cfg(feature = "stargate")]
            (Self::Gov(l0), Self::Gov(r0)) => l0 == r0,
            #[cfg(feature = "ibc2")]
            (Self::Ibc2(l0), Self::Ibc2(r0)) => l0 == r0,
            _ => false,
        }
    }
}
impl Eq for CosmosMsg {}

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
    /// This is translated to a [MsgWithdrawDelegatorReward](https://github.com/cosmos/cosmos-sdk/blob/v0.42.4/proto/cosmos/distribution/v1beta1/tx.proto#L42-L50).
    /// `delegator_address` is automatically filled with the current contract's address.
    WithdrawDelegatorReward {
        /// The `validator_address`
        validator: String,
    },
    /// This is translated to a [[MsgFundCommunityPool](https://github.com/cosmos/cosmos-sdk/blob/v0.42.4/proto/cosmos/distribution/v1beta1/tx.proto#LL69C1-L76C2).
    /// `depositor` is automatically filled with the current contract's address.
    #[cfg(feature = "cosmwasm_1_3")]
    FundCommunityPool {
        /// The amount to spend
        amount: Vec<Coin>,
    },
}

/// A message encoded the same way as a protobuf [Any](https://github.com/protocolbuffers/protobuf/blob/master/src/google/protobuf/any.proto).
/// This is the same structure as messages in `TxBody` from [ADR-020](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-020-protobuf-transaction-encoding.md)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AnyMsg {
    pub type_url: String,
    pub value: Binary,
}

#[allow(dead_code)]
struct BinaryToStringEncoder<'a>(&'a Binary);

impl fmt::Display for BinaryToStringEncoder<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match core::str::from_utf8(self.0.as_slice()) {
            Ok(s) => f.write_str(s),
            Err(_) => fmt::Debug::fmt(self.0, f),
        }
    }
}

/// The message types of the wasm module.
///
/// See https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WasmMsg {
    /// Dispatches a call to another contract at a known address (with known ABI).
    ///
    /// This is translated to a [MsgExecuteContract](https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto#L68-L78).
    /// `sender` is automatically filled with the current contract's address.
    Execute {
        contract_addr: String,
        /// msg is the json-encoded ExecuteMsg struct (as raw Binary)
        #[debug("{}", BinaryToStringEncoder(msg))]
        msg: Binary,
        funds: Vec<Coin>,
    },
    /// Instantiates a new contracts from previously uploaded Wasm code.
    ///
    /// The contract address is non-predictable. But it is guaranteed that
    /// when emitting the same Instantiate message multiple times,
    /// multiple instances on different addresses will be generated. See also
    /// Instantiate2.
    ///
    /// This is translated to a [MsgInstantiateContract](https://github.com/CosmWasm/wasmd/blob/v0.29.2/proto/cosmwasm/wasm/v1/tx.proto#L53-L71).
    /// `sender` is automatically filled with the current contract's address.
    Instantiate {
        admin: Option<String>,
        code_id: u64,
        /// msg is the JSON-encoded InstantiateMsg struct (as raw Binary)
        #[debug("{}", BinaryToStringEncoder(msg))]
        msg: Binary,
        funds: Vec<Coin>,
        /// A human-readable label for the contract.
        ///
        /// Valid values should:
        /// - not be empty
        /// - not be bigger than 128 bytes (or some chain-specific limit)
        /// - not start / end with whitespace
        label: String,
    },
    /// Instantiates a new contracts from previously uploaded Wasm code
    /// using a predictable address derivation algorithm implemented in
    /// [`cosmwasm_std::instantiate2_address`].
    ///
    /// This is translated to a [MsgInstantiateContract2](https://github.com/CosmWasm/wasmd/blob/v0.29.2/proto/cosmwasm/wasm/v1/tx.proto#L73-L96).
    /// `sender` is automatically filled with the current contract's address.
    /// `fix_msg` is automatically set to false.
    #[cfg(feature = "cosmwasm_1_2")]
    Instantiate2 {
        admin: Option<String>,
        code_id: u64,
        /// A human-readable label for the contract.
        ///
        /// Valid values should:
        /// - not be empty
        /// - not be bigger than 128 bytes (or some chain-specific limit)
        /// - not start / end with whitespace
        label: String,
        /// msg is the JSON-encoded InstantiateMsg struct (as raw Binary)
        #[debug("{}", BinaryToStringEncoder(msg))]
        msg: Binary,
        funds: Vec<Coin>,
        salt: Binary,
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
        /// the code_id of the new logic to place in the given contract
        new_code_id: u64,
        /// msg is the json-encoded MigrateMsg struct that will be passed to the new code
        #[debug("{}", BinaryToStringEncoder(msg))]
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

/// This message type allows the contract interact with the [x/gov] module in order
/// to cast votes.
///
/// [x/gov]: https://github.com/cosmos/cosmos-sdk/tree/v0.45.12/x/gov
///
/// ## Examples
///
/// Cast a simple vote:
///
/// ```
/// # use cosmwasm_std::{
/// #     HexBinary,
/// #     Storage, Api, Querier, DepsMut, Deps, entry_point, Env, StdError, MessageInfo,
/// #     Response, QueryResponse,
/// # };
/// # type ExecuteMsg = ();
/// use cosmwasm_std::{GovMsg, VoteOption};
///
/// #[entry_point]
/// pub fn execute(
///     deps: DepsMut,
///     env: Env,
///     info: MessageInfo,
///     msg: ExecuteMsg,
/// ) -> Result<Response, StdError> {
///     // ...
///     Ok(Response::new().add_message(GovMsg::Vote {
///         proposal_id: 4,
///         option: VoteOption::Yes,
///     }))
/// }
/// ```
///
/// Cast a weighted vote:
///
/// ```
/// # use cosmwasm_std::{
/// #     HexBinary,
/// #     Storage, Api, Querier, DepsMut, Deps, entry_point, Env, StdError, MessageInfo,
/// #     Response, QueryResponse,
/// # };
/// # type ExecuteMsg = ();
/// # #[cfg(feature = "cosmwasm_1_2")]
/// use cosmwasm_std::{Decimal, GovMsg, VoteOption, WeightedVoteOption};
///
/// # #[cfg(feature = "cosmwasm_1_2")]
/// #[entry_point]
/// pub fn execute(
///     deps: DepsMut,
///     env: Env,
///     info: MessageInfo,
///     msg: ExecuteMsg,
/// ) -> Result<Response, StdError> {
///     // ...
///     Ok(Response::new().add_message(GovMsg::VoteWeighted {
///         proposal_id: 4,
///         options: vec![
///             WeightedVoteOption {
///                 option: VoteOption::Yes,
///                 weight: Decimal::percent(65),
///             },
///             WeightedVoteOption {
///                 option: VoteOption::Abstain,
///                 weight: Decimal::percent(35),
///             },
///         ],
///     }))
/// }
/// ```
#[cfg(feature = "stargate")]
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovMsg {
    /// This maps directly to [MsgVote](https://github.com/cosmos/cosmos-sdk/blob/v0.42.5/proto/cosmos/gov/v1beta1/tx.proto#L46-L56) in the Cosmos SDK with voter set to the contract address.
    Vote {
        proposal_id: u64,
        /// The vote option.
        ///
        /// This used to be called "vote", but was changed for consistency with Cosmos SDK.
        option: VoteOption,
    },
    /// This maps directly to [MsgVoteWeighted](https://github.com/cosmos/cosmos-sdk/blob/v0.45.8/proto/cosmos/gov/v1beta1/tx.proto#L66-L78) in the Cosmos SDK with voter set to the contract address.
    #[cfg(feature = "cosmwasm_1_2")]
    VoteWeighted {
        proposal_id: u64,
        options: Vec<WeightedVoteOption>,
    },
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

#[cfg(all(feature = "stargate", feature = "cosmwasm_1_2"))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct WeightedVoteOption {
    pub option: VoteOption,
    pub weight: Decimal,
}

/// Shortcut helper as the construction of WasmMsg::Instantiate can be quite verbose in contract code.
///
/// When using this, `admin` is always unset. If you need more flexibility, create the message directly.
pub fn wasm_instantiate(
    code_id: u64,
    msg: &impl Serialize,
    funds: Vec<Coin>,
    label: String,
) -> StdResult<WasmMsg> {
    let payload = to_json_binary(msg)?;
    Ok(WasmMsg::Instantiate {
        admin: None,
        code_id,
        msg: payload,
        funds,
        label,
    })
}

/// Shortcut helper as the construction of WasmMsg::Execute can be quite verbose in contract code
pub fn wasm_execute(
    contract_addr: impl Into<String>,
    msg: &impl Serialize,
    funds: Vec<Coin>,
) -> StdResult<WasmMsg> {
    let payload = to_json_binary(msg)?;
    Ok(WasmMsg::Execute {
        contract_addr: contract_addr.into(),
        msg: payload,
        funds,
    })
}

impl From<BankMsg> for CosmosMsg {
    fn from(msg: BankMsg) -> Self {
        CosmosMsg::Bank(msg)
    }
}

#[cfg(feature = "staking")]
impl From<StakingMsg> for CosmosMsg {
    fn from(msg: StakingMsg) -> Self {
        CosmosMsg::Staking(msg)
    }
}

#[cfg(feature = "staking")]
impl From<DistributionMsg> for CosmosMsg {
    fn from(msg: DistributionMsg) -> Self {
        CosmosMsg::Distribution(msg)
    }
}

// By implementing `From<MyType> for cosmwasm_std::AnyMsg`,
// you automatically get a MyType -> CosmosMsg conversion.
#[cfg(feature = "cosmwasm_2_0")]
impl<S: Into<AnyMsg>> From<S> for CosmosMsg {
    fn from(source: S) -> Self {
        CosmosMsg::Any(source.into())
    }
}

impl From<WasmMsg> for CosmosMsg {
    fn from(msg: WasmMsg) -> Self {
        CosmosMsg::Wasm(msg)
    }
}

#[cfg(feature = "stargate")]
impl From<IbcMsg> for CosmosMsg {
    fn from(msg: IbcMsg) -> Self {
        CosmosMsg::Ibc(msg)
    }
}

#[cfg(feature = "stargate")]
impl From<GovMsg> for CosmosMsg {
    fn from(msg: GovMsg) -> Self {
        CosmosMsg::Gov(msg)
    }
}

#[cfg(feature = "ibc2")]
impl From<Ibc2Msg> for CosmosMsg {
    fn from(msg: Ibc2Msg) -> Self {
        CosmosMsg::Ibc2(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coin, coins};
    use fmt::Debug;

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

    #[test]
    #[cfg(feature = "cosmwasm_2_0")]
    fn from_any_msg_works() {
        // should work with AnyMsg
        let any = AnyMsg {
            type_url: "/cosmos.foo.v1beta.MsgBar".to_string(),
            value: Binary::from_base64("5yu/rQ+HrMcxH1zdga7P5hpGMLE=").unwrap(),
        };
        let msg: CosmosMsg = any.clone().into();
        assert!(matches!(msg, CosmosMsg::Any(a) if a == any));

        // should work with Into<AnyMsg>
        struct IntoAny;

        impl From<IntoAny> for AnyMsg {
            fn from(_: IntoAny) -> Self {
                AnyMsg {
                    type_url: "/cosmos.foo.v1beta.MsgBar".to_string(),
                    value: Binary::from_base64("5yu/rQ+HrMcxH1zdga7P5hpGMLE=").unwrap(),
                }
            }
        }

        let msg: CosmosMsg = IntoAny.into();
        assert!(matches!(
            msg,
            CosmosMsg::Any(a) if a == any
        ));
    }

    #[test]
    fn wasm_msg_serializes_to_correct_json() {
        // Instantiate with admin
        let msg = WasmMsg::Instantiate {
            admin: Some("king".to_string()),
            code_id: 7897,
            msg: br#"{"claim":{}}"#.into(),
            funds: vec![],
            label: "my instance".to_string(),
        };
        let json = to_json_binary(&msg).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&json),
            r#"{"instantiate":{"admin":"king","code_id":7897,"msg":"eyJjbGFpbSI6e319","funds":[],"label":"my instance"}}"#,
        );

        // Instantiate without admin
        let msg = WasmMsg::Instantiate {
            admin: None,
            code_id: 7897,
            msg: br#"{"claim":{}}"#.into(),
            funds: vec![],
            label: "my instance".to_string(),
        };
        let json = to_json_binary(&msg).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&json),
            r#"{"instantiate":{"admin":null,"code_id":7897,"msg":"eyJjbGFpbSI6e319","funds":[],"label":"my instance"}}"#,
        );

        // Instantiate with funds
        let msg = WasmMsg::Instantiate {
            admin: None,
            code_id: 7897,
            msg: br#"{"claim":{}}"#.into(),
            funds: vec![coin(321, "stones")],
            label: "my instance".to_string(),
        };
        let json = to_json_binary(&msg).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&json),
            r#"{"instantiate":{"admin":null,"code_id":7897,"msg":"eyJjbGFpbSI6e319","funds":[{"denom":"stones","amount":"321"}],"label":"my instance"}}"#,
        );

        // Instantiate2
        #[cfg(feature = "cosmwasm_1_2")]
        {
            let msg = WasmMsg::Instantiate2 {
                admin: None,
                code_id: 7897,
                label: "my instance".to_string(),
                msg: br#"{"claim":{}}"#.into(),
                funds: vec![coin(321, "stones")],
                salt: Binary::from_base64("UkOVazhiwoo=").unwrap(),
            };
            let json = to_json_binary(&msg).unwrap();
            assert_eq!(
                String::from_utf8_lossy(&json),
                r#"{"instantiate2":{"admin":null,"code_id":7897,"label":"my instance","msg":"eyJjbGFpbSI6e319","funds":[{"denom":"stones","amount":"321"}],"salt":"UkOVazhiwoo="}}"#,
            );
        }
    }

    #[test]
    #[cfg(feature = "cosmwasm_2_0")]
    fn any_msg_serializes_to_correct_json() {
        // Same serialization as CosmosMsg::Stargate (see above), except the top level key
        let msg: CosmosMsg = CosmosMsg::Any(AnyMsg {
            type_url: "/cosmos.foo.v1beta.MsgBar".to_string(),
            value: Binary::from_base64("5yu/rQ+HrMcxH1zdga7P5hpGMLE=").unwrap(),
        });
        let json = crate::to_json_string(&msg).unwrap();
        assert_eq!(
            json,
            r#"{"any":{"type_url":"/cosmos.foo.v1beta.MsgBar","value":"5yu/rQ+HrMcxH1zdga7P5hpGMLE="}}"#,
        );
    }

    #[test]
    #[cfg(all(feature = "cosmwasm_1_3", feature = "staking"))]
    fn msg_distribution_serializes_to_correct_json() {
        // FundCommunityPool
        let fund_coins = vec![coin(200, "feathers"), coin(200, "stones")];
        let fund_msg = DistributionMsg::FundCommunityPool { amount: fund_coins };
        let fund_json = to_json_binary(&fund_msg).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&fund_json),
            r#"{"fund_community_pool":{"amount":[{"denom":"feathers","amount":"200"},{"denom":"stones","amount":"200"}]}}"#,
        );

        // SetWithdrawAddress
        let set_msg = DistributionMsg::SetWithdrawAddress {
            address: String::from("withdrawer"),
        };
        let set_json = to_json_binary(&set_msg).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&set_json),
            r#"{"set_withdraw_address":{"address":"withdrawer"}}"#,
        );

        // WithdrawDelegatorRewards
        let withdraw_msg = DistributionMsg::WithdrawDelegatorReward {
            validator: String::from("fancyoperator"),
        };
        let withdraw_json = to_json_binary(&withdraw_msg).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&withdraw_json),
            r#"{"withdraw_delegator_reward":{"validator":"fancyoperator"}}"#
        );
    }

    #[test]
    fn wasm_msg_debug_decodes_binary_string_when_possible() {
        #[cosmwasm_schema::cw_serde]
        enum ExecuteMsg {
            Mint { coin: Coin },
        }

        let msg = WasmMsg::Execute {
            contract_addr: "joe".to_string(),
            msg: to_json_binary(&ExecuteMsg::Mint {
                coin: coin(10, "BTC"),
            })
            .unwrap(),
            funds: vec![],
        };

        assert_eq!(
            format!("{msg:?}"),
            "Execute { contract_addr: \"joe\", msg: {\"mint\":{\"coin\":{\"denom\":\"BTC\",\"amount\":\"10\"}}}, funds: [] }"
        );
    }

    #[test]
    fn wasm_msg_debug_dumps_binary_when_not_utf8() {
        let msg = WasmMsg::Execute {
            contract_addr: "joe".to_string(),
            msg: Binary::from([0, 159, 146, 150]),
            funds: vec![],
        };

        assert_eq!(
            format!("{msg:?}"),
            "Execute { contract_addr: \"joe\", msg: Binary(009f9296), funds: [] }"
        );
    }

    #[test]
    #[cfg(feature = "stargate")]
    fn gov_msg_serializes_to_correct_json() {
        // Vote
        let msg = GovMsg::Vote {
            proposal_id: 4,
            option: VoteOption::NoWithVeto,
        };
        let json = to_json_binary(&msg).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&json),
            r#"{"vote":{"proposal_id":4,"option":"no_with_veto"}}"#,
        );

        // VoteWeighted
        #[cfg(feature = "cosmwasm_1_2")]
        {
            let msg = GovMsg::VoteWeighted {
                proposal_id: 25,
                options: vec![
                    WeightedVoteOption {
                        weight: Decimal::percent(25),
                        option: VoteOption::Yes,
                    },
                    WeightedVoteOption {
                        weight: Decimal::percent(25),
                        option: VoteOption::No,
                    },
                    WeightedVoteOption {
                        weight: Decimal::percent(50),
                        option: VoteOption::Abstain,
                    },
                ],
            };

            let json = to_json_binary(&msg).unwrap();
            assert_eq!(
                String::from_utf8_lossy(&json),
                r#"{"vote_weighted":{"proposal_id":25,"options":[{"option":"yes","weight":"0.25"},{"option":"no","weight":"0.25"},{"option":"abstain","weight":"0.5"}]}}"#,
            );
        }
    }
}
