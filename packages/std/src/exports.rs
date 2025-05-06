//! exports exposes the public wasm API
//!
//! interface_version_8, allocate and deallocate turn into Wasm exports
//! as soon as cosmwasm_std is `use`d in the contract, even privately.
//!
//! `do_execute`, `do_instantiate`, `do_migrate`, `do_query`, `do_reply`
//! and `do_sudo` should be wrapped with a extern "C" entry point including
//! the contract-specific function pointer. This is done via the `#[entry_point]`
//! macro attribute from cosmwasm-derive.
use alloc::vec::Vec;
use core::{marker::PhantomData, ptr};

use serde::de::DeserializeOwned;

use crate::deps::OwnedDeps;
#[cfg(any(feature = "stargate", feature = "ibc2"))]
use crate::ibc::IbcReceiveResponse;
use crate::ibc::{IbcBasicResponse, IbcDestinationCallbackMsg, IbcSourceCallbackMsg};
#[cfg(feature = "stargate")]
use crate::ibc::{
    IbcChannelCloseMsg, IbcChannelConnectMsg, IbcPacketAckMsg, IbcPacketReceiveMsg,
    IbcPacketTimeoutMsg,
};
use crate::ibc::{IbcChannelOpenMsg, IbcChannelOpenResponse};
#[cfg(feature = "ibc2")]
use crate::ibc2::{Ibc2PacketReceiveMsg, Ibc2PacketTimeoutMsg};
use crate::imports::{ExternalApi, ExternalQuerier, ExternalStorage};
use crate::memory::{Owned, Region};
use crate::panic::install_panic_handler;
use crate::query::CustomQuery;
use crate::results::{ContractResult, QueryResponse, Reply, Response};
use crate::serde::{from_json, to_json_vec};
use crate::types::Env;
use crate::{CustomMsg, Deps, DepsMut, MessageInfo, MigrateInfo};

// These functions are used as markers for the chain to know which features this contract requires.
// If the chain does not support all the required features, it will reject storing the contract.
// See `docs/CAPABILITIES.md` for more details.
#[cfg(feature = "iterator")]
#[no_mangle]
extern "C" fn requires_iterator() {}

#[cfg(feature = "staking")]
#[no_mangle]
extern "C" fn requires_staking() {}

#[cfg(feature = "stargate")]
#[no_mangle]
extern "C" fn requires_stargate() {}

#[cfg(feature = "ibc2")]
#[no_mangle]
extern "C" fn requires_ibc2() {}

#[cfg(feature = "cosmwasm_1_1")]
#[no_mangle]
extern "C" fn requires_cosmwasm_1_1() {}

#[cfg(feature = "cosmwasm_1_2")]
#[no_mangle]
extern "C" fn requires_cosmwasm_1_2() {}

#[cfg(feature = "cosmwasm_1_3")]
#[no_mangle]
extern "C" fn requires_cosmwasm_1_3() {}

#[cfg(feature = "cosmwasm_1_4")]
#[no_mangle]
extern "C" fn requires_cosmwasm_1_4() {}

#[cfg(feature = "cosmwasm_2_0")]
#[no_mangle]
extern "C" fn requires_cosmwasm_2_0() {}

#[cfg(feature = "cosmwasm_2_1")]
#[no_mangle]
extern "C" fn requires_cosmwasm_2_1() {}

#[cfg(feature = "cosmwasm_2_2")]
#[no_mangle]
extern "C" fn requires_cosmwasm_2_2() {}

/// interface_version_* exports mark which Wasm VM interface level this contract is compiled for.
/// They can be checked by cosmwasm_vm.
/// Update this whenever the Wasm VM interface breaks.
#[no_mangle]
extern "C" fn interface_version_8() {}

/// allocate reserves the given number of bytes in wasm memory and returns a pointer
/// to a Region defining this data. This space is managed by the calling process
/// and should be accompanied by a corresponding deallocate
#[no_mangle]
extern "C" fn allocate(size: usize) -> u32 {
    Region::with_capacity(size).to_heap_ptr() as u32
}

