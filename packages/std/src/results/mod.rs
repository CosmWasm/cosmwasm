//! This module contains the messages that are sent from the contract to the VM as an execution result

mod attribute;
mod contract_result;
mod cosmos_msg;
mod empty;
mod query;
mod response;
mod subcall;
mod system_result;

pub use attribute::{attr, Attribute};
pub use contract_result::ContractResult;
pub use cosmos_msg::{wasm_execute, wasm_instantiate, BankMsg, CosmosMsg, WasmMsg};
#[cfg(feature = "staking")]
pub use cosmos_msg::{DistributionMsg, StakingMsg};
pub use empty::Empty;
pub use query::QueryResponse;
pub use response::Response;
pub use subcall::{Event, Reply, ReplyOn, SubMsg, SubcallResponse};
pub use system_result::SystemResult;

#[deprecated(since = "0.14.0", note = "Renamed to Response.")]
pub type InitResponse<T = Empty> = Response<T>;

#[deprecated(since = "0.14.0", note = "Renamed to Response.")]
pub type HandleResponse<T = Empty> = Response<T>;

#[deprecated(since = "0.14.0", note = "Renamed to Response.")]
pub type MigrateResponse<T = Empty> = Response<T>;
