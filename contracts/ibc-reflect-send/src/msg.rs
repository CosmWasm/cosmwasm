use cosmwasm_std::{Coin, CosmosMsg, Empty, HumanAddr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::AccountData;

/// This needs no info. Owner of the contract is whoever signed the InstantiateMsg.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Changes the admin
    UpdateAdmin {
        admin: HumanAddr,
    },
    SendMsgs {
        channel_id: String,
        // Note: we don't handle custom messages on remote chains
        msgs: Vec<CosmosMsg<Empty>>,
    },
    CheckRemoteBalance {
        channel_id: String,
    },
    /// If you sent funds to this contract, it will attempt to ibc transfer them
    /// to the account on the remote side of this channel.
    /// If we don't have the address yet, this fails.
    SendFunds {
        /// The channel id we use above to talk with the reflect contract
        reflect_channel_id: String,
        /// The channel to use for ibctransfer. This is bound to a different
        /// port and handled by a different module.
        /// It should connect to the same chain as the reflect_channel_id does
        transfer_channel_id: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Returns current admin
    Admin {},
    // Shows all open accounts (incl. remote info)
    ListAccounts {},
    // Get account for one channel
    Account { channel_id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AdminResponse {
    pub admin: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ListAccountsResponse {
    pub accounts: Vec<AccountInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AccountInfo {
    pub channel_id: String,
    /// last block balance was updated (0 is never)
    pub last_update_time: u64,
    /// in normal cases, it should be set, but there is a delay between binding
    /// the channel and making a query and in that time it is empty
    pub remote_addr: Option<HumanAddr>,
    pub remote_balance: Vec<Coin>,
}

impl AccountInfo {
    pub fn convert(channel_id: String, input: AccountData) -> Self {
        AccountInfo {
            channel_id,
            last_update_time: input.last_update_time,
            remote_addr: input.remote_addr,
            remote_balance: input.remote_balance,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AccountResponse {
    /// last block balance was updated (0 is never)
    pub last_update_time: u64,
    /// in normal cases, it should be set, but there is a delay between binding
    /// the channel and making a query and in that time it is empty
    pub remote_addr: Option<HumanAddr>,
    pub remote_balance: Vec<Coin>,
}

impl From<AccountData> for AccountResponse {
    fn from(input: AccountData) -> Self {
        AccountResponse {
            last_update_time: input.last_update_time,
            remote_addr: input.remote_addr,
            remote_balance: input.remote_balance,
        }
    }
}
