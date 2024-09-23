use cosmwasm_std::{
    entry_point, to_json_binary, to_json_vec, Binary, ContractResult, CosmosMsg, Deps, DepsMut,
    Env, MessageInfo, QueryRequest, QueryResponse, Reply, Response, StdError, StdResult, SubMsg,
    SystemResult,
};

use crate::errors::ReflectError;
use crate::msg::{
    CapitalizedResponse, ChainResponse, CustomMsg, ExecuteMsg, InstantiateMsg, OwnerResponse,
    QueryMsg, RawResponse, SpecialQuery, SpecialResponse,
};
use crate::state::{load_config, load_reply, save_config, save_reply, State};

#[entry_point]
pub fn instantiate(
    deps: DepsMut<SpecialQuery>,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response<CustomMsg>> {
    let state = State { owner: info.sender };
    save_config(deps.storage, &state)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut<SpecialQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<CustomMsg>, ReflectError> {
    match msg {
        ExecuteMsg::ReflectMsg { msgs } => try_reflect(deps, env, info, msgs),
        ExecuteMsg::ReflectSubMsg { msgs } => try_reflect_subcall(deps, env, info, msgs),
        ExecuteMsg::ChangeOwner { owner } => try_change_owner(deps, env, info, owner),
    }
}

pub fn try_reflect(
    deps: DepsMut<SpecialQuery>,
    _env: Env,
    info: MessageInfo,
    msgs: Vec<CosmosMsg<CustomMsg>>,
) -> Result<Response<CustomMsg>, ReflectError> {
    let state = load_config(deps.storage)?;

    if info.sender != state.owner {
        return Err(ReflectError::NotCurrentOwner {
            expected: state.owner.into(),
            actual: info.sender.into(),
        });
    }

    if msgs.is_empty() {
        return Err(ReflectError::MessagesEmpty);
    }

    Ok(Response::new()
        .add_attribute("action", "reflect")
        .add_messages(msgs))
}

pub fn try_reflect_subcall(
    deps: DepsMut<SpecialQuery>,
    _env: Env,
    info: MessageInfo,
    msgs: Vec<SubMsg<CustomMsg>>,
) -> Result<Response<CustomMsg>, ReflectError> {
    let state = load_config(deps.storage)?;
    if info.sender != state.owner {
        return Err(ReflectError::NotCurrentOwner {
            expected: state.owner.into(),
            actual: info.sender.into(),
        });
    }

    if msgs.is_empty() {
        return Err(ReflectError::MessagesEmpty);
    }

    Ok(Response::new()
        .add_attribute("action", "reflect_subcall")
        .add_submessages(msgs))
}

pub fn try_change_owner(
    deps: DepsMut<SpecialQuery>,
    _env: Env,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response<CustomMsg>, ReflectError> {
    let api = deps.api;

    let mut state = load_config(deps.storage)?;

    if info.sender != state.owner {
        return Err(ReflectError::NotCurrentOwner {
            expected: state.owner.into(),
            actual: info.sender.into(),
        });
    }
    state.owner = api.addr_validate(&new_owner)?;

    save_config(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "change_owner")
        .add_attribute("owner", new_owner))
}

/// This just stores the result for future query
#[entry_point]
pub fn reply(deps: DepsMut<SpecialQuery>, _env: Env, msg: Reply) -> Result<Response, ReflectError> {
    save_reply(deps.storage, msg.id, &msg)?;
    Ok(Response::default())
}

#[entry_point]
pub fn query(deps: Deps<SpecialQuery>, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Owner {} => to_json_binary(&query_owner(deps)?),
        QueryMsg::Capitalized { text } => to_json_binary(&query_capitalized(deps, text)?),
        QueryMsg::Chain { request } => to_json_binary(&query_chain(deps, &request)?),
        QueryMsg::Raw { contract, key } => to_json_binary(&query_raw(deps, contract, key)?),
        QueryMsg::SubMsgResult { id } => to_json_binary(&query_subcall(deps, id)?),
    }
}

fn query_owner(deps: Deps<SpecialQuery>) -> StdResult<OwnerResponse> {
    let state = load_config(deps.storage)?;
    let resp = OwnerResponse {
        owner: state.owner.into(),
    };
    Ok(resp)
}

fn query_subcall(deps: Deps<SpecialQuery>, id: u64) -> StdResult<Reply> {
    load_reply(deps.storage, id)
}

fn query_capitalized(deps: Deps<SpecialQuery>, text: String) -> StdResult<CapitalizedResponse> {
    let req = SpecialQuery::Capitalized { text }.into();
    let response: SpecialResponse = deps.querier.query(&req)?;
    Ok(CapitalizedResponse { text: response.msg })
}

fn query_chain(
    deps: Deps<SpecialQuery>,
    request: &QueryRequest<SpecialQuery>,
) -> StdResult<ChainResponse> {
    let raw = to_json_vec(request).map_err(|serialize_err| {
        StdError::generic_err(format!("Serializing QueryRequest: {serialize_err}"))
    })?;
    match deps.querier.raw_query(&raw) {
        SystemResult::Err(system_err) => Err(StdError::generic_err(format!(
            "Querier system error: {system_err}"
        ))),
        SystemResult::Ok(ContractResult::Err(contract_err)) => Err(StdError::generic_err(format!(
            "Querier contract error: {contract_err}"
        ))),
        SystemResult::Ok(ContractResult::Ok(value)) => Ok(ChainResponse { data: value }),
    }
}

fn query_raw(deps: Deps<SpecialQuery>, contract: String, key: Binary) -> StdResult<RawResponse> {
    let response: Option<Vec<u8>> = deps.querier.query_wasm_raw(contract, key)?;
    Ok(RawResponse {
        data: response.unwrap_or_default().into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::mock_dependencies_with_custom_querier;
    use cosmwasm_std::testing::{message_info, mock_env, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{
        coin, coins, from_json, AllBalanceResponse, BankMsg, BankQuery, Binary, Event, StakingMsg,
        StdError, SubMsgResponse, SubMsgResult,
    };

    #[test]
    fn proper_instantialization() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);
        let creator = deps.api.addr_make("creator");

        let msg = InstantiateMsg {};
        let info = message_info(&creator, &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let value = query_owner(deps.as_ref()).unwrap();
        assert_eq!(value.owner, creator.to_string());
    }

    #[test]
    fn reflect() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);
        let creator = deps.api.addr_make("creator");

        let msg = InstantiateMsg {};
        let info = message_info(&creator, &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let payload = vec![BankMsg::Send {
            to_address: String::from("friend"),
            amount: coins(1, "token"),
        }
        .into()];

        let msg = ExecuteMsg::ReflectMsg {
            msgs: payload.clone(),
        };
        let info = message_info(&creator, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        let payload: Vec<_> = payload.into_iter().map(SubMsg::new).collect();
        assert_eq!(payload, res.messages);
    }

    #[test]
    fn reflect_requires_owner() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);
        let creator = deps.api.addr_make("creator");
        let random = deps.api.addr_make("random");

        let msg = InstantiateMsg {};
        let info = message_info(&creator, &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // signer is not owner
        let payload = vec![BankMsg::Send {
            to_address: String::from("friend"),
            amount: coins(1, "token"),
        }
        .into()];
        let msg = ExecuteMsg::ReflectMsg { msgs: payload };

        let info = message_info(&random, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res.unwrap_err() {
            ReflectError::NotCurrentOwner { .. } => {}
            err => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn reflect_reject_empty_msgs() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);
        let creator = deps.api.addr_make("creator");

        let msg = InstantiateMsg {};
        let info = message_info(&creator, &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = message_info(&creator, &[]);
        let payload = vec![];

        let msg = ExecuteMsg::ReflectMsg { msgs: payload };
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(err, ReflectError::MessagesEmpty);
    }

    #[test]
    fn reflect_multiple_messages() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);
        let creator = deps.api.addr_make("creator");

        let msg = InstantiateMsg {};
        let info = message_info(&creator, &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let payload = vec![
            BankMsg::Send {
                to_address: String::from("friend"),
                amount: coins(1, "token"),
            }
            .into(),
            // make sure we can pass through custom native messages
            CustomMsg::Raw(Binary::new(b"{\"foo\":123}".to_vec())).into(),
            CustomMsg::Debug("Hi, Dad!".to_string()).into(),
            StakingMsg::Delegate {
                validator: String::from("validator"),
                amount: coin(100, "ustake"),
            }
            .into(),
        ];

        let msg = ExecuteMsg::ReflectMsg {
            msgs: payload.clone(),
        };
        let info = message_info(&creator, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        let payload: Vec<_> = payload.into_iter().map(SubMsg::new).collect();
        assert_eq!(payload, res.messages);
    }

    #[test]
    fn change_owner_works() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);
        let creator = deps.api.addr_make("creator");

        let msg = InstantiateMsg {};
        let info = message_info(&creator, &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = message_info(&creator, &[]);
        let new_owner = deps.api.addr_make("friend");
        let msg = ExecuteMsg::ChangeOwner {
            owner: new_owner.to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should change state
        assert_eq!(0, res.messages.len());
        let value = query_owner(deps.as_ref()).unwrap();
        assert_eq!(value.owner, new_owner.as_str());
    }

    #[test]
    fn change_owner_requires_current_owner_as_sender() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);
        let creator = deps.api.addr_make("creator");
        let random = deps.api.addr_make("random");
        let friend = deps.api.addr_make("friend");

        let msg = InstantiateMsg {};
        let info = message_info(&creator, &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = message_info(&random, &[]);
        let msg = ExecuteMsg::ChangeOwner {
            owner: friend.to_string(),
        };

        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(
            err,
            ReflectError::NotCurrentOwner {
                expected: creator.to_string(),
                actual: random.to_string(),
            }
        );
    }

    #[test]
    fn change_owner_errors_for_invalid_new_address() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);
        let creator = deps.api.addr_make("creator");

        let msg = InstantiateMsg {};
        let info = message_info(&creator, &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = message_info(&creator, &[]);
        let msg = ExecuteMsg::ChangeOwner {
            owner: String::from("x"),
        };
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        match err {
            ReflectError::Std(StdError::GenericErr { msg, .. }) => {
                assert!(msg.contains("Error decoding bech32"))
            }
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn capitalized_query_works() {
        let deps = mock_dependencies_with_custom_querier(&[]);

        let msg = QueryMsg::Capitalized {
            text: "demo one".to_string(),
        };
        let response = query(deps.as_ref(), mock_env(), msg).unwrap();
        let value: CapitalizedResponse = from_json(response).unwrap();
        assert_eq!(value.text, "DEMO ONE");
    }

    #[test]
    fn chain_query_works() {
        let deps = mock_dependencies_with_custom_querier(&coins(123, "ucosm"));

        // with bank query
        #[allow(deprecated)]
        let msg = QueryMsg::Chain {
            request: BankQuery::AllBalances {
                address: MOCK_CONTRACT_ADDR.to_string(),
            }
            .into(),
        };
        let response = query(deps.as_ref(), mock_env(), msg).unwrap();
        let outer: ChainResponse = from_json(response).unwrap();
        let inner: AllBalanceResponse = from_json(outer.data).unwrap();
        assert_eq!(inner.amount, coins(123, "ucosm"));

        // with custom query
        let msg = QueryMsg::Chain {
            request: SpecialQuery::Ping {}.into(),
        };
        let response = query(deps.as_ref(), mock_env(), msg).unwrap();
        let outer: ChainResponse = from_json(response).unwrap();
        let inner: SpecialResponse = from_json(outer.data).unwrap();
        assert_eq!(inner.msg, "pong");
    }

    #[test]
    fn reflect_subcall() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);
        let creator = deps.api.addr_make("creator");

        let msg = InstantiateMsg {};
        let info = message_info(&creator, &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let id = 123u64;
        let payload = SubMsg::reply_always(
            BankMsg::Send {
                to_address: String::from("friend"),
                amount: coins(1, "token"),
            },
            id,
        );

        let msg = ExecuteMsg::ReflectSubMsg {
            msgs: vec![payload.clone()],
        };
        let info = message_info(&creator, &[]);
        let mut res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(1, res.messages.len());
        let msg = res.messages.pop().expect("must have a message");
        assert_eq!(payload, msg);
    }

    // this mocks out what happens after reflect_subcall
    #[test]
    fn reply_and_query() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);
        let creator = deps.api.addr_make("creator");

        let msg = InstantiateMsg {};
        let info = message_info(&creator, &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let id = 123u64;
        let payload = Binary::from(b"my dear");
        let data = Binary::from(b"foobar");
        let events = vec![Event::new("message").add_attribute("signer", "caller-addr")];
        let gas_used = 1234567u64;
        #[allow(deprecated)]
        let result = SubMsgResult::Ok(SubMsgResponse {
            events: events.clone(),
            data: Some(data.clone()),
            msg_responses: vec![],
        });
        let the_reply = Reply {
            id,
            payload,
            gas_used,
            result,
        };
        let res = reply(deps.as_mut(), mock_env(), the_reply).unwrap();
        assert_eq!(0, res.messages.len());

        // query for a non-existent id
        let qres = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::SubMsgResult { id: 65432 },
        );
        assert!(qres.is_err());

        // query for the real id
        let raw = query(deps.as_ref(), mock_env(), QueryMsg::SubMsgResult { id }).unwrap();
        let qres: Reply = from_json(raw).unwrap();
        assert_eq!(qres.id, id);
        let result = qres.result.unwrap();
        #[allow(deprecated)]
        {
            assert_eq!(result.data, Some(data));
        }
        assert_eq!(result.events, events);
    }
}
