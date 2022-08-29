use cosmwasm_std::{entry_point, DepsMut, Empty, Env, MessageInfo, Response, StdError};

use crate::errors::HackError;
use crate::msg::ExecuteMsg;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> Result<Response, HackError> {
    deps.api.debug("here we go 🚀");

    // This adds some unrelated event attribute for testing purposes
    Ok(Response::new().add_attribute("Let the", "hacking begin"))
}

#[entry_point]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, HackError> {
    match msg {
        ExecuteMsg::Argon2 {
            mem_cost,
            time_cost,
        } => do_argon2(mem_cost, time_cost),
    }
}

fn do_argon2(mem_cost: u32, time_cost: u32) -> Result<Response, HackError> {
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

// #[entry_point]
// pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
//     match msg {}
// }