/// deallocate expects a pointer to a Region created with allocate.
/// It will free both the Region and the memory referenced by the Region.
#[no_mangle]
extern "C" fn deallocate(pointer: u32) {
    // auto-drop Region on function end
    let _ =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(pointer as *mut Region<Owned>).unwrap()) };
}

// TODO: replace with https://doc.rust-lang.org/std/ops/trait.Try.html once stabilized
macro_rules! r#try_into_contract_result {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => {
                return ContractResult::Err(err.to_string());
            }
        }
    };
    ($expr:expr,) => {
        $crate::try_into_contract_result!($expr)
    };
}

/// This should be wrapped in an external "C" export, containing a contract-specific function as an argument.
///
/// - `Q`: custom query type (see QueryRequest)
/// - `M`: message type for request
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
pub fn do_instantiate<Q, M, C, E>(
    instantiate_fn: &dyn Fn(DepsMut<Q>, Env, MessageInfo, M) -> Result<Response<C>, E>,
    env_ptr: u32,
    info_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    M: DeserializeOwned,
    C: CustomMsg,
    E: ToString,
{
    install_panic_handler();
    let res = _do_instantiate(
        instantiate_fn,
        env_ptr as *mut Region<Owned>,
        info_ptr as *mut Region<Owned>,
        msg_ptr as *mut Region<Owned>,
    );
    let v = to_json_vec(&res).unwrap();
    Region::from_vec(v).to_heap_ptr() as u32
}

/// do_execute should be wrapped in an external "C" export, containing a contract-specific function as arg
///
/// - `Q`: custom query type (see QueryRequest)
/// - `M`: message type for request
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
pub fn do_execute<Q, M, C, E>(
    execute_fn: &dyn Fn(DepsMut<Q>, Env, MessageInfo, M) -> Result<Response<C>, E>,
    env_ptr: u32,
    info_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    M: DeserializeOwned,
    C: CustomMsg,
    E: ToString,
{
    install_panic_handler();
    let res = _do_execute(
        execute_fn,
        env_ptr as *mut Region<Owned>,
        info_ptr as *mut Region<Owned>,
        msg_ptr as *mut Region<Owned>,
    );
    let v = to_json_vec(&res).unwrap();
    Region::from_vec(v).to_heap_ptr() as u32
}

/// do_migrate should be wrapped in an external "C" export, containing a contract-specific function as arg
///
/// - `Q`: custom query type (see QueryRequest)
/// - `M`: message type for request
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
pub fn do_migrate<Q, M, C, E>(
    migrate_fn: &dyn Fn(DepsMut<Q>, Env, M) -> Result<Response<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    M: DeserializeOwned,
    C: CustomMsg,
    E: ToString,
{
    install_panic_handler();
    let res = _do_migrate(
        migrate_fn,
        env_ptr as *mut Region<Owned>,
        msg_ptr as *mut Region<Owned>,
    );
    let v = to_json_vec(&res).unwrap();
    Region::from_vec(v).to_heap_ptr() as u32
}

/// do_migrate_with_info should be wrapped in an external "C" export,
/// containing a contract-specific function as arg
///
/// - `Q`: custom query type (see QueryRequest)
/// - `M`: message type for request
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
#[cfg(feature = "cosmwasm_2_2")]
pub fn do_migrate_with_info<Q, M, C, E>(
    migrate_with_info_fn: &dyn Fn(DepsMut<Q>, Env, M, MigrateInfo) -> Result<Response<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
    migrate_info_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    M: DeserializeOwned,
    C: CustomMsg,
    E: ToString,
{
    install_panic_handler();
    let res = _do_migrate_with_info(
        migrate_with_info_fn,
        env_ptr as *mut Region<Owned>,
        msg_ptr as *mut Region<Owned>,
        migrate_info_ptr as *mut Region<Owned>,
    );
    let v = to_json_vec(&res).unwrap();
    Region::from_vec(v).to_heap_ptr() as u32
}

/// do_sudo should be wrapped in an external "C" export, containing a contract-specific function as arg
///
/// - `Q`: custom query type (see QueryRequest)
/// - `M`: message type for request
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
pub fn do_sudo<Q, M, C, E>(
    sudo_fn: &dyn Fn(DepsMut<Q>, Env, M) -> Result<Response<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    M: DeserializeOwned,
    C: CustomMsg,
    E: ToString,
{
    install_panic_handler();
    let res = _do_sudo(
        sudo_fn,
        env_ptr as *mut Region<Owned>,
        msg_ptr as *mut Region<Owned>,
    );
    let v = to_json_vec(&res).unwrap();
    Region::from_vec(v).to_heap_ptr() as u32
}

/// do_reply should be wrapped in an external "C" export, containing a contract-specific function as arg
/// message body is always `SubcallResult`
///
/// - `Q`: custom query type (see QueryRequest)
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
pub fn do_reply<Q, C, E>(
    reply_fn: &dyn Fn(DepsMut<Q>, Env, Reply) -> Result<Response<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    install_panic_handler();
    let res = _do_reply(
        reply_fn,
        env_ptr as *mut Region<Owned>,
        msg_ptr as *mut Region<Owned>,
    );
    let v = to_json_vec(&res).unwrap();
    Region::from_vec(v).to_heap_ptr() as u32
}

/// do_query should be wrapped in an external "C" export, containing a contract-specific function as arg
///
/// - `Q`: custom query type (see QueryRequest)
/// - `M`: message type for request
/// - `E`: error type for responses
pub fn do_query<Q, M, E>(
    query_fn: &dyn Fn(Deps<Q>, Env, M) -> Result<QueryResponse, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    M: DeserializeOwned,
    E: ToString,
{
    install_panic_handler();
    let res = _do_query(
        query_fn,
        env_ptr as *mut Region<Owned>,
        msg_ptr as *mut Region<Owned>,
    );
    let v = to_json_vec(&res).unwrap();
    Region::from_vec(v).to_heap_ptr() as u32
}

/// do_ibc_channel_open is designed for use with #[entry_point] to make a "C" extern
///
/// contract_fn does the protocol version negotiation during channel handshake phase
///
/// - `Q`: custom query type (see QueryRequest)
/// - `E`: error type for responses
#[cfg(feature = "stargate")]
pub fn do_ibc_channel_open<Q, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcChannelOpenMsg) -> Result<IbcChannelOpenResponse, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    E: ToString,
{
    install_panic_handler();
    let res = _do_ibc_channel_open(
        contract_fn,
        env_ptr as *mut Region<Owned>,
        msg_ptr as *mut Region<Owned>,
    );
    let v = to_json_vec(&res).unwrap();
    Region::from_vec(v).to_heap_ptr() as u32
}

/// do_ibc_channel_connect is designed for use with #[entry_point] to make a "C" extern
///
/// contract_fn is a callback when a IBC channel is established (after both sides agree in open)
///
/// - `Q`: custom query type (see QueryRequest)
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
#[cfg(feature = "stargate")]
pub fn do_ibc_channel_connect<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcChannelConnectMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    install_panic_handler();
    let res = _do_ibc_channel_connect(
        contract_fn,
        env_ptr as *mut Region<Owned>,
        msg_ptr as *mut Region<Owned>,
    );
    let v = to_json_vec(&res).unwrap();
    Region::from_vec(v).to_heap_ptr() as u32
}

/// do_ibc_channel_close is designed for use with #[entry_point] to make a "C" extern
///
/// contract_fn is a callback when a IBC channel belonging to this contract is closed
///
/// - `Q`: custom query type (see QueryRequest)
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
#[cfg(feature = "stargate")]
pub fn do_ibc_channel_close<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcChannelCloseMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    install_panic_handler();
    let res = _do_ibc_channel_close(
        contract_fn,
        env_ptr as *mut Region<Owned>,
        msg_ptr as *mut Region<Owned>,
    );
    let v = to_json_vec(&res).unwrap();
    Region::from_vec(v).to_heap_ptr() as u32
}

/// do_ibc_packet_receive is designed for use with #[entry_point] to make a "C" extern
///
/// contract_fn is called when this chain receives an IBC Packet on a channel belonging
/// to this contract
///
/// - `Q`: custom query type (see QueryRequest)
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
#[cfg(feature = "stargate")]
pub fn do_ibc_packet_receive<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcPacketReceiveMsg) -> Result<IbcReceiveResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    install_panic_handler();
    let res = _do_ibc_packet_receive(
        contract_fn,
        env_ptr as *mut Region<Owned>,
        msg_ptr as *mut Region<Owned>,
    );
    let v = to_json_vec(&res).unwrap();
    Region::from_vec(v).to_heap_ptr() as u32
}

/// do_ibc_packet_ack is designed for use with #[entry_point] to make a "C" extern
///
/// contract_fn is called when this chain receives an IBC Acknowledgement for a packet
/// that this contract previously sent
///
/// - `Q`: custom query type (see QueryRequest)
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
#[cfg(feature = "stargate")]
pub fn do_ibc_packet_ack<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcPacketAckMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    install_panic_handler();
    let res = _do_ibc_packet_ack(
        contract_fn,
        env_ptr as *mut Region<Owned>,
        msg_ptr as *mut Region<Owned>,
    );
    let v = to_json_vec(&res).unwrap();
    Region::from_vec(v).to_heap_ptr() as u32
}

