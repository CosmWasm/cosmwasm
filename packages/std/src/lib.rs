#[cfg(not(feature = "std"))]
core::compile_error!(
    r#"Please enable `cosmwasm-std`'s `std` feature, as we might move existing functionality to that feature in the future.
Builds without the std feature are currently not expected to work. If you need no_std support see #1484.
"#
);

#[macro_use]
extern crate alloc;

// Exposed on all platforms

mod __internal;
mod addresses;
mod assertions;
mod binary;
mod checksum;
mod coin;
mod coins;
mod conversion;
mod deps;
mod encoding;
mod errors;
mod forward_ref;
mod hex_binary;
mod ibc;
mod ibc2;
mod import_helpers;
#[cfg(feature = "iterator")]
mod iterator;
mod math;
mod metadata;
mod msgpack;
mod never;
mod pagination;
mod query;
mod results;
mod sections;
mod serde;
mod stdack;
mod timestamp;
mod traits;
mod types;
mod utils;

/// This module is to simplify no_std imports
pub(crate) mod prelude;

/// This modules is very advanced and will not be used directly by the vast majority of users.
/// We want to offer it to ensure a stable storage key composition system but don't encourage
/// contract devs to use it directly.
pub mod storage_keys;

pub use crate::addresses::{
    instantiate2_address, instantiate2_address_impl, Addr, CanonicalAddr, Instantiate2AddressError,
};
pub use crate::binary::Binary;
pub use crate::checksum::{Checksum, ChecksumError};
pub use crate::coin::{coin, coins, has_coins, Coin};
pub use crate::coins::Coins;
pub use crate::deps::{Deps, DepsMut, OwnedDeps};
pub use crate::encoding::{from_base64, from_hex, to_base64, to_hex};
pub use crate::errors::{
    AggregationError, CheckedFromRatioError, CheckedMultiplyFractionError,
    CheckedMultiplyRatioError, CoinFromStrError, CoinsError, ConversionOverflowError,
    DivideByZeroError, DivisionError, ErrorKind as StdErrorKind, OverflowError, OverflowOperation,
    PairingEqualityError, RecoverPubkeyError, RoundDownOverflowError, RoundUpOverflowError,
    StdError, StdResult, StdResultExt, SystemError, VerificationError,
};
pub use crate::hex_binary::HexBinary;
pub use crate::ibc::IbcChannelOpenResponse;
pub use crate::ibc::{
    Ibc3ChannelOpenResponse, IbcAckCallbackMsg, IbcAcknowledgement, IbcBasicResponse,
    IbcCallbackRequest, IbcChannel, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg,
    IbcDestinationCallbackMsg, IbcDstCallback, IbcEndpoint, IbcMsg, IbcOrder, IbcPacket,
    IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse,
    IbcSourceCallbackMsg, IbcSrcCallback, IbcTimeout, IbcTimeoutBlock, IbcTimeoutCallbackMsg,
    IbcTransferCallback, TransferMsgBuilder,
};
pub use crate::ibc2::{
    Ibc2Msg, Ibc2PacketAckMsg, Ibc2PacketReceiveMsg, Ibc2PacketSendMsg, Ibc2PacketTimeoutMsg,
    Ibc2Payload,
};
#[cfg(feature = "iterator")]
pub use crate::iterator::{Order, Record};
pub use crate::math::{
    Decimal, Decimal256, Decimal256RangeExceeded, DecimalRangeExceeded, Fraction, Int128, Int256,
    Int512, Int64, Isqrt, SignedDecimal, SignedDecimal256, SignedDecimal256RangeExceeded,
    SignedDecimalRangeExceeded, Uint128, Uint256, Uint512, Uint64,
};
pub use crate::metadata::{DenomMetadata, DenomUnit};
pub use crate::msgpack::{from_msgpack, to_msgpack_binary, to_msgpack_vec};
pub use crate::never::Never;
pub use crate::pagination::PageRequest;
pub use crate::query::{
    AllDelegationsResponse, AllDenomMetadataResponse, AllValidatorsResponse, BalanceResponse,
    BankQuery, BondedDenomResponse, ChannelResponse, CodeInfoResponse, ContractInfoResponse,
    CustomQuery, DecCoin, Delegation, DelegationResponse, DelegationRewardsResponse,
    DelegationTotalRewardsResponse, DelegatorReward, DelegatorValidatorsResponse,
    DelegatorWithdrawAddressResponse, DenomMetadataResponse, DistributionQuery, FullDelegation,
    GrpcQuery, IbcQuery, PortIdResponse, QueryRequest, RawRangeEntry, RawRangeResponse,
    StakingQuery, SupplyResponse, Validator, ValidatorMetadata, ValidatorResponse, WasmQuery,
};

