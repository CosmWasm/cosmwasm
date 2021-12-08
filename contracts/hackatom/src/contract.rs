use sha2::{Digest, Sha256};

use cosmwasm_std::{
    entry_point, from_slice, to_binary, to_vec, Addr, AllBalanceResponse, Api, BankMsg,
    CanonicalAddr, Deps, DepsMut, Env, Event, MessageInfo, QueryRequest, QueryResponse, Response,
    StdError, StdResult, WasmQuery,
};

use crate::errors::HackError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, IntResponse, MigrateMsg, QueryMsg, RecurseResponse, SudoMsg,
    VerifierResponse,
};
use crate::state::{State, CONFIG_KEY};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, HackError> {
    deps.api.debug("here we go 🚀");

    deps.storage.set(
        CONFIG_KEY,
        &to_vec(&State {
            verifier: deps.api.addr_validate(&msg.verifier)?,
            beneficiary: deps.api.addr_validate(&msg.beneficiary)?,
            funder: info.sender,
        })?,
    );

    // This adds some unrelated event attribute for testing purposes
    Ok(Response::new().add_attribute("Let the", "hacking begin"))
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, HackError> {
    let data = deps
        .storage
        .get(CONFIG_KEY)
        .ok_or_else(|| StdError::not_found("State"))?;
    let mut config: State = from_slice(&data)?;
    config.verifier = deps.api.addr_validate(&msg.verifier)?;
    deps.storage.set(CONFIG_KEY, &to_vec(&config)?);

    Ok(Response::default())
}

#[entry_point]
pub fn sudo(_deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, HackError> {
    match msg {
        SudoMsg::StealFunds { recipient, amount } => {
            let msg = BankMsg::Send {
                to_address: recipient,
                amount,
            };
            Ok(Response::new().add_message(msg))
        }
    }
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, HackError> {
    match msg {
        ExecuteMsg::Release {} => do_release(deps, env, info),
        ExecuteMsg::Argon2 {
            mem_cost,
            time_cost,
        } => do_argon2(mem_cost, time_cost),
        ExecuteMsg::CpuLoop {} => do_cpu_loop(),
        ExecuteMsg::StorageLoop {} => do_storage_loop(deps),
        ExecuteMsg::MemoryLoop {} => do_memory_loop(),
        ExecuteMsg::AllocateLargeMemory { pages } => do_allocate_large_memory(pages),
        ExecuteMsg::Panic {} => do_panic(),
        ExecuteMsg::UserErrorsInApiCalls {} => do_user_errors_in_api_calls(deps.api),
    }
}

fn do_release(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, HackError> {
    let data = deps
        .storage
        .get(CONFIG_KEY)
        .ok_or_else(|| StdError::not_found("State"))?;
    let state: State = from_slice(&data)?;

    if info.sender == state.verifier {
        let to_addr = state.beneficiary;
        let balance = deps.querier.query_all_balances(env.contract.address)?;

        let resp = Response::new()
            .add_attribute("action", "release")
            .add_attribute("destination", to_addr.clone())
            .add_event(Event::new("hackatom").add_attribute("action", "release"))
            .add_message(BankMsg::Send {
                to_address: to_addr.into(),
                amount: balance,
            })
            .set_data(&[0xF0, 0x0B, 0xAA]);
        Ok(resp)
    } else {
        Err(HackError::Unauthorized {})
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

fn do_cpu_loop() -> Result<Response, HackError> {
    let mut counter = 0u64;
    loop {
        counter += 1;
        if counter >= 9_000_000_000 {
            counter = 0;
        }
    }
}

fn do_storage_loop(deps: DepsMut) -> Result<Response, HackError> {
    let mut test_case = 0u64;
    loop {
        deps.storage
            .set(b"test.key", test_case.to_string().as_bytes());
        test_case += 1;
    }
}

fn do_memory_loop() -> Result<Response, HackError> {
    let mut data = vec![1usize];
    loop {
        // add one element
        data.push((*data.last().expect("must not be empty")) + 1);
    }
}

#[allow(unused_variables)]
fn do_allocate_large_memory(pages: u32) -> Result<Response, HackError> {
    // We create memory pages explicitely since Rust's default allocator seems to be clever enough
    // to not grow memory for unused capacity like `Vec::<u8>::with_capacity(100 * 1024 * 1024)`.
    // Even with std::alloc::alloc the memory did now grow beyond 1.5 MiB.

    #[cfg(target_arch = "wasm32")]
    {
        use core::arch::wasm32;
        let old_size = wasm32::memory_grow(0, pages as usize);
        if old_size == usize::max_value() {
            return Err(StdError::generic_err("memory.grow failed").into());
        }
        Ok(Response::new().set_data((old_size as u32).to_be_bytes()))
    }

    #[cfg(not(target_arch = "wasm32"))]
    Err(StdError::generic_err("Unsupported architecture").into())
}

fn do_panic() -> Result<Response, HackError> {
    panic!("This page intentionally faulted");
}

fn do_user_errors_in_api_calls(api: &dyn Api) -> Result<Response, HackError> {
    // Canonicalize

    let empty = "";
    match api.addr_canonicalize(empty).unwrap_err() {
        StdError::GenericErr { .. } => {}
        err => {
            return Err(StdError::generic_err(format!(
                "Unexpected error in do_user_errors_in_api_calls: {:?}",
                err
            ))
            .into())
        }
    }

    let invalid_bech32 =
        "bn9hhssomeltvhzgvuqkwjkpwxojfuigltwedayzxljucefikuieillowaticksoistqoynmgcnj219a";
    match api.addr_canonicalize(invalid_bech32).unwrap_err() {
        StdError::GenericErr { .. } => {}
        err => {
            return Err(StdError::generic_err(format!(
                "Unexpected error in do_user_errors_in_api_calls: {:?}",
                err
            ))
            .into())
        }
    }

    // Humanize

    let empty: CanonicalAddr = vec![].into();
    match api.addr_humanize(&empty).unwrap_err() {
        StdError::GenericErr { .. } => {}
        err => {
            return Err(StdError::generic_err(format!(
                "Unexpected error in do_user_errors_in_api_calls: {:?}",
                err
            ))
            .into())
        }
    }

    let too_short: CanonicalAddr = vec![0xAA, 0xBB, 0xCC].into();
    match api.addr_humanize(&too_short).unwrap_err() {
        StdError::GenericErr { .. } => {}
        err => {
            return Err(StdError::generic_err(format!(
                "Unexpected error in do_user_errors_in_api_calls: {:?}",
                err
            ))
            .into())
        }
    }

    let wrong_length: CanonicalAddr = vec![0xA6; 17].into();
    match api.addr_humanize(&wrong_length).unwrap_err() {
        StdError::GenericErr { .. } => {}
        err => {
            return Err(StdError::generic_err(format!(
                "Unexpected error in do_user_errors_in_api_calls: {:?}",
                err
            ))
            .into())
        }
    }

    Ok(Response::default())
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Verifier {} => to_binary(&query_verifier(deps)?),
        QueryMsg::OtherBalance { address } => to_binary(&query_other_balance(deps, address)?),
        QueryMsg::Recurse { depth, work } => {
            to_binary(&query_recurse(deps, depth, work, env.contract.address)?)
        }
        QueryMsg::GetInt {} => to_binary(&query_int()),
    }
}

fn query_verifier(deps: Deps) -> StdResult<VerifierResponse> {
    let data = deps
        .storage
        .get(CONFIG_KEY)
        .ok_or_else(|| StdError::not_found("State"))?;
    let state: State = from_slice(&data)?;
    Ok(VerifierResponse {
        verifier: state.verifier.into(),
    })
}

fn query_other_balance(deps: Deps, address: String) -> StdResult<AllBalanceResponse> {
    let amount = deps.querier.query_all_balances(address)?;
    Ok(AllBalanceResponse { amount })
}

fn query_recurse(deps: Deps, depth: u32, work: u32, contract: Addr) -> StdResult<RecurseResponse> {
    // perform all hashes as requested
    let mut hashed: Vec<u8> = contract.as_str().into();
    for _ in 0..work {
        hashed = Sha256::digest(&hashed).to_vec()
    }

    // the last contract should return the response
    if depth == 0 {
        Ok(RecurseResponse {
            hashed: hashed.into(),
        })
    } else {
        // otherwise, we go one level deeper and return the response of the next level
        let req = QueryMsg::Recurse {
            depth: depth - 1,
            work,
        };
        let query = QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract.into(),
            msg: to_binary(&req)?,
        });
        deps.querier.query(&query)
    }
}

fn query_int() -> IntResponse {
    IntResponse { int: 0xf00baa }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_dependencies_with_balances, mock_env, mock_info, MOCK_CONTRACT_ADDR,
    };
    // import trait Storage to get access to read
    use cosmwasm_std::{coins, Storage, SubMsg};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let verifier = String::from("verifies");
        let beneficiary = String::from("benefits");
        let creator = String::from("creator");
        let expected_state = State {
            verifier: deps.api.addr_validate(&verifier).unwrap(),
            beneficiary: deps.api.addr_validate(&beneficiary).unwrap(),
            funder: deps.api.addr_validate(&creator).unwrap(),
        };

        let msg = InstantiateMsg {
            verifier,
            beneficiary,
        };
        let info = mock_info(creator.as_str(), &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 0);
        assert_eq!(res.attributes, [("Let the", "hacking begin")]);

        // it worked, let's check the state
        let data = deps.storage.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_slice(&data).unwrap();
        assert_eq!(state, expected_state);
    }

    #[test]
    fn instantiate_and_query() {
        let mut deps = mock_dependencies();

        let verifier = String::from("verifies");
        let beneficiary = String::from("benefits");
        let creator = String::from("creator");
        let msg = InstantiateMsg {
            verifier: verifier.clone(),
            beneficiary,
        };
        let info = mock_info(&creator, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // now let's query
        let query_response = query_verifier(deps.as_ref()).unwrap();
        assert_eq!(query_response.verifier, verifier);
    }

    #[test]
    fn migrate_verifier() {
        let mut deps = mock_dependencies();

        let verifier = String::from("verifies");
        let beneficiary = String::from("benefits");
        let creator = String::from("creator");
        let msg = InstantiateMsg {
            verifier,
            beneficiary,
        };
        let info = mock_info(&creator, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // check it is 'verifies'
        let query_response = query(deps.as_ref(), mock_env(), QueryMsg::Verifier {}).unwrap();
        assert_eq!(query_response.as_slice(), b"{\"verifier\":\"verifies\"}");

        // change the verifier via migrate
        let new_verifier = String::from("someone else");
        let msg = MigrateMsg {
            verifier: new_verifier.clone(),
        };
        let res = migrate(deps.as_mut(), mock_env(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        // check it is 'someone else'
        let query_response = query_verifier(deps.as_ref()).unwrap();
        assert_eq!(query_response.verifier, new_verifier);
    }

    #[test]
    fn sudo_can_steal_tokens() {
        let mut deps = mock_dependencies();

        let verifier = String::from("verifies");
        let beneficiary = String::from("benefits");
        let creator = String::from("creator");
        let msg = InstantiateMsg {
            verifier,
            beneficiary,
        };
        let info = mock_info(&creator, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // sudo takes any tax it wants
        let to_address = String::from("community-pool");
        let amount = coins(700, "gold");
        let sys_msg = SudoMsg::StealFunds {
            recipient: to_address.clone(),
            amount: amount.clone(),
        };
        let res = sudo(deps.as_mut(), mock_env(), sys_msg).unwrap();
        assert_eq!(1, res.messages.len());
        let msg = res.messages.get(0).expect("no message");
        assert_eq!(msg, &SubMsg::new(BankMsg::Send { to_address, amount }));
    }

    #[test]
    fn querier_callbacks_work() {
        let rich_addr = String::from("foobar");
        let rich_balance = coins(10000, "gold");
        let deps = mock_dependencies_with_balances(&[(&rich_addr, &rich_balance)]);

        // querying with balance gets the balance
        let bal = query_other_balance(deps.as_ref(), rich_addr).unwrap();
        assert_eq!(bal.amount, rich_balance);

        // querying other accounts gets none
        let bal = query_other_balance(deps.as_ref(), String::from("someone else")).unwrap();
        assert_eq!(bal.amount, vec![]);
    }

    #[test]
    fn execute_release_works() {
        let mut deps = mock_dependencies();

        // initialize the store
        let creator = String::from("creator");
        let verifier = String::from("verifies");
        let beneficiary = String::from("benefits");

        let instantiate_msg = InstantiateMsg {
            verifier: verifier.clone(),
            beneficiary: beneficiary.clone(),
        };
        let init_amount = coins(1000, "earth");
        let init_info = mock_info(&creator, &init_amount);
        let init_res = instantiate(deps.as_mut(), mock_env(), init_info, instantiate_msg).unwrap();
        assert_eq!(init_res.messages.len(), 0);

        // balance changed in init
        deps.querier.update_balance(MOCK_CONTRACT_ADDR, init_amount);

        // beneficiary can release it
        let execute_info = mock_info(verifier.as_str(), &[]);
        let execute_res = execute(
            deps.as_mut(),
            mock_env(),
            execute_info,
            ExecuteMsg::Release {},
        )
        .unwrap();
        assert_eq!(execute_res.messages.len(), 1);
        let msg = execute_res.messages.get(0).expect("no message");
        assert_eq!(
            msg,
            &SubMsg::new(BankMsg::Send {
                to_address: beneficiary,
                amount: coins(1000, "earth"),
            }),
        );
        assert_eq!(
            execute_res.attributes,
            vec![("action", "release"), ("destination", "benefits")],
        );
        assert_eq!(execute_res.data, Some(vec![0xF0, 0x0B, 0xAA].into()));
    }

    #[test]
    fn execute_release_fails_for_wrong_sender() {
        let mut deps = mock_dependencies();

        // initialize the store
        let creator = String::from("creator");
        let verifier = String::from("verifies");
        let beneficiary = String::from("benefits");

        let instantiate_msg = InstantiateMsg {
            verifier: verifier.clone(),
            beneficiary: beneficiary.clone(),
        };
        let init_amount = coins(1000, "earth");
        let init_info = mock_info(&creator, &init_amount);
        let init_res = instantiate(deps.as_mut(), mock_env(), init_info, instantiate_msg).unwrap();
        assert_eq!(init_res.messages.len(), 0);

        // balance changed in init
        deps.querier.update_balance(MOCK_CONTRACT_ADDR, init_amount);

        // beneficiary cannot release it
        let execute_info = mock_info(beneficiary.as_str(), &[]);
        let execute_res = execute(
            deps.as_mut(),
            mock_env(),
            execute_info,
            ExecuteMsg::Release {},
        );
        assert_eq!(execute_res.unwrap_err(), HackError::Unauthorized {});

        // state should not change
        let data = deps.storage.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_slice(&data).unwrap();
        assert_eq!(
            state,
            State {
                verifier: Addr::unchecked(verifier),
                beneficiary: Addr::unchecked(beneficiary),
                funder: Addr::unchecked(creator),
            }
        );
    }

    #[test]
    #[should_panic(expected = "This page intentionally faulted")]
    fn execute_panic() {
        let mut deps = mock_dependencies();

        // initialize the store
        let verifier = String::from("verifies");
        let beneficiary = String::from("benefits");
        let creator = String::from("creator");

        let instantiate_msg = InstantiateMsg {
            verifier,
            beneficiary: beneficiary.clone(),
        };
        let init_info = mock_info(&creator, &coins(1000, "earth"));
        let init_res = instantiate(deps.as_mut(), mock_env(), init_info, instantiate_msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        let execute_info = mock_info(&beneficiary, &[]);
        // this should panic
        let _ = execute(
            deps.as_mut(),
            mock_env(),
            execute_info,
            ExecuteMsg::Panic {},
        );
    }

    #[test]
    fn execute_user_errors_in_api_calls() {
        let mut deps = mock_dependencies();

        let instantiate_msg = InstantiateMsg {
            verifier: String::from("verifies"),
            beneficiary: String::from("benefits"),
        };
        let init_info = mock_info("creator", &coins(1000, "earth"));
        let init_res = instantiate(deps.as_mut(), mock_env(), init_info, instantiate_msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        let execute_info = mock_info("anyone", &[]);
        execute(
            deps.as_mut(),
            mock_env(),
            execute_info,
            ExecuteMsg::UserErrorsInApiCalls {},
        )
        .unwrap();
    }

    #[test]
    fn get_int() {
        let get_int_query = query_int();
        assert_eq!(get_int_query.int, 0xf00baa);
    }
}
