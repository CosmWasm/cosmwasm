#![cfg(not(target_arch = "wasm32"))]

// Exposed for testing only
// Both unit tests and integration tests are compiled to native code, so everything in here does not need to compile to Wasm.

mod assertions;
mod message_info;
mod mock;
mod storage;

pub use assertions::assert_approx_eq_impl;
#[cfg(test)]
pub use assertions::assert_hash_works_impl;

pub use message_info::message_info;
#[cfg(feature = "cosmwasm_1_3")]
pub use mock::DistributionQuerier;
#[cfg(feature = "staking")]
pub use mock::StakingQuerier;
pub use mock::{
    mock_dependencies, mock_dependencies_with_balance, mock_dependencies_with_balances, mock_env,
    mock_wasmd_attr, BankQuerier, Envs, EnvsOptions, MockApi, MockQuerier,
    MockQuerierCustomHandlerResult, MOCK_CONTRACT_ADDR,
};
#[cfg(feature = "ibc2")]
pub use mock::{
    mock_ibc2_packet_ack, mock_ibc2_packet_recv, mock_ibc2_packet_send, mock_ibc2_packet_timeout,
};
#[cfg(feature = "stargate")]
pub use mock::{
    mock_ibc_channel, mock_ibc_channel_close_confirm, mock_ibc_channel_close_init,
    mock_ibc_channel_connect_ack, mock_ibc_channel_connect_confirm, mock_ibc_channel_open_init,
    mock_ibc_channel_open_try, mock_ibc_packet_ack, mock_ibc_packet_recv, mock_ibc_packet_timeout,
};

pub use storage::MockStorage;
