use serde::de::DeserializeOwned;
use wasmer::Value;

use cosmwasm_std::{
    ContractResult, CustomMsg, Env, IbcBasicResponse, IbcDestinationCallbackMsg,
    IbcSourceCallbackMsg, MessageInfo, MigrateInfo, QueryResponse, Reply, Response,
};
#[cfg(feature = "stargate")]
use cosmwasm_std::{
    Ibc3ChannelOpenResponse, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg,
    IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse,
};

#[cfg(feature = "ibc2")]
use cosmwasm_std::Ibc2PacketReceiveMsg;

use crate::backend::{BackendApi, Querier, Storage};
use crate::conversion::ref_to_u32;
use crate::errors::{VmError, VmResult};
use crate::instance::Instance;
use crate::serde::{from_slice, to_vec};

/// The limits in here protect the host from allocating an unreasonable amount of memory
/// and copying an unreasonable amount of data.
///
/// A JSON deserializer would want to set the limit to a much smaller value because
/// deserializing JSON is more expensive. As a consequence, any sane contract should hit
/// the deserializer limit before the read limit.
mod read_limits {
    /// A mebi (mega binary)
    const MI: usize = 1024 * 1024;
    /// Max length (in bytes) of the result data from an instantiate call.
    pub const RESULT_INSTANTIATE: usize = 64 * MI;
    /// Max length (in bytes) of the result data from an execute call.
    pub const RESULT_EXECUTE: usize = 64 * MI;
    /// Max length (in bytes) of the result data from a migrate call.
    pub const RESULT_MIGRATE: usize = 64 * MI;
    /// Max length (in bytes) of the result data from a sudo call.
    pub const RESULT_SUDO: usize = 64 * MI;
    /// Max length (in bytes) of the result data from a reply call.
    pub const RESULT_REPLY: usize = 64 * MI;
    /// Max length (in bytes) of the result data from a query call.
    pub const RESULT_QUERY: usize = 64 * MI;
    /// Max length (in bytes) of the result data from a ibc_channel_open call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_CHANNEL_OPEN: usize = 64 * MI;
    /// Max length (in bytes) of the result data from a ibc_channel_connect call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_CHANNEL_CONNECT: usize = 64 * MI;
    /// Max length (in bytes) of the result data from a ibc_channel_close call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_CHANNEL_CLOSE: usize = 64 * MI;
    /// Max length (in bytes) of the result data from a ibc_packet_receive call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_PACKET_RECEIVE: usize = 64 * MI;
    /// Max length (in bytes) of the result data from a ibc_packet_ack call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_PACKET_ACK: usize = 64 * MI;
    /// Max length (in bytes) of the result data from a ibc_packet_timeout call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_PACKET_TIMEOUT: usize = 64 * MI;
    /// Max length (in bytes) of the result data from a ibc_source_callback call.
    pub const RESULT_IBC_SOURCE_CALLBACK: usize = 64 * MI;
    /// Max length (in bytes) of the result data from a ibc_destination_callback call.
    pub const RESULT_IBC_DESTINATION_CALLBACK: usize = 64 * MI;
}

/// The limits for the JSON deserialization.
///
/// Those limits are not used when the Rust JSON deserializer is bypassed by using the
/// public `call_*_raw` functions directly.
mod deserialization_limits {
    /// A kibi (kilo binary)
    const KI: usize = 1024;
    /// Max length (in bytes) of the result data from an instantiate call.
    pub const RESULT_INSTANTIATE: usize = 256 * KI;
    /// Max length (in bytes) of the result data from an execute call.
    pub const RESULT_EXECUTE: usize = 256 * KI;
    /// Max length (in bytes) of the result data from a migrate call.
    pub const RESULT_MIGRATE: usize = 256 * KI;
    /// Max length (in bytes) of the result data from a sudo call.
    pub const RESULT_SUDO: usize = 256 * KI;
    /// Max length (in bytes) of the result data from a reply call.
    pub const RESULT_REPLY: usize = 256 * KI;
    /// Max length (in bytes) of the result data from a query call.
    pub const RESULT_QUERY: usize = 256 * KI;
    /// Max length (in bytes) of the result data from a ibc_channel_open call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_CHANNEL_OPEN: usize = 256 * KI;
    /// Max length (in bytes) of the result data from a ibc_channel_connect call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_CHANNEL_CONNECT: usize = 256 * KI;
    /// Max length (in bytes) of the result data from a ibc_channel_close call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_CHANNEL_CLOSE: usize = 256 * KI;
    /// Max length (in bytes) of the result data from a ibc_packet_receive call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_PACKET_RECEIVE: usize = 256 * KI;
    /// Max length (in bytes) of the result data from a ibc_packet_ack call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_PACKET_ACK: usize = 256 * KI;
    /// Max length (in bytes) of the result data from a ibc_packet_timeout call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_PACKET_TIMEOUT: usize = 256 * KI;
    /// Max length (in bytes) of the result data from a ibc_source_callback call.
    pub const RESULT_IBC_SOURCE_CALLBACK: usize = 256 * KI;
    /// Max length (in bytes) of the result data from a ibc_destination_callback call.
    pub const RESULT_IBC_DESTINATION_CALLBACK: usize = 256 * KI;
}

