use cosmwasm_std::{
    attr, to_binary, to_vec, Binary, ContractResult, CosmosMsg, DepsMut, DepsRef, Env,
    HandleResponse, HumanAddr, InitResponse, MessageInfo, QueryRequest, StdError, StdResult,
    SystemResult,
};

use crate::errors::ReflectError;
use crate::msg::{
    CapitalizedResponse, ChainResponse, CustomMsg, HandleMsg, InitMsg, OwnerResponse, QueryMsg,
    RawResponse, SpecialQuery, SpecialResponse,
};
use crate::state::{config, config_read, State};

pub fn init(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InitMsg,
) -> StdResult<InitResponse<CustomMsg>> {
    let state = State {
        owner: deps.api.canonical_address(&info.sender)?,
    };

    config(deps.storage).save(&state)?;
    Ok(InitResponse::default())
}

pub fn handle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> Result<HandleResponse<CustomMsg>, ReflectError> {
    match msg {
        HandleMsg::ReflectMsg { msgs } => try_reflect(deps, env, info, msgs),
        HandleMsg::ChangeOwner { owner } => try_change_owner(deps, env, info, owner),
    }
}

pub fn try_reflect(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msgs: Vec<CosmosMsg<CustomMsg>>,
) -> Result<HandleResponse<CustomMsg>, ReflectError> {
    let state = config(deps.storage).load()?;

    let sender = deps.api.canonical_address(&info.sender)?;
    if sender != state.owner {
        return Err(ReflectError::NotCurrentOwner {
            expected: state.owner,
            actual: sender,
        });
    }

    if msgs.is_empty() {
        return Err(ReflectError::MessagesEmpty);
    }
    let res = HandleResponse {
        messages: msgs,
        attributes: vec![attr("action", "reflect")],
        data: None,
    };
    Ok(res)
}

pub fn try_change_owner(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    owner: HumanAddr,
) -> Result<HandleResponse<CustomMsg>, ReflectError> {
    let api = deps.api;
    config(deps.storage).update(|mut state| {
        let sender = api.canonical_address(&info.sender)?;
        if sender != state.owner {
            return Err(ReflectError::NotCurrentOwner {
                expected: state.owner,
                actual: sender,
            });
        }
        state.owner = api.canonical_address(&owner)?;
        Ok(state)
    })?;
    Ok(HandleResponse {
        attributes: vec![attr("action", "change_owner"), attr("owner", owner)],
        ..HandleResponse::default()
    })
}

pub fn query(deps: DepsRef, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Owner {} => to_binary(&query_owner(deps)?),
        QueryMsg::Capitalized { text } => to_binary(&query_capitalized(deps, text)?),
        QueryMsg::Chain { request } => to_binary(&query_chain(deps, &request)?),
        QueryMsg::Raw { contract, key } => to_binary(&query_raw(deps, contract, key)?),
    }
}

fn query_owner(deps: DepsRef) -> StdResult<OwnerResponse> {
    let state = config_read(deps.storage.as_ref()).load()?;
    let resp = OwnerResponse {
        owner: deps.api.human_address(&state.owner)?,
    };
    Ok(resp)
}

fn query_capitalized(deps: DepsRef, text: String) -> StdResult<CapitalizedResponse> {
    let req = SpecialQuery::Capitalized { text }.into();
    let response: SpecialResponse = deps.querier.custom_query(&req)?;
    Ok(CapitalizedResponse { text: response.msg })
}

fn query_chain(deps: DepsRef, request: &QueryRequest<SpecialQuery>) -> StdResult<ChainResponse> {
    let raw = to_vec(request).map_err(|serialize_err| {
        StdError::generic_err(format!("Serializing QueryRequest: {}", serialize_err))
    })?;
    match deps.querier.raw_query(&raw) {
        SystemResult::Err(system_err) => Err(StdError::generic_err(format!(
            "Querier system error: {}",
            system_err
        ))),
        SystemResult::Ok(ContractResult::Err(contract_err)) => Err(StdError::generic_err(format!(
            "Querier contract error: {}",
            contract_err
        ))),
        SystemResult::Ok(ContractResult::Ok(value)) => Ok(ChainResponse { data: value }),
    }
}

