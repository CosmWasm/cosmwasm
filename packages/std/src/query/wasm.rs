use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Binary;

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
    /// returns a ContractInfoResponse with metadata on the contract from the runtime
    ContractInfo { contract_addr: String },
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ContractInfoResponse {
    pub code_id: u64,
    /// Checksum was added in CosmWasm XXXX.YYY. Unfortunately we need to keep this optional
    /// because we otherwise had to break the ContractInfoResponse constructor.
    /// See <https://github.com/CosmWasm/cosmwasm/issues/1545>.
    pub checksum: Option<String>,
    /// address that instantiated this contract
    pub creator: String,
    /// admin who can run migrations (if any)
    pub admin: Option<String>,
    /// if set, the contract is pinned to the cache, and thus uses less gas when called
    pub pinned: bool,
    /// set if this contract has bound an IBC port
    pub ibc_port: Option<String>,
}

impl ContractInfoResponse {
    /// Convenience constructor for tests / mocks
    #[doc(hidden)]
    pub fn new(code_id: u64, creator: impl Into<String>) -> Self {
        Self {
            code_id,
            checksum: None,
            creator: creator.into(),
            admin: None,
            pinned: false,
            ibc_port: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::from_slice;

    #[test]
    fn contractinforesponse_deserialize_works() {
        let info: ContractInfoResponse = from_slice(br#"{"code_id":3456,"creator":"tgrade1js7ezrm55fqgxu3p62d9xn6patjku2z7ne5dvg","admin":"tgrade1z363ulwcrxged4z5jswyt5dn5v3lzsemwz9ewj","pinned":false,"ibc_port":"wasm.abcdef"}"#).unwrap();
        assert_eq!(
            info,
            ContractInfoResponse {
                code_id: 3456,
                checksum: None,
                creator: "tgrade1js7ezrm55fqgxu3p62d9xn6patjku2z7ne5dvg".to_string(),
                admin: Some("tgrade1z363ulwcrxged4z5jswyt5dn5v3lzsemwz9ewj".to_string()),
                pinned: false,
                ibc_port: Some("wasm.abcdef".to_string()),
            }
        );

        // JSON now contains extra checksum
        let info: ContractInfoResponse = from_slice(br#"{"code_id":3456,"checksum":"75b6183689b80a229ea27994a7c8cd9c17ddd29e947998f2734abda825eac3c0","creator":"tgrade1js7ezrm55fqgxu3p62d9xn6patjku2z7ne5dvg","admin":"tgrade1z363ulwcrxged4z5jswyt5dn5v3lzsemwz9ewj","pinned":false,"ibc_port":"wasm.abcdef"}"#).unwrap();
        assert_eq!(
            info,
            ContractInfoResponse {
                code_id: 3456,
                checksum: Some(
                    "75b6183689b80a229ea27994a7c8cd9c17ddd29e947998f2734abda825eac3c0".to_string()
                ),
                creator: "tgrade1js7ezrm55fqgxu3p62d9xn6patjku2z7ne5dvg".to_string(),
                admin: Some("tgrade1z363ulwcrxged4z5jswyt5dn5v3lzsemwz9ewj".to_string()),
                pinned: false,
                ibc_port: Some("wasm.abcdef".to_string()),
            }
        );
    }

    #[test]
    fn contractinforesponse_from_cosmwasm_1_0_deserialize_works() {
        // This is the version sipped in 1.0 contracts.
        #[non_exhaustive]
        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
        pub struct ContractInfoResponseCosmWasm1 {
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

        let info: ContractInfoResponseCosmWasm1 = from_slice(br#"{"code_id":3456,"creator":"tgrade1js7ezrm55fqgxu3p62d9xn6patjku2z7ne5dvg","admin":"tgrade1z363ulwcrxged4z5jswyt5dn5v3lzsemwz9ewj","pinned":false,"ibc_port":"wasm.abcdef"}"#).unwrap();
        assert_eq!(
            info,
            ContractInfoResponseCosmWasm1 {
                code_id: 3456,
                creator: "tgrade1js7ezrm55fqgxu3p62d9xn6patjku2z7ne5dvg".to_string(),
                admin: Some("tgrade1z363ulwcrxged4z5jswyt5dn5v3lzsemwz9ewj".to_string()),
                pinned: false,
                ibc_port: Some("wasm.abcdef".to_string()),
            }
        );

        // JSON now contains extra checksum
        let info: ContractInfoResponseCosmWasm1 = from_slice(br#"{"code_id":3456,"checksum":"75b6183689b80a229ea27994a7c8cd9c17ddd29e947998f2734abda825eac3c0","creator":"tgrade1js7ezrm55fqgxu3p62d9xn6patjku2z7ne5dvg","admin":"tgrade1z363ulwcrxged4z5jswyt5dn5v3lzsemwz9ewj","pinned":false,"ibc_port":"wasm.abcdef"}"#).unwrap();
        assert_eq!(
            info,
            ContractInfoResponseCosmWasm1 {
                code_id: 3456,
                creator: "tgrade1js7ezrm55fqgxu3p62d9xn6patjku2z7ne5dvg".to_string(),
                admin: Some("tgrade1z363ulwcrxged4z5jswyt5dn5v3lzsemwz9ewj".to_string()),
                pinned: false,
                ibc_port: Some("wasm.abcdef".to_string()),
            }
        );
    }
}
