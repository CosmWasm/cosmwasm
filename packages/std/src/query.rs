use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::api::ApiResult;
use crate::coins::Coin;
use crate::encoding::Binary;
use crate::types::HumanAddr;

pub type QueryResponse = Binary;

pub type QueryResult = ApiResult<QueryResponse>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryRequest {
    Bank(BankQuery),
    Wasm(WasmQuery),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BankQuery {
    // This calls into the native bank module for one denomination
    // Return value is BalanceResponse
    Balance { address: HumanAddr, denom: String },
    // This calls into the native bank module for all denominations.
    // Note that this may be much more expensive than Balance and should be avoided if possible.
    // Return value is AllBalanceResponse.
    AllBalances { address: HumanAddr },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WasmQuery {
    // this queries the public API of another contract at a known address (with known ABI)
    // msg is the json-encoded QueryMsg struct
    // return value is whatever the contract returns (caller should know)
    Smart {
        contract_addr: HumanAddr,
        msg: Binary, // we pass this in as Vec<u8> to the contract, so allow any binary encoding (later, limit to rawjson?)
    },
    // this queries the raw kv-store of the contract.
    // returns the raw, unparsed data stored at that key (or `Ok(Err(ApiError:NotFound{}))` if missing)
    Raw {
        contract_addr: HumanAddr,
        key: Binary, // we pass this in as Vec<u8> to the contract, so allow any binary encoding (later, limit to rawjson?)
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BalanceResponse {
    // Always returns a Coin with the requested denom.
    // This may be of 0 amount if no such funds.
    pub amount: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AllBalanceResponse {
    // Returns all non-zero coins held by this account.
    pub amount: Vec<Coin>,
}
