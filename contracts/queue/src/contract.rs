use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use cosmwasm_std::{
    from_slice, to_vec, Api, Binary, Env, Extern, Order, ParseErr, Response, Result, SerializeErr,
    Storage,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {}

// we store one entry for each item in the queue
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub value: i32,
}

// failure modes to help test wasmd, based on this comment
// https://github.com/cosmwasm/wasmd/issues/8#issuecomment-576146751
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum HandleMsg {
    // Push will add some value to the end of list
    Push { value: i32 },
    // Pop will remove value from start of the list
    Pop {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum QueryMsg {
    // how many items are in the queue
    Count {},
    // total of all values in the queue
    Sum {},
    //    // first element in the queue
    //    First {},
    //    // last element in the queue
    //    Last {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
    pub count: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SumResponse {
    pub sum: i32,
}

// init is a no-op, just empty data
pub fn init<S: Storage, A: Api>(
    _deps: &mut Extern<S, A>,
    _env: Env,
    _msg: InitMsg,
) -> Result<Response> {
    Ok(Response::default())
}

pub fn handle<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    env: Env,
    msg: HandleMsg,
) -> Result<Response> {
    match msg {
        HandleMsg::Push { value } => do_push(deps, env, value),
        HandleMsg::Pop {} => do_pop(deps, env),
    }
}

const FIRST_KEY: u8 = 1;

fn do_push<S: Storage, A: Api>(deps: &mut Extern<S, A>, _env: Env, value: i32) -> Result<Response> {
    // find the last element in the queue and extract key
    let last = deps
        .storage
        .range(None, None, Order::Descending)
        .next()
        .map(|(k, _)| k);

    // all keys are one byte
    let my_key = match last {
        Some(k) => k[0] + 1,
        None => FIRST_KEY,
    };
    let data = to_vec(&State { value }).context(SerializeErr { kind: "State" })?;

    deps.storage.set(&[my_key], &data);
    Ok(Response::default())
}

fn do_pop<S: Storage, A: Api>(deps: &mut Extern<S, A>, _env: Env) -> Result<Response> {
    // find the first element in the queue and extract value
    let first = deps.storage.range(None, None, Order::Ascending).next();

    let mut res = Response::default();
    if let Some((k, v)) = first {
        // remove from storage and return old value
        deps.storage.remove(&k);
        res.data = Some(Binary(v));
        Ok(res)
    } else {
        Ok(res)
    }
}

pub fn query<S: Storage, A: Api>(deps: &Extern<S, A>, msg: QueryMsg) -> Result<Vec<u8>> {
    match msg {
        QueryMsg::Count {} => query_count(deps),
        QueryMsg::Sum {} => query_sum(deps),
    }
}

fn query_count<S: Storage, A: Api>(deps: &Extern<S, A>) -> Result<Vec<u8>> {
    let count = deps.storage.range(None, None, Order::Ascending).count() as i32;
    to_vec(&CountResponse { count }).context(SerializeErr {
        kind: "CountResponse",
    })
}

fn query_sum<S: Storage, A: Api>(deps: &Extern<S, A>) -> Result<Vec<u8>> {
    let values: Result<Vec<State>> = deps
        .storage
        .range(None, None, Order::Ascending)
        .map(|(_, v)| from_slice(&v).context(ParseErr { kind: "State" }))
        .collect();
    let sum = values?.iter().fold(0, |s, v| s + v.value);
    to_vec(&SumResponse { sum }).context(SerializeErr {
        kind: "SumResponse",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockApi, MockStorage};
    use cosmwasm_std::{coin, HumanAddr};

    fn create_contract() -> (Extern<MockStorage, MockApi>, Env) {
        let mut deps = mock_dependencies(20);
        let creator = HumanAddr(String::from("creator"));
        let env = mock_env(&deps.api, creator.as_str(), &coin("1000", "earth"), &[]);
        let res = init(&mut deps, env.clone(), InitMsg {}).unwrap();
        assert_eq!(0, res.messages.len());
        (deps, env)
    }

    fn get_count(deps: &Extern<MockStorage, MockApi>) -> i32 {
        let data = query(deps, QueryMsg::Count {}).unwrap();
        let res: CountResponse = from_slice(&data).unwrap();
        res.count
    }

    fn get_sum(deps: &Extern<MockStorage, MockApi>) -> i32 {
        let data = query(deps, QueryMsg::Sum {}).unwrap();
        let res: SumResponse = from_slice(&data).unwrap();
        res.sum
    }

    #[test]
    fn init_and_query() {
        let (deps, _) = create_contract();
        assert_eq!(get_count(&deps), 0);
        assert_eq!(get_sum(&deps), 0);
    }

    #[test]
    fn push_and_query() {
        let (mut deps, env) = create_contract();
        handle(&mut deps, env.clone(), HandleMsg::Push { value: 25 }).unwrap();
        assert_eq!(get_count(&deps), 1);
        assert_eq!(get_sum(&deps), 25);
    }

    #[test]
    fn multiple_push() {
        let (mut deps, env) = create_contract();
        handle(&mut deps, env.clone(), HandleMsg::Push { value: 25 }).unwrap();
        handle(&mut deps, env.clone(), HandleMsg::Push { value: 35 }).unwrap();
        handle(&mut deps, env.clone(), HandleMsg::Push { value: 45 }).unwrap();
        assert_eq!(get_count(&deps), 3);
        assert_eq!(get_sum(&deps), 105);
    }

    #[test]
    fn push_and_pop() {
        let (mut deps, env) = create_contract();
        handle(&mut deps, env.clone(), HandleMsg::Push { value: 25 }).unwrap();
        handle(&mut deps, env.clone(), HandleMsg::Push { value: 17 }).unwrap();
        let res = handle(&mut deps, env.clone(), HandleMsg::Pop {}).unwrap();
        // ensure we popped properly
        assert!(res.data.is_some());
        let data = res.data.unwrap();
        let state: State = from_slice(data.as_slice()).unwrap();
        assert_eq!(state.value, 25);

        assert_eq!(get_count(&deps), 1);
        assert_eq!(get_sum(&deps), 17);
    }
}
