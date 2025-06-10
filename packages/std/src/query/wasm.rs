use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::prelude::*;
#[cfg(all(feature = "cosmwasm_3_0", feature = "iterator"))]
use crate::storage_keys::{range_to_bounds, ToByteVec};
use crate::{Addr, Binary, Checksum};
#[cfg(all(feature = "cosmwasm_3_0", feature = "iterator"))]
use core::ops::RangeBounds;

use super::query_response::QueryResponseType;

use crate::utils::impl_hidden_constructor;

#[non_exhaustive]
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
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
    /// Queries a range of keys from the storage of a (different) contract,
    /// returning a [`RawRangeResponse`].
    ///
    /// This is a low-level query that allows you to query the storage of another contract.
    /// Please keep in mind that the contract you are querying might change its storage layout using
    /// migrations, which could break your queries, so it is recommended to only use this for
    /// contracts you control.
    #[cfg(all(feature = "cosmwasm_3_0", feature = "iterator"))]
    RawRange {
        /// The address of the contract to query
        contract_addr: String,
        /// Inclusive start bound. This is the first key you would like to get data for.
        ///
        /// If `start` is lexicographically greater than or equal to `end`,
        /// an empty range is described, mo matter of the order.
        start: Option<Binary>,
        /// Exclusive end bound. This is the key after the last key you would like to get data for.
        end: Option<Binary>,
        /// Maximum number of elements to return.
        ///
        /// Make sure to set a reasonable limit to avoid running out of memory or into
        /// the deserialization limits of the VM. Also keep in mind that these limitations depend
        /// on the full JSON size of the response type.
        limit: u16,
        /// The order in which you want to receive the key-value pairs.
        order: crate::Order,
    },
}

impl WasmQuery {
    /// Creates a new [`WasmQuery::RawRange`] from the given parameters.
    ///
    /// This takes a [`RangeBounds`] to allow for specifying the range in a more idiomatic way.
    #[cfg(all(feature = "cosmwasm_3_0", feature = "iterator"))]
    pub fn raw_range<'a, R, B>(
        contract_addr: impl Into<String>,
        range: R,
        limit: u16,
        order: crate::Order,
    ) -> Self
    where
        R: RangeBounds<&'a B>,
        B: ToByteVec + ?Sized + 'a,
    {
        let (start, end) = range_to_bounds(&range);

        WasmQuery::RawRange {
            contract_addr: contract_addr.into(),
            start: start.map(Binary::new),
            end: end.map(Binary::new),
            limit,
            order,
        }
    }
}

#[non_exhaustive]
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
pub struct ContractInfoResponse {
    pub code_id: u64,
    /// address that instantiated this contract
    pub creator: Addr,
    /// admin who can run migrations (if any)
    pub admin: Option<Addr>,
    /// if set, the contract is pinned to the cache, and thus uses less gas when called
    pub pinned: bool,
    /// set if this contract has bound an IBC port
    pub ibc_port: Option<String>,
    /// set if this contract has bound an Ibc2 port
    pub ibc2_port: Option<String>,
}

impl QueryResponseType for ContractInfoResponse {}

impl_hidden_constructor!(
    ContractInfoResponse,
    code_id: u64,
    creator: Addr,
    admin: Option<Addr>,
    pinned: bool,
    ibc_port: Option<String>,
    ibc2_port: Option<String>
);

/// The essential data from wasmd's [CodeInfo]/[CodeInfoResponse].
///
/// `code_hash`/`data_hash` was renamed to `checksum` to follow the CosmWasm
/// convention and naming in `instantiate2_address`.
///
/// [CodeInfo]: https://github.com/CosmWasm/wasmd/blob/v0.30.0/proto/cosmwasm/wasm/v1/types.proto#L62-L72
/// [CodeInfoResponse]: https://github.com/CosmWasm/wasmd/blob/v0.30.0/proto/cosmwasm/wasm/v1/query.proto#L184-L199
#[non_exhaustive]
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
pub struct CodeInfoResponse {
    pub code_id: u64,
    /// The address that initially stored the code
    pub creator: Addr,
    /// The hash of the Wasm blob
    pub checksum: Checksum,
}

