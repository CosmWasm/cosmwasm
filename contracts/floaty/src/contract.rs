use cosmwasm_std::{
    entry_point, to_json_binary, Deps, DepsMut, Empty, Env, MessageInfo, QueryResponse, Response,
    StdResult,
};
use rand_chacha::rand_core::SeedableRng;

#[cfg(target_arch = "wasm32")]
use crate::instructions::run_instruction;
use crate::{
    instructions::{random_args_for, Value, FLOAT_INSTRUCTIONS},
    msg::QueryMsg,
};

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> Result<Response, String> {
    Ok(Response::default())
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::RandomArgsFor { instruction, seed } => {
            let mut rng = rand_chacha::ChaChaRng::seed_from_u64(seed);
            to_json_binary(&random_args_for(&instruction, &mut rng))
        }
        QueryMsg::Instructions {} => to_json_binary(&FLOAT_INSTRUCTIONS.to_vec()),
        QueryMsg::Run { instruction, args } => to_json_binary(&query_run(&instruction, args)?),
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
fn query_run(instruction: &str, args: Vec<Value>) -> StdResult<Value> {
    #[cfg(not(target_arch = "wasm32"))]
    panic!();

    #[cfg(target_arch = "wasm32")]
    {
        let result = run_instruction(instruction, &args);
        Ok(result)
    }
}