/// do_ibc_packet_timeout is designed for use with #[entry_point] to make a "C" extern
///
/// contract_fn is called when a packet that this contract previously sent has provably
/// timed out and will never be relayed to the destination chain.
///
/// - `Q`: custom query type (see QueryRequest)
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
#[cfg(feature = "stargate")]
pub fn do_ibc_packet_timeout<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcPacketTimeoutMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    install_panic_handler();
    let res = _do_ibc_packet_timeout(
        contract_fn,
        env_ptr as *mut Region<Owned>,
        msg_ptr as *mut Region<Owned>,
    );
    let v = to_json_vec(&res).unwrap();
    Region::from_vec(v).to_heap_ptr() as u32
}

pub fn do_ibc_source_callback<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcSourceCallbackMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    install_panic_handler();
    let res = _do_ibc_source_callback(
        contract_fn,
        env_ptr as *mut Region<Owned>,
        msg_ptr as *mut Region<Owned>,
    );
    let v = to_json_vec(&res).unwrap();
    Region::from_vec(v).to_heap_ptr() as u32
}

pub fn do_ibc_destination_callback<Q, C, E>(
    contract_fn: &dyn Fn(
        DepsMut<Q>,
        Env,
        IbcDestinationCallbackMsg,
    ) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    install_panic_handler();
    let res = _do_ibc_destination_callback(
        contract_fn,
        env_ptr as *mut Region<Owned>,
        msg_ptr as *mut Region<Owned>,
    );
    let v = to_json_vec(&res).unwrap();
    Region::from_vec(v).to_heap_ptr() as u32
}

