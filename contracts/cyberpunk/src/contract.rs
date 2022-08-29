use cosmwasm_std::{
    entry_point, to_binary, Deps, DepsMut, Empty, Env, MessageInfo, QueryResponse, Response,
    StdError, StdResult,
};

use crate::errors::ContractError;
use crate::msg::{ExecuteMsg, QueryMsg};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> Result<Response, ContractError> {
    deps.api.debug("here we go 🚀");

    // This adds some unrelated event attribute for testing purposes
    Ok(Response::new().add_attribute("Let the", "hacking begin"))
}

#[entry_point]
pub fn execute(
    _deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;

    match msg {
        Argon2 {
            mem_cost,
            time_cost,
        } => execute::argon2(mem_cost, time_cost),
        MirrorEnv {} => execute::mirror_env(env),
    }
}

mod execute {
    use super::*;

    pub fn argon2(mem_cost: u32, time_cost: u32) -> Result<Response, ContractError> {
        let password = b"password";
        let salt = b"othersalt";
        let config = argon2::Config {
            variant: argon2::Variant::Argon2i,
            version: argon2::Version::Version13,
            mem_cost,
            time_cost,
            lanes: 4,
            thread_mode: argon2::ThreadMode::Sequential,
            secret: &[],
            ad: &[],
            hash_length: 32,
        };
        let hash = argon2::hash_encoded(password, salt, &config)
            .map_err(|e| StdError::generic_err(format!("hash_encoded errored: {}", e)))?;
        // let matches = argon2::verify_encoded(&hash, password).unwrap();
        // assert!(matches);
        Ok(Response::new().set_data(hash.into_bytes()))
        //Ok(Response::new())
    }

    pub fn mirror_env(env: Env) -> Result<Response, ContractError> {
        Ok(Response::new().set_data(to_binary(&env)?))
    }
}

#[entry_point]
pub fn query(_deps: Deps, env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    use QueryMsg::*;

    match msg {
        MirrorEnv {} => to_binary(&query::mirror_env(env)),
    }
}

mod query {
    use super::*;

    pub fn mirror_env(env: Env) -> Env {
        env
    }
}