pub fn call_instantiate<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    info: &MessageInfo,
    msg: &[u8],
) -> VmResult<ContractResult<Response<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let info = to_vec(info)?;
    let data = call_instantiate_raw(instance, &env, &info, msg)?;
    let result: ContractResult<Response<U>> =
        from_slice(&data, deserialization_limits::RESULT_INSTANTIATE)?;
    Ok(result)
}

pub fn call_execute<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    info: &MessageInfo,
    msg: &[u8],
) -> VmResult<ContractResult<Response<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let info = to_vec(info)?;
    let data = call_execute_raw(instance, &env, &info, msg)?;
    let result: ContractResult<Response<U>> =
        from_slice(&data, deserialization_limits::RESULT_EXECUTE)?;
    Ok(result)
}

pub fn call_migrate<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &[u8],
) -> VmResult<ContractResult<Response<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let data = call_migrate_raw(instance, &env, msg)?;
    let result: ContractResult<Response<U>> =
        from_slice(&data, deserialization_limits::RESULT_MIGRATE)?;
    Ok(result)
}

pub fn call_migrate_with_info<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &[u8],
    migrate_info: &MigrateInfo,
) -> VmResult<ContractResult<Response<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let migrate_info = to_vec(migrate_info)?;
    let data = call_migrate_with_info_raw(instance, &env, msg, &migrate_info)?;
    let result: ContractResult<Response<U>> =
        from_slice(&data, deserialization_limits::RESULT_MIGRATE)?;
    Ok(result)
}

pub fn call_sudo<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &[u8],
) -> VmResult<ContractResult<Response<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let data = call_sudo_raw(instance, &env, msg)?;
    let result: ContractResult<Response<U>> =
        from_slice(&data, deserialization_limits::RESULT_SUDO)?;
    Ok(result)
}

pub fn call_reply<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &Reply,
) -> VmResult<ContractResult<Response<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let msg = to_vec(msg)?;
    let data = call_reply_raw(instance, &env, &msg)?;
    let result: ContractResult<Response<U>> =
        from_slice(&data, deserialization_limits::RESULT_REPLY)?;
    Ok(result)
}

pub fn call_query<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &[u8],
) -> VmResult<ContractResult<QueryResponse>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    let env = to_vec(env)?;
    let data = call_query_raw(instance, &env, msg)?;
    let result: ContractResult<QueryResponse> =
        from_slice(&data, deserialization_limits::RESULT_QUERY)?;
    // Ensure query response is valid JSON
    if let ContractResult::Ok(binary_response) = &result {
        serde_json::from_slice::<serde_json::Value>(binary_response.as_slice())
            .map_err(|e| VmError::generic_err(format!("Query response must be valid JSON. {e}")))?;
    }

    Ok(result)
}

#[cfg(feature = "stargate")]
pub fn call_ibc_channel_open<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &IbcChannelOpenMsg,
) -> VmResult<ContractResult<Option<Ibc3ChannelOpenResponse>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    let env = to_vec(env)?;
    let msg = to_vec(msg)?;
    let data = call_ibc_channel_open_raw(instance, &env, &msg)?;
    let result: ContractResult<Option<Ibc3ChannelOpenResponse>> =
        from_slice(&data, deserialization_limits::RESULT_IBC_CHANNEL_OPEN)?;
    Ok(result)
}