#[cfg(all(feature = "stargate", feature = "cosmwasm_1_2"))]
pub use crate::results::WeightedVoteOption;
pub use crate::results::{
    attr, wasm_execute, wasm_instantiate, AnyMsg, Attribute, BankMsg, ContractResult, CosmosMsg,
    CustomMsg, Empty, Event, MsgResponse, QueryResponse, Reply, ReplyOn, Response, SubMsg,
    SubMsgResponse, SubMsgResult, SystemResult, WasmMsg,
};
#[cfg(feature = "staking")]
pub use crate::results::{DistributionMsg, StakingMsg};
#[cfg(feature = "stargate")]
pub use crate::results::{GovMsg, VoteOption};
pub use crate::serde::{from_json, to_json_binary, to_json_string, to_json_vec};
pub use crate::stdack::StdAck;
pub use crate::timestamp::Timestamp;
pub use crate::traits::{Api, HashFunction, Querier, QuerierResult, QuerierWrapper, Storage};
pub use crate::types::{BlockInfo, ContractInfo, Env, MessageInfo, MigrateInfo, TransactionInfo};

//
// Exports
//

#[cfg(all(feature = "exports", target_arch = "wasm32"))]
mod exports;

#[cfg(all(feature = "exports", target_arch = "wasm32", feature = "cosmwasm_2_2"))]
pub use crate::exports::do_migrate_with_info;
#[cfg(all(feature = "exports", target_arch = "wasm32"))]
pub use crate::exports::{
    do_execute, do_ibc_destination_callback, do_ibc_source_callback, do_instantiate, do_migrate,
    do_query, do_reply, do_sudo,
};
#[cfg(all(feature = "exports", target_arch = "wasm32", feature = "ibc2"))]
pub use crate::exports::{
    do_ibc2_packet_ack, do_ibc2_packet_receive, do_ibc2_packet_send, do_ibc2_packet_timeout,
};
#[cfg(all(feature = "exports", target_arch = "wasm32", feature = "stargate"))]
pub use crate::exports::{
    do_ibc_channel_close, do_ibc_channel_connect, do_ibc_channel_open, do_ibc_packet_ack,
    do_ibc_packet_receive, do_ibc_packet_timeout,
};

/// Exposed for testing only
/// Both unit tests and integration tests are compiled to native code, so everything in here does not need to compile to Wasm.
#[cfg(not(target_arch = "wasm32"))]
pub mod testing;

pub use cosmwasm_core::{BLS12_381_G1_GENERATOR, BLS12_381_G2_GENERATOR};

/// This attribute macro generates the boilerplate required to call into the
/// contract-specific logic from the entry-points to the Wasm module.
///
/// It should be added to the contract's init, handle, migrate and query implementations
/// like this:
/// ```
/// # use cosmwasm_std::{
/// #     Storage, Api, Querier, DepsMut, Deps, entry_point, Env, StdError, MessageInfo,
/// #     Response, QueryResponse,
/// # };
/// #
/// # type InstantiateMsg = ();
/// # type ExecuteMsg = ();
/// # type QueryMsg = ();
///
/// #[entry_point]
/// pub fn instantiate(
///     deps: DepsMut,
///     env: Env,
///     info: MessageInfo,
///     msg: InstantiateMsg,
/// ) -> Result<Response, StdError> {
/// #   Ok(Default::default())
/// }
///
/// #[entry_point]
/// pub fn execute(
///     deps: DepsMut,
///     env: Env,
///     info: MessageInfo,
///     msg: ExecuteMsg,
/// ) -> Result<Response, StdError> {
/// #   Ok(Default::default())
/// }
///
/// #[entry_point]
/// pub fn query(
///     deps: Deps,
///     env: Env,
///     msg: QueryMsg,
/// ) -> Result<QueryResponse, StdError> {
/// #   Ok(Default::default())
/// }
/// ```
///
/// where `InstantiateMsg`, `ExecuteMsg`, and `QueryMsg` are contract defined
/// types that implement `DeserializeOwned`.
///
/// ## Set the version of the state of your contract
///
/// The VM will use this as a hint whether it needs to run the migrate function of your contract or not.
///
/// ```
/// # use cosmwasm_std::{
/// #     DepsMut, entry_point, Env, MigrateInfo,
/// #     Response, StdResult,
/// # };
/// #
/// # type MigrateMsg = ();
/// #[entry_point]
/// #[migrate_version(2)]
/// pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg, migrate_info: MigrateInfo) -> StdResult<Response> {
///     todo!();
/// }
/// ```
///
/// It is also possible to assign the migrate version number to
/// a given constant name:
///
/// ```
/// # use cosmwasm_std::{
/// #     DepsMut, entry_point, Env, MigrateInfo,
/// #     Response, StdResult,
/// # };
/// #
/// # type MigrateMsg = ();
/// const CONTRACT_VERSION: u64 = 66;
///
/// #[entry_point]
/// #[migrate_version(CONTRACT_VERSION)]
/// pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg, migrate_info: MigrateInfo) -> StdResult<Response> {
///     todo!();
/// }
/// ```
pub use cosmwasm_derive::entry_point;
