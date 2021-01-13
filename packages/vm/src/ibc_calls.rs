#![cfg(feature = "stargate")]
use cosmwasm_std::{ContractResult, Env, IbcChannel};

use crate::backend::{Api, Querier, Storage};
use crate::calls::call_raw;
use crate::errors::VmResult;
use crate::instance::Instance;
use crate::serde::{from_slice, to_vec};

const MAX_LENGTH_IBC: usize = 100_000;

pub fn call_ibc_channel_open<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    channel: &IbcChannel,
) -> VmResult<ContractResult<()>>
where
    A: Api + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    let env = to_vec(env)?;
    let msg = to_vec(channel)?;
    instance.set_storage_readonly(false);
    let data = call_raw(instance, "ibc_channel_open", &[&env, &msg], MAX_LENGTH_IBC)?;
    let result: ContractResult<()> = from_slice(&data)?;
    Ok(result)
}