#[cfg(feature = "stargate")]
pub fn call_ibc_channel_connect<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &IbcChannelConnectMsg,
) -> VmResult<ContractResult<IbcBasicResponse<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let msg = to_vec(msg)?;
    let data = call_ibc_channel_connect_raw(instance, &env, &msg)?;
    let result = from_slice(&data, deserialization_limits::RESULT_IBC_CHANNEL_CONNECT)?;
    Ok(result)
}

#[cfg(feature = "stargate")]
pub fn call_ibc_channel_close<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &IbcChannelCloseMsg,
) -> VmResult<ContractResult<IbcBasicResponse<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let msg = to_vec(msg)?;
    let data = call_ibc_channel_close_raw(instance, &env, &msg)?;
    let result = from_slice(&data, deserialization_limits::RESULT_IBC_CHANNEL_CLOSE)?;
    Ok(result)
}

#[cfg(feature = "stargate")]
pub fn call_ibc_packet_receive<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &IbcPacketReceiveMsg,
) -> VmResult<ContractResult<IbcReceiveResponse<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let msg = to_vec(msg)?;
    let data = call_ibc_packet_receive_raw(instance, &env, &msg)?;
    let result = from_slice(&data, deserialization_limits::RESULT_IBC_PACKET_RECEIVE)?;
    Ok(result)
}

#[cfg(feature = "stargate")]
pub fn call_ibc_packet_ack<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &IbcPacketAckMsg,
) -> VmResult<ContractResult<IbcBasicResponse<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let msg = to_vec(msg)?;
    let data = call_ibc_packet_ack_raw(instance, &env, &msg)?;
    let result = from_slice(&data, deserialization_limits::RESULT_IBC_PACKET_ACK)?;
    Ok(result)
}

#[cfg(feature = "stargate")]
pub fn call_ibc_packet_timeout<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &IbcPacketTimeoutMsg,
) -> VmResult<ContractResult<IbcBasicResponse<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let msg = to_vec(msg)?;
    let data = call_ibc_packet_timeout_raw(instance, &env, &msg)?;
    let result = from_slice(&data, deserialization_limits::RESULT_IBC_PACKET_TIMEOUT)?;
    Ok(result)
}

pub fn call_ibc_source_callback<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &IbcSourceCallbackMsg,
) -> VmResult<ContractResult<IbcBasicResponse<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let msg = to_vec(msg)?;
    let data = call_ibc_source_callback_raw(instance, &env, &msg)?;
    let result = from_slice(&data, deserialization_limits::RESULT_IBC_SOURCE_CALLBACK)?;
    Ok(result)
}

pub fn call_ibc_destination_callback<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &IbcDestinationCallbackMsg,
) -> VmResult<ContractResult<IbcBasicResponse<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let msg = to_vec(msg)?;
    let data = call_ibc_destination_callback_raw(instance, &env, &msg)?;
    let result = from_slice(
        &data,
        deserialization_limits::RESULT_IBC_DESTINATION_CALLBACK,
    )?;
    Ok(result)
}

/// Calls Wasm export "instantiate" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_instantiate_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    info: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "instantiate",
        &[env, info, msg],
        read_limits::RESULT_INSTANTIATE,
    )
}

/// Calls Wasm export "execute" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_execute_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    info: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "execute",
        &[env, info, msg],
        read_limits::RESULT_EXECUTE,
    )
}

/// Calls Wasm export "migrate" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_migrate_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "migrate",
        &[env, msg],
        read_limits::RESULT_MIGRATE,
    )
}

/// Calls Wasm export "migrate" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
/// The difference between this function and [call_migrate_raw] is the
/// additional argument - `migrate_info`. It contains additional data
/// related to the contract's migration procedure.
///
/// It is safe to call this method instead of [call_migrate_raw] even
/// if a contract contains the migrate entrypoint without `migrate_info`.
/// In such case this structure is omitted.
pub fn call_migrate_with_info_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
    migrate_info: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "migrate",
        &[env, msg, migrate_info],
        read_limits::RESULT_MIGRATE,
    )
    .or_else(|err| {
        if matches!(err, VmError::FunctionArityMismatch { .. }) {
            call_raw(
                instance,
                "migrate",
                &[env, msg],
                read_limits::RESULT_MIGRATE,
            )
        } else {
            Err(err)
        }
    })
}

/// Calls Wasm export "sudo" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_sudo_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(instance, "sudo", &[env, msg], read_limits::RESULT_SUDO)
}