fn query_raw(deps: DepsRef, contract: HumanAddr, key: Binary) -> StdResult<RawResponse> {
    let response: Option<Vec<u8>> = deps.querier.query_wasm_raw(contract, key)?;
    Ok(RawResponse {
        data: response.unwrap_or_default().into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::mock_dependencies_with_custom_querier;
    use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{
        coin, coins, from_binary, AllBalanceResponse, Api, BankMsg, BankQuery, Binary, StakingMsg,
        StdError,
    };

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);

        let msg = InitMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let value = query_owner(deps.as_ref()).unwrap();
        assert_eq!("creator", value.owner.as_str());
    }

    #[test]
    fn reflect() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);

        let msg = InitMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

        let payload = vec![BankMsg::Send {
            from_address: HumanAddr::from(MOCK_CONTRACT_ADDR),
            to_address: HumanAddr::from("friend"),
            amount: coins(1, "token"),
        }
        .into()];

        let msg = HandleMsg::ReflectMsg {
            msgs: payload.clone(),
        };
        let info = mock_info("creator", &[]);
        let res = handle(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(payload, res.messages);
    }

    #[test]
    fn reflect_requires_owner() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);

        let msg = InitMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

        // signer is not owner
        let payload = vec![BankMsg::Send {
            from_address: HumanAddr::from(MOCK_CONTRACT_ADDR),
            to_address: HumanAddr::from("friend"),
            amount: coins(1, "token"),
        }
        .into()];
        let msg = HandleMsg::ReflectMsg {
            msgs: payload.clone(),
        };

        let info = mock_info("random", &[]);
        let res = handle(deps.as_mut(), mock_env(), info, msg);
        match res.unwrap_err() {
            ReflectError::NotCurrentOwner { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn reflect_reject_empty_msgs() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);

        let msg = InitMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("creator", &[]);
        let payload = vec![];

        let msg = HandleMsg::ReflectMsg {
            msgs: payload.clone(),
        };
        let res = handle(deps.as_mut(), mock_env(), info, msg);
        match res.unwrap_err() {
            ReflectError::MessagesEmpty => {}
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn reflect_multiple_messages() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);

        let msg = InitMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

        let payload = vec![
            BankMsg::Send {
                from_address: HumanAddr::from(MOCK_CONTRACT_ADDR),
                to_address: HumanAddr::from("friend"),
                amount: coins(1, "token"),
            }
            .into(),
            // make sure we can pass through custom native messages
            CustomMsg::Raw(Binary(b"{\"foo\":123}".to_vec())).into(),
            CustomMsg::Debug("Hi, Dad!".to_string()).into(),
            StakingMsg::Delegate {
                validator: HumanAddr::from("validator"),
                amount: coin(100, "ustake"),
            }
            .into(),
        ];

        let msg = HandleMsg::ReflectMsg {
            msgs: payload.clone(),
        };
        let info = mock_info("creator", &[]);
        let res = handle(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(payload, res.messages);
    }

    #[test]
    fn change_owner_works() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);

        let msg = InitMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("creator", &[]);
        let new_owner = HumanAddr::from("friend");
        let msg = HandleMsg::ChangeOwner {
            owner: new_owner.clone(),
        };
        let res = handle(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should change state
        assert_eq!(0, res.messages.len());
        let value = query_owner(deps.as_ref()).unwrap();
        assert_eq!("friend", value.owner.as_str());
    }

    #[test]
    fn change_owner_requires_current_owner_as_sender() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);

        let msg = InitMsg {};
        let creator = HumanAddr::from("creator");
        let info = mock_info(&creator, &coins(2, "token"));
        let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

        let random = HumanAddr::from("random");
        let info = mock_info(&random, &[]);
        let new_owner = HumanAddr::from("friend");
        let msg = HandleMsg::ChangeOwner {
            owner: new_owner.clone(),
        };

        let res = handle(deps.as_mut(), mock_env(), info, msg);
        match res.unwrap_err() {
            ReflectError::NotCurrentOwner { expected, actual } => {
                assert_eq!(expected, deps.api.canonical_address(&creator).unwrap());
                assert_eq!(actual, deps.api.canonical_address(&random).unwrap());
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn change_owner_errors_for_invalid_new_address() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);
        let creator = HumanAddr::from("creator");

        let msg = InitMsg {};
        let info = mock_info(&creator, &coins(2, "token"));
        let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info(&creator, &[]);
        let msg = HandleMsg::ChangeOwner {
            owner: HumanAddr::from("x"),
        };
        let res = handle(deps.as_mut(), mock_env(), info, msg);
        match res.unwrap_err() {
            ReflectError::Std(StdError::GenericErr { msg, .. }) => {
                assert!(msg.contains("human address too short"))
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn capitalized_query_works() {
        let deps = mock_dependencies_with_custom_querier(&[]);

        let msg = QueryMsg::Capitalized {
            text: "demo one".to_string(),
        };
        let response = query(deps.as_ref(), mock_env(), msg).unwrap();
        let value: CapitalizedResponse = from_binary(&response).unwrap();
        assert_eq!(value.text, "DEMO ONE");
    }

    #[test]
    fn chain_query_works() {
        let deps = mock_dependencies_with_custom_querier(&coins(123, "ucosm"));

        // with bank query
        let msg = QueryMsg::Chain {
            request: BankQuery::AllBalances {
                address: HumanAddr::from(MOCK_CONTRACT_ADDR),
            }
            .into(),
        };
        let response = query(deps.as_ref(), mock_env(), msg).unwrap();
        let outer: ChainResponse = from_binary(&response).unwrap();
        let inner: AllBalanceResponse = from_binary(&outer.data).unwrap();
        assert_eq!(inner.amount, coins(123, "ucosm"));

        // with custom query
        let msg = QueryMsg::Chain {
            request: SpecialQuery::Ping {}.into(),
        };
        let response = query(deps.as_ref(), mock_env(), msg).unwrap();
        let outer: ChainResponse = from_binary(&response).unwrap();
        let inner: SpecialResponse = from_binary(&outer.data).unwrap();
        assert_eq!(inner.msg, "pong");
    }
}
