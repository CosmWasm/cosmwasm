use cosmwasm_std::{
    entry_point, to_json_binary, Api, DenomMetadata, Deps, DepsMut, Empty, Env, MessageInfo,
    PageRequest, QueryResponse, Response, StdError, StdResult, WasmMsg,
};

use crate::errors::ContractError;
use crate::msg::{ExecuteMsg, QueryMsg};

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;

    match msg {
        Argon2 {
            mem_cost,
            time_cost,
        } => execute_argon2(mem_cost, time_cost),
        CpuLoop {} => execute_cpu_loop(),
        StorageLoop {} => execute_storage_loop(deps),
        MemoryLoop {} => execute_memory_loop(),
        MessageLoop {} => execute_message_loop(env),
        AllocateLargeMemory { pages } => execute_allocate_large_memory(pages),
        Panic {} => execute_panic(),
        Unreachable {} => execute_unreachable(),
        MirrorEnv {} => execute_mirror_env(env),
        Debug {} => execute_debug(deps.api),
        Noop {} => execute_noop(),
    }
}

fn execute_argon2(mem_cost: u32, time_cost: u32) -> Result<Response, ContractError> {
    let password = b"password";
    let salt = b"othersalt";
    let config = argon2::Config {
        variant: argon2::Variant::Argon2i,
        version: argon2::Version::Version13,
        mem_cost,
        time_cost,
        lanes: 4,
        secret: &[],
        ad: &[],
        hash_length: 32,
    };
    let hash = argon2::hash_encoded(password, salt, &config)
        .map_err(|e| StdError::generic_err(format!("hash_encoded errored: {e}")))?;
    // let matches = argon2::verify_encoded(&hash, password).unwrap();
    // assert!(matches);
    Ok(Response::new().set_data(hash.into_bytes()))
    //Ok(Response::new())
}

fn execute_cpu_loop() -> Result<Response, ContractError> {
    let mut counter = 0u64;
    loop {
        counter += 1;
        if counter >= 9_000_000_000 {
            counter = 0;
        }
    }
}

fn execute_storage_loop(deps: DepsMut) -> Result<Response, ContractError> {
    let mut test_case = 0u64;
    loop {
        deps.storage
            .set(b"test.key", test_case.to_string().as_bytes());
        test_case += 1;
    }
}

fn execute_memory_loop() -> Result<Response, ContractError> {
    let mut data = vec![1usize];
    loop {
        // add one element
        data.push((*data.last().expect("must not be empty")) + 1);
    }
}

fn execute_message_loop(env: Env) -> Result<Response, ContractError> {
    let resp = Response::new().add_message(WasmMsg::Execute {
        contract_addr: env.contract.address.into(),
        msg: to_json_binary(&ExecuteMsg::MessageLoop {})?,
        funds: vec![],
    });
    Ok(resp)
}

#[allow(unused_variables)]
fn execute_allocate_large_memory(pages: u32) -> Result<Response, ContractError> {
    // We create memory pages explicitly since Rust's default allocator seems to be clever enough
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

fn execute_panic() -> Result<Response, ContractError> {
    // Uncomment your favourite panic case

    // panicked at 'This page intentionally faulted', src/contract.rs:53:5
    panic!("This page intentionally faulted");

    // panicked at 'oh no (a = 3)', src/contract.rs:56:5
    // let a = 3;
    // panic!("oh no (a = {a})");

    // panicked at 'attempt to subtract with overflow', src/contract.rs:59:13
    // #[allow(arithmetic_overflow)]
    // let _ = 5u32 - 8u32;

    // panicked at 'no entry found for key', src/contract.rs:62:13
    // let map = std::collections::HashMap::<String, String>::new();
    // let _ = map["foo"];
}

fn execute_unreachable() -> Result<Response, ContractError> {
    #[cfg(target_arch = "wasm32")]
    core::arch::wasm32::unreachable();

    #[cfg(not(target_arch = "wasm32"))]
    Err(StdError::generic_err("Unsupported architecture").into())
}

fn execute_mirror_env(env: Env) -> Result<Response, ContractError> {
    Ok(Response::new().set_data(to_json_binary(&env)?))
}

fn execute_debug(api: &dyn Api) -> Result<Response, ContractError> {
    api.debug("Hey, ho – let's go");

    let password = b"password";
    let salt = b"othersalt";

    for r in 1..10 {
        api.debug(&format!("Round {r} starting"));
        let config = argon2::Config {
            variant: argon2::Variant::Argon2i,
            version: argon2::Version::Version13,
            mem_cost: 32,
            time_cost: r,
            lanes: 4,
            secret: &[],
            ad: &[],
            hash_length: 32,
        };
        let _hash = argon2::hash_encoded(password, salt, &config).unwrap();
        api.debug(&format!("Round {r} done"));
    }

    api.debug("Work completed, bye");
    Ok(Response::default())
}

fn execute_noop() -> Result<Response, ContractError> {
    Ok(Response::new())
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    use QueryMsg::*;

    match msg {
        MirrorEnv {} => to_json_binary(&query_mirror_env(env)),
        Denoms {} => to_json_binary(&query_denoms(deps)?),
        Denom { denom } => to_json_binary(&query_denom(deps, denom)?),
    }
}

fn query_mirror_env(env: Env) -> Env {
    env
}

fn query_denoms(deps: Deps) -> StdResult<Vec<DenomMetadata>> {
    const PAGE_SIZE: u32 = 10;
    let mut next_key = None;
    let mut all_metadata = Vec::new();
    loop {
        let page = deps.querier.query_all_denom_metadata(PageRequest {
            key: next_key,
            limit: PAGE_SIZE,
            reverse: false,
        })?;

        let len = page.metadata.len() as u32;
        all_metadata.extend(page.metadata);
        next_key = page.next_key;

        if next_key.is_none() || len < PAGE_SIZE {
            break;
        }
    }

    Ok(all_metadata)
}

fn query_denom(deps: Deps, denom: String) -> StdResult<DenomMetadata> {
    deps.querier.query_denom_metadata(denom)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{from_json, DenomMetadata, DenomUnit, OwnedDeps};

    fn setup() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();
        let creator = deps.api.addr_make("creator");
        let msg = Empty {};
        let info = message_info(&creator, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        deps
    }

    #[test]
    fn instantiate_works() {
        setup();
    }

    #[test]
    fn debug_works() {
        let mut deps = setup();
        let caller = deps.api.addr_make("caller");

        let msg = ExecuteMsg::Debug {};
        execute(deps.as_mut(), mock_env(), message_info(&caller, &[]), msg).unwrap();
    }

    #[test]
    fn query_denoms_works() {
        let mut deps = setup();

        deps.querier.bank.set_denom_metadata(
            &(0..98)
                .map(|i| DenomMetadata {
                    symbol: format!("FOO{i}"),
                    name: "Foo".to_string(),
                    description: "Foo coin".to_string(),
                    denom_units: vec![DenomUnit {
                        denom: "ufoo".to_string(),
                        exponent: 8,
                        aliases: vec!["microfoo".to_string(), "foobar".to_string()],
                    }],
                    display: "FOO".to_string(),
                    base: format!("ufoo{i}"),
                    uri: "https://foo.bar".to_string(),
                    uri_hash: "foo".to_string(),
                })
                .collect::<Vec<_>>(),
        );

        let symbols: Vec<DenomMetadata> =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::Denoms {}).unwrap()).unwrap();

        assert_eq!(symbols.len(), 98);

        let denom: DenomMetadata = from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Denom {
                    denom: "ufoo0".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(denom.symbol, "FOO0");
    }
}