/// do_ibc2_packet_receive is designed for use with #[entry_point] to make a "C" extern
///
/// contract_fn is called when this chain receives an Ibc2 payload on port belonging
/// to this contract
///
/// - `Q`: custom query type (see QueryRequest)
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
#[cfg(feature = "ibc2")]
pub fn do_ibc2_packet_receive<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, Ibc2PacketReceiveMsg) -> Result<IbcReceiveResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    install_panic_handler();
    let res = _do_ibc2_packet_receive(
        contract_fn,
        env_ptr as *mut Region<Owned>,
        msg_ptr as *mut Region<Owned>,
    );
    let v = to_json_vec(&res).unwrap();
    Region::from_vec(v).to_heap_ptr() as u32
}

/// do_ibc2_packet_timeout is designed for use with #[entry_point] to make a "C" extern
///
/// contract_fn is called when a packet that this contract previously sent has provably
/// timed out and will never be relayed to the destination chain.
///
/// - `Q`: custom query type (see QueryRequest)
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
#[cfg(feature = "ibc2")]
pub fn do_ibc2_packet_timeout<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, Ibc2PacketTimeoutMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    install_panic_handler();
    let res = _do_ibc2_packet_timeout(
        contract_fn,
        env_ptr as *mut Region<Owned>,
        msg_ptr as *mut Region<Owned>,
    );
    let v = to_json_vec(&res).unwrap();
    Region::from_vec(v).to_heap_ptr() as u32
}

