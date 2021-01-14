//! This module contains the messages that are sent from the contract to the VM as an execution result

mod attribute;
mod context;
mod contract_result;
mod cosmos_msg;
mod handle;
mod init;
mod migrate;
mod query;
mod system_result;

pub use attribute::{attr, Attribute};
pub use context::Context;
pub use contract_result::ContractResult;
pub use cosmos_msg::{wasm_execute, wasm_instantiate, BankMsg, CosmosMsg, StakingMsg, WasmMsg};
#[allow(deprecated)]
pub use handle::{HandleResponse, HandleResult};
#[allow(deprecated)]
pub use init::{InitResponse, InitResult};
#[allow(deprecated)]
pub use migrate::{MigrateResponse, MigrateResult};
#[allow(deprecated)]
pub use query::{QueryResponse, QueryResult};
pub use system_result::SystemResult;
