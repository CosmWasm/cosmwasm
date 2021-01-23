#![cfg_attr(feature = "backtraces", feature(backtrace))]
#![allow(clippy::field_reassign_with_default)] // see https://github.com/CosmWasm/cosmwasm/issues/685

// Exposed on all platforms

mod addresses;
mod binary;
mod coins;
mod deps;
mod entry_points;
mod errors;
mod ibc;
#[cfg(feature = "iterator")]
mod iterator;
mod math;
mod query;
mod results;
mod sections;
mod serde;
mod storage;
mod traits;
mod types;

pub use crate::addresses::{CanonicalAddr, HumanAddr};
pub use crate::binary::{Binary, ByteArray};
pub use crate::coins::{coin, coins, has_coins, Coin};
pub use crate::deps::{Deps, DepsMut, OwnedDeps};
pub use crate::errors::{StdError, StdResult, SystemError};
#[cfg(feature = "stargate")]
pub use crate::ibc::{
    ChannelResponse, IbcAcknowledgement, IbcBasicResponse, IbcChannel, IbcEndpoint, IbcMsg,
    IbcOrder, IbcPacket, IbcQuery, IbcReceiveResponse, IbcTimeoutBlock, ListChannelsResponse,
    PortIdResponse,
};
#[cfg(feature = "iterator")]
pub use crate::iterator::{Order, KV};
pub use crate::math::{Decimal, Uint128};
pub use crate::query::{
    AllBalanceResponse, AllDelegationsResponse, BalanceResponse, BankQuery, BondedDenomResponse,
    CustomQuery, Delegation, FullDelegation, QueryRequest, StakingQuery, Validator,
    ValidatorsResponse, WasmQuery,
};
#[allow(deprecated)]
pub use crate::results::Context;
pub use crate::results::{
    attr, wasm_execute, wasm_instantiate, Attribute, BankMsg, ContractResult, CosmosMsg,
    HandleResponse, InitResponse, MigrateResponse, MutResponse, QueryResponse, StakingMsg,
    SystemResult, WasmMsg,
};
pub use crate::serde::{from_binary, from_slice, to_binary, to_vec};
pub use crate::storage::MemoryStorage;
pub use crate::traits::{Api, Querier, QuerierResult, QuerierWrapper, Storage};
pub use crate::types::{BlockInfo, ContractInfo, Empty, Env, MessageInfo};

// Exposed in wasm build only

#[cfg(target_arch = "wasm32")]
mod exports;
#[cfg(target_arch = "wasm32")]
mod imports;
#[cfg(target_arch = "wasm32")]
mod memory; // Used by exports and imports only. This assumes pointers are 32 bit long, which makes it untestable on dev machines.

#[cfg(target_arch = "wasm32")]
pub use crate::exports::{do_handle, do_init, do_migrate, do_query};
#[cfg(target_arch = "wasm32")]
pub use crate::imports::{ExternalApi, ExternalQuerier, ExternalStorage};

#[cfg(all(feature = "stargate", target_arch = "wasm32"))]
mod ibc_exports;
#[cfg(all(feature = "stargate", target_arch = "wasm32"))]
pub use crate::ibc_exports::{
    do_ibc_channel_close, do_ibc_channel_connect, do_ibc_channel_open, do_ibc_packet_ack,
    do_ibc_packet_receive, do_ibc_packet_timeout,
};

// Exposed for testing only
// Both unit tests and integration tests are compiled to native code, so everything in here does not need to compile to Wasm.

#[cfg(not(target_arch = "wasm32"))]
mod mock;
#[cfg(not(target_arch = "wasm32"))]
pub mod testing {
    pub use crate::mock::{
        digit_sum, mock_dependencies, mock_dependencies_with_balances, mock_env, mock_info,
        riffle_shuffle, BankQuerier, MockApi, MockQuerier, MockQuerierCustomHandlerResult,
        MockStorage, StakingQuerier, MOCK_CONTRACT_ADDR,
    };
    #[cfg(feature = "stargate")]
    pub use crate::mock::{mock_ibc_channel, mock_ibc_packet_ack, mock_ibc_packet_recv};
}

// Re-exports

pub use cosmwasm_derive::entry_point;
