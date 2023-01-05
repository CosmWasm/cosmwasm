use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Binary;
#[cfg(feature = "cosmwasm_1_2")]
use crate::HexBinary;

use super::query_response::QueryResponseType;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WasmQuery {
    /// this queries the public API of another contract at a known address (with known ABI)
    /// Return value is whatever the contract returns (caller should know), wrapped in a
    /// ContractResult that is JSON encoded.
    Smart {
        contract_addr: String,
        /// msg is the json-encoded QueryMsg struct
        msg: Binary,
    },
    /// this queries the raw kv-store of the contract.
    /// returns the raw, unparsed data stored at that key, which may be an empty vector if not present
    Raw {
        contract_addr: String,
        /// Key is the raw key used in the contracts Storage
        key: Binary,
    },
    /// Returns a [`ContractInfoResponse`] with metadata on the contract from the runtime
    ContractInfo { contract_addr: String },
    /// Returns a [`CodeInfoResponse`] with metadata of the code
    #[cfg(feature = "cosmwasm_1_2")]
    CodeInfo { code_id: u64 },
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, JsonSchema)]
pub struct ContractInfoResponse {
    pub code_id: u64,
    /// address that instantiated this contract
    pub creator: String,
    /// admin who can run migrations (if any)
    pub admin: Option<String>,
    /// if set, the contract is pinned to the cache, and thus uses less gas when called
    pub pinned: bool,
    /// set if this contract has bound an IBC port
    pub ibc_port: Option<String>,
}

impl QueryResponseType for ContractInfoResponse {}

impl ContractInfoResponse {
    /// Constructor for testing frameworks such as cw-multi-test.
    /// This is required because query response types should be #[non_exhaustive].
    /// As a contract developer you should not need this constructor since
    /// query responses are constructed for you via deserialization.
    #[doc(hidden)]
    #[deprecated(
        note = "Use ContractInfoResponse::default() and mutate the fields you want to set."
    )]
    pub fn new(code_id: u64, creator: impl Into<String>) -> Self {
        ContractInfoResponse {
            code_id,
            creator: creator.into(),
            ..Default::default()
        }
    }
}

/// The essential data from wasmd's [CodeInfo]/[CodeInfoResponse].
///
/// `code_hash`/`data_hash` was renamed to `checksum` to follow the CosmWasm
/// convention and naming in `instantiate2_address`.
///
/// [CodeInfo]: https://github.com/CosmWasm/wasmd/blob/v0.30.0/proto/cosmwasm/wasm/v1/types.proto#L62-L72
/// [CodeInfoResponse]: https://github.com/CosmWasm/wasmd/blob/v0.30.0/proto/cosmwasm/wasm/v1/query.proto#L184-L199
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, JsonSchema)]
#[cfg(feature = "cosmwasm_1_2")]
pub struct CodeInfoResponse {
    pub code_id: u64,
    /// The address that initially stored the code
    pub creator: String,
    /// The hash of the Wasm blob
    pub checksum: HexBinary,
}

#[cfg(feature = "cosmwasm_1_2")]
impl QueryResponseType for CodeInfoResponse {}
