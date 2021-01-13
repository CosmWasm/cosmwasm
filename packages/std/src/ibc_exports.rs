#![cfg(all(feature = "stargate", target_arch = "wasm32"))]
use std::fmt;
use std::vec::Vec;

use schemars::JsonSchema;
use serde::Serialize;

use crate::deps::DepsMut;
use crate::exports::make_dependencies;
use crate::ibc::{IbcBasicResponse, IbcChannel};
use crate::memory::{consume_region, release_buffer, Region};
use crate::results::ContractResult;
use crate::serde::{from_slice, to_vec};
use crate::types::Env;
use crate::{IbcAcknowledgement, IbcPacket, IbcReceiveResponse};

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

/// do_ibc_channel_open is designed for use with #[entry_point] to make a "C" extern
///
/// ibc_open_fn does the protocol version negotiation during channel handshake phase
pub fn do_ibc_channel_open<E>(
    ibc_open_fn: &dyn Fn(DepsMut, Env, IbcChannel) -> Result<(), E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    E: ToString,
{
    let res = _do_ibc_channel_open(ibc_open_fn, env_ptr as *mut Region, msg_ptr as *mut Region);
    let v = to_vec(&res).unwrap();
    release_buffer(v) as u32
}

fn _do_ibc_channel_open<E>(
    ibc_open_fn: &dyn Fn(DepsMut, Env, IbcChannel) -> Result<(), E>,
    env_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<bool>
where
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_slice(&env));
    let msg: IbcChannel = try_into_contract_result!(from_slice(&msg));

    let mut deps = make_dependencies();
    match ibc_open_fn(deps.as_mut(), env, msg) {
        Ok(_) => ContractResult::Ok(true),
        Err(e) => ContractResult::Err(e.to_string()),
    }
}

/// do_ibc_channel_connect is designed for use with #[entry_point] to make a "C" extern
///
/// ibc_connect_fn is a callback when a IBC channel is established (after both sides agree in open)
pub fn do_ibc_channel_connect<C, E>(
    ibc_connect_fn: &dyn Fn(DepsMut, Env, IbcChannel) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    C: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema,
    E: ToString,
{
    let res = _do_ibc_channel_connect(
        ibc_connect_fn,
        env_ptr as *mut Region,
        msg_ptr as *mut Region,
    );
    let v = to_vec(&res).unwrap();
    release_buffer(v) as u32
}

fn _do_ibc_channel_connect<C, E>(
    ibc_connect_fn: &dyn Fn(DepsMut, Env, IbcChannel) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<IbcBasicResponse<C>>
where
    C: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema,
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_slice(&env));
    let msg: IbcChannel = try_into_contract_result!(from_slice(&msg));

    let mut deps = make_dependencies();
    ibc_connect_fn(deps.as_mut(), env, msg).into()
}

/// do_ibc_channel_close is designed for use with #[entry_point] to make a "C" extern
///
/// ibc_close_fn is a callback when a IBC channel belonging to this contract is closed
pub fn do_ibc_channel_close<C, E>(
    ibc_close_fn: &dyn Fn(DepsMut, Env, IbcChannel) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    C: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema,
    E: ToString,
{
    let res = _do_ibc_channel_close(ibc_close_fn, env_ptr as *mut Region, msg_ptr as *mut Region);
    let v = to_vec(&res).unwrap();
    release_buffer(v) as u32
}

fn _do_ibc_channel_close<C, E>(
    ibc_close_fn: &dyn Fn(DepsMut, Env, IbcChannel) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<IbcBasicResponse<C>>
where
    C: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema,
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_slice(&env));
    let msg: IbcChannel = try_into_contract_result!(from_slice(&msg));

    let mut deps = make_dependencies();
    ibc_close_fn(deps.as_mut(), env, msg).into()
}

/// do_ibc_packet_receive is designed for use with #[entry_point] to make a "C" extern
///
/// ibc_receive_fn is called when this chain receives an IBC Packet on a channel belonging
/// to this contract
pub fn do_ibc_packet_receive<C, E>(
    ibc_receive_fn: &dyn Fn(DepsMut, Env, IbcPacket) -> Result<IbcReceiveResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    C: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema,
    E: ToString,
{
    let res = _do_ibc_packet_receive(
        ibc_receive_fn,
        env_ptr as *mut Region,
        msg_ptr as *mut Region,
    );
    let v = to_vec(&res).unwrap();
    release_buffer(v) as u32
}

fn _do_ibc_packet_receive<C, E>(
    ibc_receive_fn: &dyn Fn(DepsMut, Env, IbcPacket) -> Result<IbcReceiveResponse<C>, E>,
    env_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<IbcReceiveResponse<C>>
where
    C: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema,
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_slice(&env));
    let msg: IbcPacket = try_into_contract_result!(from_slice(&msg));

    let mut deps = make_dependencies();
    ibc_receive_fn(deps.as_mut(), env, msg).into()
}

/// do_ibc_packet_ack is designed for use with #[entry_point] to make a "C" extern
///
/// ibc_ack_fn is called when this chain receives an IBC Acknowledgement for a packet
/// that this contract previously sent
pub fn do_ibc_packet_ack<C, E>(
    ibc_ack_fn: &dyn Fn(DepsMut, Env, IbcAcknowledgement) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    C: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema,
    E: ToString,
{
    let res = _do_ibc_packet_ack(ibc_ack_fn, env_ptr as *mut Region, msg_ptr as *mut Region);
    let v = to_vec(&res).unwrap();
    release_buffer(v) as u32
}

fn _do_ibc_packet_ack<C, E>(
    ibc_ack_fn: &dyn Fn(DepsMut, Env, IbcAcknowledgement) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<IbcBasicResponse<C>>
where
    C: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema,
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_slice(&env));
    let msg: IbcAcknowledgement = try_into_contract_result!(from_slice(&msg));

    let mut deps = make_dependencies();
    ibc_ack_fn(deps.as_mut(), env, msg).into()
}

/// do_ibc_packet_timeout is designed for use with #[entry_point] to make a "C" extern
///
/// ibc_timeout_fn is called when a packet that this contract previously sent has provably
/// timedout and will never be relayed to the calling chain. This generally behaves
/// like ick_ack_fn upon an acknowledgement containing an error.
pub fn do_ibc_packet_timeout<C, E>(
    ibc_timeout_fn: &dyn Fn(DepsMut, Env, IbcPacket) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    C: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema,
    E: ToString,
{
    let res = _do_ibc_packet_timeout(
        ibc_timeout_fn,
        env_ptr as *mut Region,
        msg_ptr as *mut Region,
    );
    let v = to_vec(&res).unwrap();
    release_buffer(v) as u32
}

fn _do_ibc_packet_timeout<C, E>(
    ibc_timeout_fn: &dyn Fn(DepsMut, Env, IbcPacket) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<IbcBasicResponse<C>>
where
    C: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema,
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_slice(&env));
    let msg: IbcPacket = try_into_contract_result!(from_slice(&msg));

    let mut deps = make_dependencies();
    ibc_timeout_fn(deps.as_mut(), env, msg).into()
}
