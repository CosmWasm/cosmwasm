#![cfg_attr(feature = "backtraces", feature(backtrace))]

// Exposed on all platforms

mod addresses;
mod binary;
mod coins;
mod conversion;
mod deps;
mod entry_points;
mod errors;
mod ibc;
mod import_helpers;
#[cfg(feature = "iterator")]
mod iterator;
mod math;
mod query;
mod results;
mod sections;
mod serde;
mod storage;
mod timestamp;
mod traits;
mod types;

#[allow(deprecated)]
pub use crate::addresses::{Addr, CanonicalAddr, HumanAddr};
pub use crate::binary::Binary;
pub use crate::coins::{coin, coins, has_coins, Coin};
pub use crate::deps::{Deps, DepsMut, OwnedDeps};
pub use crate::errors::{
    OverflowError, OverflowOperation, RecoverPubkeyError, StdError, StdResult, SystemError,
    VerificationError,
};
#[cfg(feature = "stargate")]
pub use crate::ibc::{
    IbcAcknowledgement, IbcAcknowledgementWithPacket, IbcBasicResponse, IbcChannel,
    IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcEndpoint, IbcMsg, IbcOrder,
    IbcPacket, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse,
    IbcTimeout, IbcTimeoutBlock,
};
#[cfg(feature = "iterator")]
#[allow(deprecated)]
pub use crate::iterator::{Order, Pair, KV};
pub use crate::math::{Decimal, Fraction, Uint128, Uint64};
pub use crate::query::{
    AllBalanceResponse, BalanceResponse, BankQuery, CustomQuery, QueryRequest, WasmQuery,
};
#[cfg(feature = "staking")]
pub use crate::query::{
    AllDelegationsResponse, AllValidatorsResponse, BondedDenomResponse, Delegation, FullDelegation,
    StakingQuery, Validator, ValidatorResponse,
};
#[cfg(feature = "stargate")]
pub use crate::query::{ChannelResponse, IbcQuery, ListChannelsResponse, PortIdResponse};
pub use crate::results::{
    attr, wasm_execute, wasm_instantiate, Attribute, BankMsg, ContractResult, CosmosMsg, Empty,
    Event, QueryResponse, Reply, ReplyOn, Response, SubMsg, SubMsgExecutionResponse, SystemResult,
    WasmMsg,
};
#[cfg(feature = "staking")]
pub use crate::results::{DistributionMsg, StakingMsg};
pub use crate::serde::{from_binary, from_slice, to_binary, to_vec};
pub use crate::storage::MemoryStorage;
pub use crate::timestamp::Timestamp;
pub use crate::traits::{Api, Querier, QuerierResult, QuerierWrapper, Storage};
pub use crate::types::{BlockInfo, ContractInfo, Env, MessageInfo};

// Exposed in wasm build only

#[cfg(target_arch = "wasm32")]
mod exports;
#[cfg(target_arch = "wasm32")]
mod imports;
#[cfg(target_arch = "wasm32")]
mod memory; // Used by exports and imports only. This assumes pointers are 32 bit long, which makes it untestable on dev machines.

#[cfg(target_arch = "wasm32")]
pub use crate::exports::{do_execute, do_instantiate, do_migrate, do_query, do_reply, do_sudo};
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
    #[cfg(feature = "staking")]
    pub use crate::mock::StakingQuerier;
    pub use crate::mock::{
        digit_sum, mock_dependencies, mock_dependencies_with_balances, mock_env, mock_info,
        riffle_shuffle, BankQuerier, MockApi, MockQuerier, MockQuerierCustomHandlerResult,
        MockStorage, MOCK_CONTRACT_ADDR,
    };
    #[cfg(feature = "stargate")]
    pub use crate::mock::{mock_ibc_channel, mock_ibc_packet_ack, mock_ibc_packet_recv};
}

// Re-exports

pub use cosmwasm_derive::entry_point;