fn _do_instantiate<Q, M, C, E>(
    instantiate_fn: &dyn Fn(DepsMut<Q>, Env, MessageInfo, M) -> Result<Response<C>, E>,
    env_ptr: *mut Region<Owned>,
    info_ptr: *mut Region<Owned>,
    msg_ptr: *mut Region<Owned>,
) -> ContractResult<Response<C>>
where
    Q: CustomQuery,
    M: DeserializeOwned,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(env_ptr).unwrap()).into_vec() };
    let info: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(info_ptr).unwrap()).into_vec() };
    let msg: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(msg_ptr).unwrap()).into_vec() };

    let env: Env = try_into_contract_result!(from_json(env));
    let info: MessageInfo = try_into_contract_result!(from_json(info));
    let msg: M = try_into_contract_result!(from_json(msg));

    let mut deps = deps_from_imports();
    instantiate_fn(deps.as_mut(), env, info, msg).into()
}

fn _do_execute<Q, M, C, E>(
    execute_fn: &dyn Fn(DepsMut<Q>, Env, MessageInfo, M) -> Result<Response<C>, E>,
    env_ptr: *mut Region<Owned>,
    info_ptr: *mut Region<Owned>,
    msg_ptr: *mut Region<Owned>,
) -> ContractResult<Response<C>>
where
    Q: CustomQuery,
    M: DeserializeOwned,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(env_ptr).unwrap()).into_vec() };
    let info: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(info_ptr).unwrap()).into_vec() };
    let msg: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(msg_ptr).unwrap()).into_vec() };

    let env: Env = try_into_contract_result!(from_json(env));
    let info: MessageInfo = try_into_contract_result!(from_json(info));
    let msg: M = try_into_contract_result!(from_json(msg));

    let mut deps = deps_from_imports();
    execute_fn(deps.as_mut(), env, info, msg).into()
}

fn _do_migrate<Q, M, C, E>(
    migrate_fn: &dyn Fn(DepsMut<Q>, Env, M) -> Result<Response<C>, E>,
    env_ptr: *mut Region<Owned>,
    msg_ptr: *mut Region<Owned>,
) -> ContractResult<Response<C>>
where
    Q: CustomQuery,
    M: DeserializeOwned,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(env_ptr).unwrap()).into_vec() };
    let msg: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(msg_ptr).unwrap()).into_vec() };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: M = try_into_contract_result!(from_json(msg));

    let mut deps = deps_from_imports();
    migrate_fn(deps.as_mut(), env, msg).into()
}

fn _do_migrate_with_info<Q, M, C, E>(
    migrate_with_info_fn: &dyn Fn(DepsMut<Q>, Env, M, MigrateInfo) -> Result<Response<C>, E>,
    env_ptr: *mut Region<Owned>,
    msg_ptr: *mut Region<Owned>,
    migrate_info_ptr: *mut Region<Owned>,
) -> ContractResult<Response<C>>
where
    Q: CustomQuery,
    M: DeserializeOwned,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(env_ptr).unwrap()).into_vec() };
    let msg: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(msg_ptr).unwrap()).into_vec() };
    let migrate_info =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(migrate_info_ptr).unwrap()).into_vec() };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: M = try_into_contract_result!(from_json(msg));
    let migrate_info: MigrateInfo = try_into_contract_result!(from_json(migrate_info));

    let mut deps = deps_from_imports();
    migrate_with_info_fn(deps.as_mut(), env, msg, migrate_info).into()
}

