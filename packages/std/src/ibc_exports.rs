#![cfg(all(feature = "stargate", target_arch = "wasm32"))]
use std::fmt;
use std::vec::Vec;

use schemars::JsonSchema;
use serde::Serialize;

use crate::deps::DepsMut;
use crate::exports::make_dependencies;
use crate::ibc::{
    IbcBasicResponse, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse,
};
use crate::memory::{consume_region, release_buffer, Region};
use crate::results::ContractResult;
use crate::serde::{from_slice, to_vec};
use crate::types::Env;

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
/// contract_fn does the protocol version negotiation during channel handshake phase
pub fn do_ibc_channel_open<E>(
    contract_fn: &dyn Fn(DepsMut, Env, IbcChannelOpenMsg) -> Result<(), E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    E: ToString,
{
    let res = _do_ibc_channel_open(contract_fn, env_ptr as *mut Region, msg_ptr as *mut Region);
    let v = to_vec(&res).unwrap();
    release_buffer(v) as u32
}

fn _do_ibc_channel_open<E>(
    contract_fn: &dyn Fn(DepsMut, Env, IbcChannelOpenMsg) -> Result<(), E>,
    env_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<()>
where
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_slice(&env));
    let msg: IbcChannelOpenMsg = try_into_contract_result!(from_slice(&msg));

    let mut deps = make_dependencies();
    contract_fn(deps.as_mut(), env, msg).into()
}

/// do_ibc_channel_connect is designed for use with #[entry_point] to make a "C" extern
///
/// contract_fn is a callback when a IBC channel is established (after both sides agree in open)
pub fn do_ibc_channel_connect<C, E>(
    contract_fn: &dyn Fn(DepsMut, Env, IbcChannelConnectMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    C: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema,
    E: ToString,
{
    let res = _do_ibc_channel_connect(contract_fn, env_ptr as *mut Region, msg_ptr as *mut Region);
    let v = to_vec(&res).unwrap();
    release_buffer(v) as u32
}

fn _do_ibc_channel_connect<C, E>(
    contract_fn: &dyn Fn(DepsMut, Env, IbcChannelConnectMsg) -> Result<IbcBasicResponse<C>, E>,
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
    let msg: IbcChannelConnectMsg = try_into_contract_result!(from_slice(&msg));

    let mut deps = make_dependencies();
    contract_fn(deps.as_mut(), env, msg).into()
}

/// do_ibc_channel_close is designed for use with #[entry_point] to make a "C" extern
///
/// contract_fn is a callback when a IBC channel belonging to this contract is closed
pub fn do_ibc_channel_close<C, E>(
    contract_fn: &dyn Fn(DepsMut, Env, IbcChannelCloseMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    C: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema,
    E: ToString,
{
    let res = _do_ibc_channel_close(contract_fn, env_ptr as *mut Region, msg_ptr as *mut Region);
    let v = to_vec(&res).unwrap();
    release_buffer(v) as u32
}

fn _do_ibc_channel_close<C, E>(
    contract_fn: &dyn Fn(DepsMut, Env, IbcChannelCloseMsg) -> Result<IbcBasicResponse<C>, E>,
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
    let msg: IbcChannelCloseMsg = try_into_contract_result!(from_slice(&msg));

    let mut deps = make_dependencies();
    contract_fn(deps.as_mut(), env, msg).into()
}

/// do_ibc_packet_receive is designed for use with #[entry_point] to make a "C" extern
///
/// contract_fn is called when this chain receives an IBC Packet on a channel belonging
/// to this contract
pub fn do_ibc_packet_receive<C, E>(
    contract_fn: &dyn Fn(DepsMut, Env, IbcPacketReceiveMsg) -> Result<IbcReceiveResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    C: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema,
    E: ToString,
{
    let res = _do_ibc_packet_receive(contract_fn, env_ptr as *mut Region, msg_ptr as *mut Region);
    let v = to_vec(&res).unwrap();
    release_buffer(v) as u32
}

fn _do_ibc_packet_receive<C, E>(
    contract_fn: &dyn Fn(DepsMut, Env, IbcPacketReceiveMsg) -> Result<IbcReceiveResponse<C>, E>,
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
    let msg: IbcPacketReceiveMsg = try_into_contract_result!(from_slice(&msg));

    let mut deps = make_dependencies();
    contract_fn(deps.as_mut(), env, msg).into()
}

/// do_ibc_packet_ack is designed for use with #[entry_point] to make a "C" extern
///
/// contract_fn is called when this chain receives an IBC Acknowledgement for a packet
/// that this contract previously sent
pub fn do_ibc_packet_ack<C, E>(
    contract_fn: &dyn Fn(DepsMut, Env, IbcPacketAckMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    C: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema,
    E: ToString,
{
    let res = _do_ibc_packet_ack(contract_fn, env_ptr as *mut Region, msg_ptr as *mut Region);
    let v = to_vec(&res).unwrap();
    release_buffer(v) as u32
}

fn _do_ibc_packet_ack<C, E>(
    contract_fn: &dyn Fn(DepsMut, Env, IbcPacketAckMsg) -> Result<IbcBasicResponse<C>, E>,
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
    let msg: IbcPacketAckMsg = try_into_contract_result!(from_slice(&msg));

    let mut deps = make_dependencies();
    contract_fn(deps.as_mut(), env, msg).into()
}

/// do_ibc_packet_timeout is designed for use with #[entry_point] to make a "C" extern
///
/// contract_fn is called when a packet that this contract previously sent has provably
/// timedout and will never be relayed to the calling chain. This generally behaves
/// like ick_ack_fn upon an acknowledgement containing an error.
pub fn do_ibc_packet_timeout<C, E>(
    contract_fn: &dyn Fn(DepsMut, Env, IbcPacketTimeoutMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    C: Serialize + Clone + fmt::Debug + PartialEq + JsonSchema,
    E: ToString,
{
    let res = _do_ibc_packet_timeout(contract_fn, env_ptr as *mut Region, msg_ptr as *mut Region);
    let v = to_vec(&res).unwrap();
    release_buffer(v) as u32
}

fn _do_ibc_packet_timeout<C, E>(
    contract_fn: &dyn Fn(DepsMut, Env, IbcPacketTimeoutMsg) -> Result<IbcBasicResponse<C>, E>,
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
    let msg: IbcPacketTimeoutMsg = try_into_contract_result!(from_slice(&msg));

    let mut deps = make_dependencies();
    contract_fn(deps.as_mut(), env, msg).into()
}
