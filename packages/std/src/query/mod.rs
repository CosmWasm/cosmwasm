// needed because the derive macros on QueryRequest use the deprecated `Stargate` variant
#![allow(deprecated)]

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::prelude::*;
use crate::Binary;
use crate::Empty;

mod bank;
mod distribution;
mod ibc;
mod query_response;
mod staking;
mod wasm;

pub use bank::*;
pub use distribution::*;
pub use ibc::*;
pub use staking::*;
pub use wasm::*;

#[non_exhaustive]
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
#[serde(rename_all = "snake_case")]
pub enum QueryRequest<C = Empty> {
    Bank(BankQuery),
    Custom(C),
    #[cfg(feature = "staking")]
    Staking(StakingQuery),
    #[cfg(feature = "cosmwasm_1_3")]
    Distribution(DistributionQuery),
    /// A Stargate query is encoded the same way as abci_query, with path and protobuf encoded request data.
    /// The format is defined in [ADR-21](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-021-protobuf-query-encoding.md).
    /// The response is protobuf encoded data directly without a JSON response wrapper.
    /// The caller is responsible for compiling the proper protobuf definitions for both requests and responses.
    #[cfg(feature = "stargate")]
    #[deprecated = "Please use the GrpcQuery instead"]
    Stargate {
        /// this is the fully qualified service path used for routing,
        /// eg. "/cosmos_sdk.x.bank.v1.Query/QueryBalance"
        path: String,
        /// this is the expected protobuf message type (not any), binary encoded
        data: Binary,
    },
    #[cfg(feature = "stargate")]
    Ibc(IbcQuery),
    Wasm(WasmQuery),
    #[cfg(feature = "cosmwasm_2_0")]
    Grpc(GrpcQuery),
}

/// Queries the chain using a grpc query.
/// This allows to query information that is not exposed in our API.
/// The chain needs to allowlist the supported queries.
/// The drawback of this query is that you have to handle the protobuf encoding and decoding yourself.
///
/// The returned data is protobuf encoded. The protobuf type depends on the query.
/// Because of this, using it with the [`query`](crate::QuerierWrapper::query) function will result
/// in a deserialization error.
/// Use [`raw_query`](crate::Querier::raw_query) or [`query_grpc`](crate::QuerierWrapper::query_grpc)
/// instead.
///
/// To find the path, as well as the request and response types,
/// you can query the chain's gRPC endpoint using a tool like
/// [grpcurl](https://github.com/fullstorydev/grpcurl).
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, cw_schema::Schemaifier,
)]
pub struct GrpcQuery {
    /// The fully qualified endpoint path used for routing.
    /// It follows the format `/service_path/method_name`,
    /// eg. "/cosmos.authz.v1beta1.Query/Grants"
    pub path: String,
    /// The expected protobuf message type (not [Any](https://protobuf.dev/programming-guides/proto3/#any)), binary encoded
    pub data: Binary,
}

/// A trait that is required to avoid conflicts with other query types like BankQuery and WasmQuery
/// in generic implementations.
/// You need to implement it in your custom query type.
///
/// # Examples
///
/// ```
/// # use cosmwasm_std::CustomQuery;
/// # use schemars::JsonSchema;
/// # use serde::{Deserialize, Serialize};
/// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, cw_schema::Schemaifier)]
/// #[serde(rename_all = "snake_case")]
/// pub enum MyCustomQuery {
///     Ping {},
///     Capitalized { text: String },
/// }
///
/// impl CustomQuery for MyCustomQuery {}
/// ```
pub trait CustomQuery: Serialize + Clone {}
// We require `Clone` because `Clone` in `QueryRequest<C>` is only derived for
// `C: Clone` and we want consistent behaviour for all `QueryRequest<C>`

impl CustomQuery for Empty {}

impl<C: CustomQuery> From<BankQuery> for QueryRequest<C> {
    fn from(msg: BankQuery) -> Self {
        QueryRequest::Bank(msg)
    }
}

impl<C: CustomQuery> From<C> for QueryRequest<C> {
    fn from(msg: C) -> Self {
        QueryRequest::Custom(msg)
    }
}

#[cfg(feature = "staking")]
impl<C: CustomQuery> From<StakingQuery> for QueryRequest<C> {
    fn from(msg: StakingQuery) -> Self {
        QueryRequest::Staking(msg)
    }
}

impl<C: CustomQuery> From<WasmQuery> for QueryRequest<C> {
    fn from(msg: WasmQuery) -> Self {
        QueryRequest::Wasm(msg)
    }
}

#[cfg(feature = "cosmwasm_2_0")]
impl<C: CustomQuery> From<GrpcQuery> for QueryRequest<C> {
    fn from(msg: GrpcQuery) -> Self {
        QueryRequest::Grpc(msg)
    }
}

#[cfg(feature = "stargate")]
impl<C: CustomQuery> From<IbcQuery> for QueryRequest<C> {
    fn from(msg: IbcQuery) -> Self {
        QueryRequest::Ibc(msg)
    }
}

#[cfg(feature = "cosmwasm_1_3")]
impl<C: CustomQuery> From<DistributionQuery> for QueryRequest<C> {
    fn from(msg: DistributionQuery) -> Self {
        QueryRequest::Distribution(msg)
    }
}
