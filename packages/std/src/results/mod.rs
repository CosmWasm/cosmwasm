//! This module contains the messages that are sent from the contract to the VM as an execution result

mod attribute;
mod contract_result;
mod cosmos_msg;
mod empty;
mod query;
mod response;
mod submessages;
mod system_result;

pub use attribute::{attr, Attribute};
pub use contract_result::ContractResult;
#[cfg(feature = "stargate")]
pub use cosmos_msg::{GovMsg, VoteOption};
pub use cosmos_msg::{wasm_execute, wasm_instantiate, BankMsg, CosmosMsg, WasmMsg};
#[cfg(feature = "staking")]
pub use cosmos_msg::{DistributionMsg, StakingMsg};
pub use empty::Empty;
pub use query::QueryResponse;
pub use response::Response;
pub use submessages::{Event, Reply, ReplyOn, SubMsg, SubMsgExecutionResponse};
pub use system_result::SystemResult;