/// Calls Wasm export "reply" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_reply_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(instance, "reply", &[env, msg], read_limits::RESULT_REPLY)
}

/// Calls Wasm export "query" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_query_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(true);
    call_raw(instance, "query", &[env, msg], read_limits::RESULT_QUERY)
}

#[cfg(feature = "stargate")]
pub fn call_ibc_channel_open_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "ibc_channel_open",
        &[env, msg],
        read_limits::RESULT_IBC_CHANNEL_OPEN,
    )
}

#[cfg(feature = "stargate")]
pub fn call_ibc_channel_connect_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "ibc_channel_connect",
        &[env, msg],
        read_limits::RESULT_IBC_CHANNEL_CONNECT,
    )
}

#[cfg(feature = "stargate")]
pub fn call_ibc_channel_close_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "ibc_channel_close",
        &[env, msg],
        read_limits::RESULT_IBC_CHANNEL_CLOSE,
    )
}

#[cfg(feature = "stargate")]
pub fn call_ibc_packet_receive_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "ibc_packet_receive",
        &[env, msg],
        read_limits::RESULT_IBC_PACKET_RECEIVE,
    )
}

#[cfg(feature = "stargate")]
pub fn call_ibc_packet_ack_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "ibc_packet_ack",
        &[env, msg],
        read_limits::RESULT_IBC_PACKET_ACK,
    )
}

#[cfg(feature = "stargate")]
pub fn call_ibc_packet_timeout_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "ibc_packet_timeout",
        &[env, msg],
        read_limits::RESULT_IBC_PACKET_TIMEOUT,
    )
}

#[cfg(feature = "ibc2")]
pub fn call_ibc2_packet_receive_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "ibc2_packet_receive",
        &[env, msg],
        read_limits::RESULT_IBC_PACKET_RECEIVE,
    )
}

pub fn call_ibc_source_callback_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "ibc_source_callback",
        &[env, msg],
        read_limits::RESULT_IBC_SOURCE_CALLBACK,
    )
}

pub fn call_ibc_destination_callback_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "ibc_destination_callback",
        &[env, msg],
        read_limits::RESULT_IBC_DESTINATION_CALLBACK,
    )
}

/// Calls a function with the given arguments.
/// The exported function must return exactly one result (an offset to the result Region).
pub(crate) fn call_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    name: &str,
    args: &[&[u8]],
    result_max_length: usize,
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    let mut arg_region_ptrs = Vec::<Value>::with_capacity(args.len());
    for arg in args {
        let region_ptr = instance.allocate(arg.len())?;
        instance.write_memory(region_ptr, arg)?;
        arg_region_ptrs.push(region_ptr.into());
    }
    let result = instance.call_function1(name, &arg_region_ptrs)?;
    let res_region_ptr = ref_to_u32(&result)?;
    let data = instance.read_memory(res_region_ptr, result_max_length)?;
    // free return value in wasm (arguments were freed in wasm code)
    instance.deallocate(res_region_ptr)?;
    Ok(data)
}