fn _do_sudo<Q, M, C, E>(
    sudo_fn: &dyn Fn(DepsMut<Q>, Env, M) -> Result<Response<C>, E>,
    env_ptr: *mut Region<Owned>,
    msg_ptr: *mut Region<Owned>,
) -> ContractResult<Response<C>>
where
    Q: CustomQuery,
    M: DeserializeOwned,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(env_ptr).unwrap()).into_vec() };
    let msg: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(msg_ptr).unwrap()).into_vec() };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: M = try_into_contract_result!(from_json(msg));

    let mut deps = deps_from_imports();
    sudo_fn(deps.as_mut(), env, msg).into()
}

fn _do_reply<Q, C, E>(
    reply_fn: &dyn Fn(DepsMut<Q>, Env, Reply) -> Result<Response<C>, E>,
    env_ptr: *mut Region<Owned>,
    msg_ptr: *mut Region<Owned>,
) -> ContractResult<Response<C>>
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(env_ptr).unwrap()).into_vec() };
    let msg: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(msg_ptr).unwrap()).into_vec() };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: Reply = try_into_contract_result!(from_json(msg));

    let mut deps = deps_from_imports();
    reply_fn(deps.as_mut(), env, msg).into()
}

fn _do_query<Q, M, E>(
    query_fn: &dyn Fn(Deps<Q>, Env, M) -> Result<QueryResponse, E>,
    env_ptr: *mut Region<Owned>,
    msg_ptr: *mut Region<Owned>,
) -> ContractResult<QueryResponse>
where
    Q: CustomQuery,
    M: DeserializeOwned,
    E: ToString,
{
    let env: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(env_ptr).unwrap()).into_vec() };
    let msg: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(msg_ptr).unwrap()).into_vec() };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: M = try_into_contract_result!(from_json(msg));

    let deps = deps_from_imports();
    query_fn(deps.as_ref(), env, msg).into()
}

fn _do_ibc_channel_open<Q, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcChannelOpenMsg) -> Result<IbcChannelOpenResponse, E>,
    env_ptr: *mut Region<Owned>,
    msg_ptr: *mut Region<Owned>,
) -> ContractResult<IbcChannelOpenResponse>
where
    Q: CustomQuery,
    E: ToString,
{
    let env: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(env_ptr).unwrap()).into_vec() };
    let msg: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(msg_ptr).unwrap()).into_vec() };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: IbcChannelOpenMsg = try_into_contract_result!(from_json(msg));

    let mut deps = deps_from_imports();
    contract_fn(deps.as_mut(), env, msg).into()
}

#[cfg(feature = "stargate")]
fn _do_ibc_channel_connect<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcChannelConnectMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: *mut Region<Owned>,
    msg_ptr: *mut Region<Owned>,
) -> ContractResult<IbcBasicResponse<C>>
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(env_ptr).unwrap()).into_vec() };
    let msg: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(msg_ptr).unwrap()).into_vec() };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: IbcChannelConnectMsg = try_into_contract_result!(from_json(msg));

    let mut deps = deps_from_imports();
    contract_fn(deps.as_mut(), env, msg).into()
}

#[cfg(feature = "stargate")]
fn _do_ibc_channel_close<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcChannelCloseMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: *mut Region<Owned>,
    msg_ptr: *mut Region<Owned>,
) -> ContractResult<IbcBasicResponse<C>>
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(env_ptr).unwrap()).into_vec() };
    let msg: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(msg_ptr).unwrap()).into_vec() };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: IbcChannelCloseMsg = try_into_contract_result!(from_json(msg));

    let mut deps = deps_from_imports();
    contract_fn(deps.as_mut(), env, msg).into()
}

#[cfg(feature = "stargate")]
fn _do_ibc_packet_receive<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcPacketReceiveMsg) -> Result<IbcReceiveResponse<C>, E>,
    env_ptr: *mut Region<Owned>,
    msg_ptr: *mut Region<Owned>,
) -> ContractResult<IbcReceiveResponse<C>>
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(env_ptr).unwrap()).into_vec() };
    let msg: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(msg_ptr).unwrap()).into_vec() };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: IbcPacketReceiveMsg = try_into_contract_result!(from_json(msg));

    let mut deps = deps_from_imports();
    contract_fn(deps.as_mut(), env, msg).into()
}