impl_hidden_constructor!(
    CodeInfoResponse,
    code_id: u64,
    creator: Addr,
    checksum: Checksum
);

impl QueryResponseType for CodeInfoResponse {}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct RawRangeResponse {
    /// The key-value pairs
    pub data: Vec<RawRangeEntry>,
    /// `None` if there are no more key-value pairs within the given key range.
    pub next_key: Option<Binary>,
}

impl_hidden_constructor!(
    RawRangeResponse,
    data: Vec<RawRangeEntry>,
    next_key: Option<Binary>
);

pub type RawRangeEntry = (Binary, Binary);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::to_json_binary;

    #[test]
    fn wasm_query_contract_info_serialization() {
        let query = WasmQuery::ContractInfo {
            contract_addr: "aabbccdd456".into(),
        };
        let json = to_json_binary(&query).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&json),
            r#"{"contract_info":{"contract_addr":"aabbccdd456"}}"#,
        );
    }

    #[test]
    #[cfg(feature = "cosmwasm_1_2")]
    fn wasm_query_code_info_serialization() {
        let query = WasmQuery::CodeInfo { code_id: 70 };
        let json = to_json_binary(&query).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&json),
            r#"{"code_info":{"code_id":70}}"#,
        );
    }

    #[test]
    fn contract_info_response_serialization() {
        let response = ContractInfoResponse {
            code_id: 67,
            creator: Addr::unchecked("jane"),
            admin: Some(Addr::unchecked("king")),
            pinned: true,
            ibc_port: Some("wasm.123".to_string()),
            ibc2_port: Some("wasm.123".to_string()),
        };
        let json = to_json_binary(&response).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&json),
            r#"{"code_id":67,"creator":"jane","admin":"king","pinned":true,"ibc_port":"wasm.123","ibc2_port":"wasm.123"}"#,
        );
    }

    #[test]
    #[cfg(feature = "cosmwasm_1_2")]
    fn code_info_response_serialization() {
        use crate::Checksum;

        let response = CodeInfoResponse {
            code_id: 67,
            creator: Addr::unchecked("jane"),
            checksum: Checksum::from_hex(
                "f7bb7b18fb01bbf425cf4ed2cd4b7fb26a019a7fc75a4dc87e8a0b768c501f00",
            )
            .unwrap(),
        };
        let json = to_json_binary(&response).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&json),
            r#"{"code_id":67,"creator":"jane","checksum":"f7bb7b18fb01bbf425cf4ed2cd4b7fb26a019a7fc75a4dc87e8a0b768c501f00"}"#,
        );
    }

    #[test]
    #[cfg(all(feature = "cosmwasm_3_0", feature = "iterator"))]
    fn raw_range_constructor_works() {
        use crate::Order;

        let query = WasmQuery::raw_range(
            "contract_addr",
            &b"asdf"[..]..&b"asdz"[..],
            100,
            Order::Ascending,
        );

        assert_eq!(
            query,
            WasmQuery::RawRange {
                contract_addr: "contract_addr".to_string(),
                start: Some(Binary::from(b"asdf")),
                end: Some(Binary::from(b"asdz")),
                limit: 100,
                order: Order::Ascending,
            }
        );

        let query = WasmQuery::raw_range("contract_addr", b"asdf"..=b"asdz", 100, Order::Ascending);
        assert_eq!(
            query,
            WasmQuery::RawRange {
                contract_addr: "contract_addr".to_string(),
                start: Some(Binary::from(b"asdf")),
                end: Some(Binary::from(b"asdz\0")),
                limit: 100,
                order: Order::Ascending,
            }
        );
    }

    #[test]
    fn raw_range_response_serialization() {
        let response = RawRangeResponse {
            data: vec![
                (Binary::from(b"key"), Binary::from(b"value")),
                (Binary::from(b"foo"), Binary::from(b"bar")),
            ],
            next_key: Some(Binary::from(b"next")),
        };
        let json = to_json_binary(&response).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&json),
            r#"{"data":[["a2V5","dmFsdWU="],["Zm9v","YmFy"]],"next_key":"bmV4dA=="}"#,
        );
    }
}