#[cfg(feature = "ibc2")]
pub fn call_ibc2_packet_receive<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &Ibc2PacketReceiveMsg,
) -> VmResult<ContractResult<IbcReceiveResponse<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let msg = to_vec(msg)?;
    let data = call_ibc2_packet_receive_raw(instance, &env, &msg)?;
    let result = from_slice(&data, deserialization_limits::RESULT_IBC_PACKET_RECEIVE)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{
        mock_env, mock_info, mock_instance, mock_instance_with_options, MockInstanceOptions,
    };
    use cosmwasm_std::{coins, from_json, to_json_string, Addr, Empty};
    use sha2::{Digest, Sha256};

    static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");
    static CYBERPUNK: &[u8] = include_bytes!("../testdata/cyberpunk.wasm");
    static FLOATY2: &[u8] = include_bytes!("../testdata/floaty_2.0.wasm");
    static EMPTY: &[u8] = include_bytes!("../testdata/empty.wasm");

    #[test]
    fn call_instantiate_works() {
        let mut instance = mock_instance(CONTRACT, &[]);

        // init
        let info = mock_info(&instance.api().addr_make("creator"), &coins(1000, "earth"));
        let verifier = instance.api().addr_make("verifies");
        let beneficiary = instance.api().addr_make("benefits");
        let msg = format!(r#"{{"verifier": "{verifier}", "beneficiary": "{beneficiary}"}}"#);
        call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg.as_bytes())
            .unwrap()
            .unwrap();
    }

    #[test]
    fn call_instantiate_handles_missing_export() {
        let mut deps = mock_instance(EMPTY, &[]);

        let msg = Empty {};
        let info = mock_info("creator", &coins(1000, "earth"));

        let serialized_msg = to_vec(&msg).unwrap();
        let err =
            call_instantiate::<_, _, _, Empty>(&mut deps, &mock_env(), &info, &serialized_msg)
                .unwrap_err();

        assert!(matches!(
            err,
            VmError::ResolveErr {
                msg,
                ..
            }
            if msg == "Could not get export: Missing export instantiate"
        ));
    }

    #[test]
    fn call_execute_works() {
        let mut instance = mock_instance(CONTRACT, &[]);

        // init
        let info = mock_info(&instance.api().addr_make("creator"), &coins(1000, "earth"));
        let verifier = instance.api().addr_make("verifies");
        let beneficiary = instance.api().addr_make("benefits");
        let msg = format!(r#"{{"verifier": "{verifier}", "beneficiary": "{beneficiary}"}}"#);
        call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg.as_bytes())
            .unwrap()
            .unwrap();

        // execute
        let info = mock_info(&verifier, &coins(15, "earth"));
        let msg = br#"{"release":{}}"#;
        call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();
    }

    #[test]
    fn call_execute_runs_out_of_gas() {
        let mut instance = mock_instance(CYBERPUNK, &[]);

        // init
        let info = mock_info("creator", &[]);
        call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, br#"{}"#)
            .unwrap()
            .unwrap();

        // execute
        let info = mock_info("looper", &[]);
        let msg = br#"{"cpu_loop":{}}"#;
        let err =
            call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap_err();
        assert!(matches!(err, VmError::GasDepletion { .. }));
    }

    #[test]
    fn call_execute_handles_panic() {
        let mut instance = mock_instance(CYBERPUNK, &[]);

        let info = mock_info("creator", &[]);
        call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, br#"{}"#)
            .unwrap()
            .unwrap();

        // execute
        let info = mock_info("troll", &[]);
        let msg = br#"{"panic":{}}"#;
        let err =
            call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap_err();
        match err {
            VmError::RuntimeErr { msg, .. } => {
                assert!(
                    msg.contains("RuntimeError: Aborted: panicked at src/contract.rs:"),
                    "Unexpected error msg: {msg}"
                )
            }
            err => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn call_execute_handles_unreachable() {
        let mut instance = mock_instance(CYBERPUNK, &[]);

        let info = mock_info("creator", &[]);
        call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, br#"{}"#)
            .unwrap()
            .unwrap();

        // execute
        let info = mock_info("troll", &[]);
        let msg = br#"{"unreachable":{}}"#;
        let err =
            call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap_err();
        match err {
            VmError::RuntimeErr { msg, .. } => {
                assert!(msg.contains("RuntimeError: unreachable"))
            }
            err => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn call_migrate_works() {
        let mut instance = mock_instance(CONTRACT, &[]);

        // init
        let info = mock_info(&instance.api().addr_make("creator"), &coins(1000, "earth"));
        let verifier = instance.api().addr_make("verifies");
        let beneficiary = instance.api().addr_make("benefits");
        let msg = format!(r#"{{"verifier": "{verifier}", "beneficiary": "{beneficiary}"}}"#);
        call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg.as_bytes())
            .unwrap()
            .unwrap();

        // change the verifier via migrate
        let someone_else = instance.api().addr_make("someone else");
        let msg = format!(r#"{{"verifier": "{someone_else}"}}"#);
        let _res = call_migrate::<_, _, _, Empty>(&mut instance, &mock_env(), msg.as_bytes())
            .unwrap()
            .unwrap();

        // query the new_verifier with verifier
        let msg = br#"{"verifier":{}}"#;
        let contract_result = call_query(&mut instance, &mock_env(), msg).unwrap();
        let query_response = contract_result.unwrap();
        assert_eq!(
            query_response,
            format!(r#"{{"verifier":"{}"}}"#, someone_else).as_bytes(),
        );
    }

    #[test]
    fn call_migrate_with_info_works() {
        let mut instance = mock_instance(CONTRACT, &[]);

        // init
        let info = mock_info(&instance.api().addr_make("creator"), &coins(1000, "earth"));
        let verifier = instance.api().addr_make("verifies");
        let beneficiary = instance.api().addr_make("benefits");
        let msg = format!(r#"{{"verifier": "{verifier}", "beneficiary": "{beneficiary}"}}"#);
        call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg.as_bytes())
            .unwrap()
            .unwrap();

        // change the verifier via migrate
        let someone_else = instance.api().addr_make("someone else");
        let msg = format!(r#"{{"verifier": "{someone_else}"}}"#);
        let migrate_info = MigrateInfo {
            sender: Addr::unchecked(someone_else.clone()),
            old_migrate_version: Some(33),
        };
        let _res = call_migrate_with_info::<_, _, _, Empty>(
            &mut instance,
            &mock_env(),
            msg.as_bytes(),
            &migrate_info,
        )
        .unwrap()
        .unwrap();

        // query the new_verifier with verifier
        let msg = br#"{"verifier":{}}"#;
        let contract_result = call_query(&mut instance, &mock_env(), msg).unwrap();
        let query_response = contract_result.unwrap();
        assert_eq!(
            query_response,
            format!(r#"{{"verifier":"{}"}}"#, someone_else).as_bytes(),
        );
    }

    #[test]
    fn call_query_works() {
        let mut instance = mock_instance(CONTRACT, &[]);

        // init
        let info = mock_info(&instance.api().addr_make("creator"), &coins(1000, "earth"));
        let verifier = instance.api().addr_make("verifies");
        let beneficiary = instance.api().addr_make("benefits");
        let msg = format!(r#"{{"verifier": "{verifier}", "beneficiary": "{beneficiary}"}}"#);
        call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg.as_bytes())
            .unwrap()
            .unwrap();

        // query
        let msg = br#"{"verifier":{}}"#;
        let contract_result = call_query(&mut instance, &mock_env(), msg).unwrap();
        let query_response = contract_result.unwrap();
        assert_eq!(
            query_response,
            format!("{{\"verifier\":\"{verifier}\"}}").as_bytes()
        );
    }

    #[test]
    fn float_instrs_are_deterministic() {
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum Value {
            U32(u32),
            U64(u64),
            F32(u32),
            F64(u64),
        }

        let mut instance = mock_instance_with_options(
            FLOATY2,
            MockInstanceOptions {
                gas_limit: u64::MAX,
                memory_limit: None,
                ..Default::default()
            },
        );

        // init
        let info = mock_info("creator", &[]);
        call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, br#"{}"#)
            .unwrap()
            .unwrap();

        // query instructions
        let msg = br#"{"instructions":{}}"#;
        let contract_result = call_query(&mut instance, &mock_env(), msg)
            .unwrap()
            .unwrap();
        let instructions: Vec<String> = from_json(contract_result).unwrap();
        // little sanity check
        assert_eq!(instructions.len(), 70);

        const RUNS_PER_INSTRUCTION: u64 = 150;
        let mut hasher = Sha256::new();
        for instr in &instructions {
            for seed in 0..RUNS_PER_INSTRUCTION {
                // query some input values for the instruction
                let args: Vec<Value> = from_json(
                    call_query(
                        &mut instance,
                        &mock_env(),
                        format!(
                            r#"{{"random_args_for":{{ "instruction": "{instr}", "seed": {seed}}}}}"#
                        )
                        .as_bytes(),
                    )
                    .unwrap()
                    .unwrap(),
                )
                .unwrap();

                // build the run message
                let args = to_json_string(&args).unwrap();
                let msg: String = format!(
                    r#"{{"run":{{
                        "instruction": "{instr}",
                        "args": {args}
                    }}}}"#
                );
                // run the instruction
                // this might throw a runtime error (e.g. if the instruction traps)
                let result = match call_query(&mut instance, &mock_env(), msg.as_bytes()) {
                    Ok(ContractResult::Ok(r)) => format!("{:?}", from_json::<Value>(&r).unwrap()),
                    Err(VmError::RuntimeErr { msg, .. }) => msg,
                    e => panic!("unexpected error: {e:?}"),
                };
                // add the result to the hash
                hasher.update(format!("{instr}{seed}{result}").as_bytes());
            }
        }
        let hash = Digest::finalize(hasher);
        assert_eq!(
            hex::encode(hash.as_slice()),
            "95f70fa6451176ab04a9594417a047a1e4d8e2ff809609b8f81099496bee2393"
        );
    }

    #[cfg(feature = "stargate")]
    mod ibc {
        use super::*;
        use crate::testing::{MockApi, MockQuerier, MockStorage};
        use cosmwasm_std::testing::mock_ibc_packet_timeout;
        use cosmwasm_std::testing::{
            mock_ibc_channel_close_init, mock_ibc_channel_connect_ack, mock_ibc_channel_open_init,
            mock_ibc_packet_ack, mock_ibc_packet_recv, mock_wasmd_attr,
        };
        use cosmwasm_std::{
            Event, IbcAckCallbackMsg, IbcAcknowledgement, IbcOrder, IbcTimeoutCallbackMsg, ReplyOn,
            SubMsgResponse, SubMsgResult,
        };
        const CONTRACT: &[u8] = include_bytes!("../testdata/ibc_reflect.wasm");
        const IBC_CALLBACKS: &[u8] = include_bytes!("../testdata/ibc_callbacks.wasm");
        const IBC_VERSION: &str = "ibc-reflect-v1";

        fn setup(
            instance: &mut Instance<MockApi, MockStorage, MockQuerier>,
            channel_id: &str,
            account: &str,
        ) {
            // init
            let info = mock_info("creator", &[]);
            let msg = br#"{"reflect_code_id":77}"#;
            call_instantiate::<_, _, _, Empty>(instance, &mock_env(), &info, msg)
                .unwrap()
                .unwrap();
            // first we try to open with a valid handshake
            let handshake_open =
                mock_ibc_channel_open_init(channel_id, IbcOrder::Ordered, IBC_VERSION);
            call_ibc_channel_open(instance, &mock_env(), &handshake_open)
                .unwrap()
                .unwrap();
            // then we connect (with counter-party version set)
            let handshake_connect =
                mock_ibc_channel_connect_ack(channel_id, IbcOrder::Ordered, IBC_VERSION);
            let res: IbcBasicResponse = call_ibc_channel_connect::<_, _, _, Empty>(
                instance,
                &mock_env(),
                &handshake_connect,
            )
            .unwrap()
            .unwrap();
            assert_eq!(1, res.messages.len());
            assert_eq!(
                res.events,
                [Event::new("ibc").add_attribute("channel", "connect")]
            );
            assert_eq!(ReplyOn::Success, res.messages[0].reply_on);
            let id = res.messages[0].id;
            let payload = res.messages[0].payload.clone();
            let event = Event::new("instantiate").add_attributes(vec![
                // We have to force this one to avoid the debug assertion against _
                mock_wasmd_attr("_contract_address", account),
            ]);
            // which creates a reflect account. here we get the callback
            #[allow(deprecated)]
            let response = Reply {
                id,
                payload,
                gas_used: 1234567,
                result: SubMsgResult::Ok(SubMsgResponse {
                    events: vec![event],
                    msg_responses: vec![],
                    data: None,
                }),
            };
            call_reply::<_, _, _, Empty>(instance, &mock_env(), &response).unwrap();
        }

        const CHANNEL_ID: &str = "channel-123";
        const ACCOUNT: &str = "account-456";

        #[test]
        fn call_ibc_channel_open_and_connect_works() {
            let mut instance = mock_instance(CONTRACT, &[]);
            setup(&mut instance, CHANNEL_ID, ACCOUNT);
        }

        #[test]
        fn call_ibc_channel_close_works() {
            let mut instance = mock_instance(CONTRACT, &[]);
            let account = instance.api().addr_make(ACCOUNT);
            setup(&mut instance, CHANNEL_ID, &account);
            let handshake_close =
                mock_ibc_channel_close_init(CHANNEL_ID, IbcOrder::Ordered, IBC_VERSION);
            call_ibc_channel_close::<_, _, _, Empty>(&mut instance, &mock_env(), &handshake_close)
                .unwrap()
                .unwrap();
        }

        #[test]
        fn call_ibc_packet_ack_works() {
            let mut instance = mock_instance(CONTRACT, &[]);
            setup(&mut instance, CHANNEL_ID, ACCOUNT);
            let ack = IbcAcknowledgement::new(br#"{}"#);
            let msg = mock_ibc_packet_ack(CHANNEL_ID, br#"{}"#, ack).unwrap();
            call_ibc_packet_ack::<_, _, _, Empty>(&mut instance, &mock_env(), &msg)
                .unwrap()
                .unwrap();
        }

        #[test]
        fn call_ibc_packet_timeout_works() {
            let mut instance = mock_instance(CONTRACT, &[]);
            setup(&mut instance, CHANNEL_ID, ACCOUNT);
            let msg = mock_ibc_packet_timeout(CHANNEL_ID, br#"{}"#).unwrap();
            call_ibc_packet_timeout::<_, _, _, Empty>(&mut instance, &mock_env(), &msg)
                .unwrap()
                .unwrap();
        }

        #[test]
        fn call_ibc_packet_receive_works() {
            let mut instance = mock_instance(CONTRACT, &[]);
            setup(&mut instance, CHANNEL_ID, ACCOUNT);
            let who_am_i = br#"{"who_am_i":{}}"#;
            let msg = mock_ibc_packet_recv(CHANNEL_ID, who_am_i).unwrap();
            call_ibc_packet_receive::<_, _, _, Empty>(&mut instance, &mock_env(), &msg)
                .unwrap()
                .unwrap();
        }

        #[test]
        fn call_ibc_source_callback_works() {
            let mut instance = mock_instance(IBC_CALLBACKS, &[]);

            // init
            let creator = instance.api().addr_make("creator");
            let info = mock_info(&creator, &[]);
            call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, br#"{}"#)
                .unwrap()
                .unwrap();

            /// Response type for the `callback_stats` query
            #[derive(serde::Serialize, serde::Deserialize)]
            struct CallbackStats {
                pub ibc_ack_callbacks: Vec<IbcPacketAckMsg>,
                pub ibc_timeout_callbacks: Vec<IbcPacketTimeoutMsg>,
            }

            // send ack callback
            let ack = mock_ibc_packet_ack(CHANNEL_ID, br#"{}"#, IbcAcknowledgement::new(br#"{}"#))
                .unwrap();
            let msg = IbcSourceCallbackMsg::Acknowledgement(IbcAckCallbackMsg::new(
                ack.acknowledgement,
                ack.original_packet,
                ack.relayer,
            ));
            call_ibc_source_callback::<_, _, _, Empty>(&mut instance, &mock_env(), &msg)
                .unwrap()
                .unwrap();
            // query the CallbackStats
            let stats: CallbackStats = serde_json::from_slice(
                &call_query::<_, _, _>(&mut instance, &mock_env(), br#"{"callback_stats":{}}"#)
                    .unwrap()
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(1, stats.ibc_ack_callbacks.len());
            assert_eq!(0, stats.ibc_timeout_callbacks.len());

            // send timeout callback
            let timeout = mock_ibc_packet_timeout(CHANNEL_ID, br#"{}"#).unwrap();
            let msg = IbcSourceCallbackMsg::Timeout(IbcTimeoutCallbackMsg::new(
                timeout.packet,
                timeout.relayer,
            ));
            call_ibc_source_callback::<_, _, _, Empty>(&mut instance, &mock_env(), &msg)
                .unwrap()
                .unwrap();
            // query the CallbackStats
            let stats: CallbackStats = serde_json::from_slice(
                &call_query::<_, _, _>(&mut instance, &mock_env(), br#"{"callback_stats":{}}"#)
                    .unwrap()
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(1, stats.ibc_ack_callbacks.len());
            assert_eq!(1, stats.ibc_timeout_callbacks.len());
        }
    }

    #[cfg(feature = "ibc2")]
    mod ibc2 {
        use super::*;
        use cosmwasm_std::testing::mock_ibc2_packet_recv;
        const CONTRACT: &[u8] = include_bytes!("../testdata/ibcv2.wasm");

        #[test]
        fn call_ibc2_packet_receive_works() {
            // init
            let mut instance = mock_instance(CONTRACT, &[]);
            let info = mock_info("creator", &[]);
            let instantiate_msg = br#"{}"#;
            call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, instantiate_msg)
                .unwrap()
                .unwrap();

            let ibc2_msg = br#"SomeRandomMsg"#;
            let ibc2_msg = mock_ibc2_packet_recv(ibc2_msg).unwrap();
            call_ibc2_packet_receive::<_, _, _, Empty>(&mut instance, &mock_env(), &ibc2_msg)
                .unwrap()
                .unwrap();
        }
    }
}
