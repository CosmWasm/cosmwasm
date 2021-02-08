//! This module contains the messages that are sent from the contract to the VM as an execution result

mod attribute;
mod context;
mod contract_result;
mod cosmos_msg;
mod empty;
mod query;
mod response;
mod system_result;

pub use attribute::{attr, Attribute};
#[allow(deprecated)]
pub use context::Context;
pub use contract_result::ContractResult;
pub use cosmos_msg::{wasm_execute, wasm_instantiate, BankMsg, CosmosMsg, StakingMsg, WasmMsg};
pub use empty::Empty;
pub use query::QueryResponse;
pub use response::Response;
pub use system_result::SystemResult;

pub type InitResponse<T = Empty> = Response<T>;
pub type HandleResponse<T = Empty> = Response<T>;
pub type MigrateResponse<T = Empty> = Response<T>;
