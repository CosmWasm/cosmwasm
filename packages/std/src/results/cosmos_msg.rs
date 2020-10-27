use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::addresses::HumanAddr;
use crate::binary::Binary;
use crate::coins::Coin;
use crate::types::Empty;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
// See https://github.com/serde-rs/serde/issues/1296 why we cannot add De-Serialize trait bounds to T
pub enum CosmosMsg<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    Bank(BankMsg),
    // by default we use RawMsg, but a contract can override that
    // to call into more app-specific code (whatever they define)
    Custom(T),
    Staking(StakingMsg),
    Wasm(WasmMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BankMsg {
    // this moves tokens in the underlying sdk
    Send {
        from_address: HumanAddr,
        to_address: HumanAddr,
        amount: Vec<Coin>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StakingMsg {
    Delegate {
        // delegator is automatically set to address of the calling contract
        validator: HumanAddr,
        amount: Coin,
    },
    Undelegate {
        // delegator is automatically set to address of the calling contract
        validator: HumanAddr,
        amount: Coin,
    },
    Withdraw {
        // delegator is automatically set to address of the calling contract
        validator: HumanAddr,
        /// this is the "withdraw address", the one that should receive the rewards
        /// if None, then use delegator address
        recipient: Option<HumanAddr>,
    },
    Redelegate {
        // delegator is automatically set to address of the calling contract
        src_validator: HumanAddr,
        dst_validator: HumanAddr,
        amount: Coin,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WasmMsg {
    /// this dispatches a call to another contract at a known address (with known ABI)
    Execute {
        contract_addr: HumanAddr,
        /// msg is the json-encoded HandleMsg struct (as raw Binary)
        msg: Binary,
        send: Vec<Coin>,
    },
    /// this instantiates a new contracts from previously uploaded wasm code
    Instantiate {
        code_id: u64,
        /// msg is the json-encoded InitMsg struct (as raw Binary)
        msg: Binary,
        send: Vec<Coin>,
        /// optional human-readbale label for the contract
        label: Option<String>,
    },
}

impl<T: Clone + fmt::Debug + PartialEq + JsonSchema> From<BankMsg> for CosmosMsg<T> {
    fn from(msg: BankMsg) -> Self {
        CosmosMsg::Bank(msg)
    }
}

#[cfg(feature = "staking")]
impl<T: Clone + fmt::Debug + PartialEq + JsonSchema> From<StakingMsg> for CosmosMsg<T> {
    fn from(msg: StakingMsg) -> Self {
        CosmosMsg::Staking(msg)
    }
}

impl<T: Clone + fmt::Debug + PartialEq + JsonSchema> From<WasmMsg> for CosmosMsg<T> {
    fn from(msg: WasmMsg) -> Self {
        CosmosMsg::Wasm(msg)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::coins;

    #[test]
    fn from_bank_msg_works() {
        let from_address = HumanAddr::from("me");
        let to_address = HumanAddr::from("you");
        let amount = coins(1015, "earth");
        let bank = BankMsg::Send {
            from_address,
            to_address,
            amount,
        };
        let msg: CosmosMsg = bank.clone().into();
        match msg {
            CosmosMsg::Bank(msg) => assert_eq!(bank, msg),
            _ => panic!("must encode in Bank variant"),
        }
    }
}