#[cfg(feature = "stargate")]
fn _do_ibc_packet_ack<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcPacketAckMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: *mut Region<Owned>,
    msg_ptr: *mut Region<Owned>,
) -> ContractResult<IbcBasicResponse<C>>
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(env_ptr).unwrap()).into_vec() };
    let msg: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(msg_ptr).unwrap()).into_vec() };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: IbcPacketAckMsg = try_into_contract_result!(from_json(msg));

    let mut deps = deps_from_imports();
    contract_fn(deps.as_mut(), env, msg).into()
}

#[cfg(feature = "stargate")]
fn _do_ibc_packet_timeout<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcPacketTimeoutMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: *mut Region<Owned>,
    msg_ptr: *mut Region<Owned>,
) -> ContractResult<IbcBasicResponse<C>>
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(env_ptr).unwrap()).into_vec() };
    let msg: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(msg_ptr).unwrap()).into_vec() };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: IbcPacketTimeoutMsg = try_into_contract_result!(from_json(msg));

    let mut deps = deps_from_imports();
    contract_fn(deps.as_mut(), env, msg).into()
}

fn _do_ibc_source_callback<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcSourceCallbackMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: *mut Region<Owned>,
    msg_ptr: *mut Region<Owned>,
) -> ContractResult<IbcBasicResponse<C>>
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(env_ptr).unwrap()).into_vec() };
    let msg: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(msg_ptr).unwrap()).into_vec() };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: IbcSourceCallbackMsg = try_into_contract_result!(from_json(msg));

    let mut deps = deps_from_imports();
    contract_fn(deps.as_mut(), env, msg).into()
}

fn _do_ibc_destination_callback<Q, C, E>(
    contract_fn: &dyn Fn(
        DepsMut<Q>,
        Env,
        IbcDestinationCallbackMsg,
    ) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: *mut Region<Owned>,
    msg_ptr: *mut Region<Owned>,
) -> ContractResult<IbcBasicResponse<C>>
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(env_ptr).unwrap()).into_vec() };
    let msg: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(msg_ptr).unwrap()).into_vec() };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: IbcDestinationCallbackMsg = try_into_contract_result!(from_json(msg));

    let mut deps = deps_from_imports();
    contract_fn(deps.as_mut(), env, msg).into()
}

#[cfg(feature = "ibc2")]
fn _do_ibc2_packet_receive<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, Ibc2PacketReceiveMsg) -> Result<IbcReceiveResponse<C>, E>,
    env_ptr: *mut Region<Owned>,
    msg_ptr: *mut Region<Owned>,
) -> ContractResult<IbcReceiveResponse<C>>
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(env_ptr).unwrap()).into_vec() };
    let msg: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(msg_ptr).unwrap()).into_vec() };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: Ibc2PacketReceiveMsg = try_into_contract_result!(from_json(msg));

    let mut deps = deps_from_imports();
    contract_fn(deps.as_mut(), env, msg).into()
}

#[cfg(feature = "ibc2")]
fn _do_ibc2_packet_timeout<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, Ibc2PacketTimeoutMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: *mut Region<Owned>,
    msg_ptr: *mut Region<Owned>,
) -> ContractResult<IbcBasicResponse<C>>
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(env_ptr).unwrap()).into_vec() };
    let msg: Vec<u8> =
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(msg_ptr).unwrap()).into_vec() };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: Ibc2PacketTimeoutMsg = try_into_contract_result!(from_json(msg));

    let mut deps = deps_from_imports();
    contract_fn(deps.as_mut(), env, msg).into()
}

/// Makes all bridges to external dependencies (i.e. Wasm imports) that are injected by the VM
fn deps_from_imports<Q>() -> OwnedDeps<ExternalStorage, ExternalApi, ExternalQuerier, Q>
where
    Q: CustomQuery,
{
    OwnedDeps {
        storage: ExternalStorage::new(),
        api: ExternalApi::new(),
        querier: ExternalQuerier::new(),
        custom_query_type: PhantomData,
    }
}
