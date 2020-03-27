use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    from_slice, to_vec, Api, Binary, Env, Extern, Order, Response, Result,
    Storage,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {}

// we store one entry for each item in the queue
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Item {
    pub value: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum HandleMsg {
    // Enqueue will add some value to the end of list
    Enqueue { value: i32 },
    // Dequeue will remove value from start of the list
    Dequeue {},
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
    pub count: u32,
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
        HandleMsg::Enqueue { value } => enqueue(deps, env, value),
        HandleMsg::Dequeue {} => dequeue(deps, env),
    }
}

const FIRST_KEY: u8 = 0;

fn enqueue<S: Storage, A: Api>(deps: &mut Extern<S, A>, _env: Env, value: i32) -> Result<Response> {
    // find the last element in the queue and extract key
    let last_key = deps
        .storage
        .range(None, None, Order::Descending)
        .next()
        .map(|(k, _)| k);

    // all keys are one byte
    let my_key = match last_key {
        Some(k) => k[0] + 1,
        None => FIRST_KEY,
    };
    let data = to_vec(&Item { value })?;

    deps.storage.set(&[my_key], &data);
    Ok(Response::default())
}

fn dequeue<S: Storage, A: Api>(deps: &mut Extern<S, A>, _env: Env) -> Result<Response> {
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
    let count = deps.storage.range(None, None, Order::Ascending).count() as u32;
    to_vec(&CountResponse { count })
}

fn query_sum<S: Storage, A: Api>(deps: &Extern<S, A>) -> Result<Vec<u8>> {
    let values: Result<Vec<Item>> = deps
        .storage
        .range(None, None, Order::Ascending)
        .map(|(_, v)| from_slice(&v))
        .collect();
    let sum = values?.iter().fold(0, |s, v| s + v.value);
    to_vec(&SumResponse { sum })
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

    fn get_count(deps: &Extern<MockStorage, MockApi>) -> u32 {
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
        handle(&mut deps, env.clone(), HandleMsg::Enqueue { value: 25 }).unwrap();
        assert_eq!(get_count(&deps), 1);
        assert_eq!(get_sum(&deps), 25);
    }

    #[test]
    fn multiple_push() {
        let (mut deps, env) = create_contract();
        handle(&mut deps, env.clone(), HandleMsg::Enqueue { value: 25 }).unwrap();
        handle(&mut deps, env.clone(), HandleMsg::Enqueue { value: 35 }).unwrap();
        handle(&mut deps, env.clone(), HandleMsg::Enqueue { value: 45 }).unwrap();
        assert_eq!(get_count(&deps), 3);
        assert_eq!(get_sum(&deps), 105);
    }

    #[test]
    fn push_and_pop() {
        let (mut deps, env) = create_contract();
        handle(&mut deps, env.clone(), HandleMsg::Enqueue { value: 25 }).unwrap();
        handle(&mut deps, env.clone(), HandleMsg::Enqueue { value: 17 }).unwrap();
        let res = handle(&mut deps, env.clone(), HandleMsg::Dequeue {}).unwrap();
        // ensure we popped properly
        assert!(res.data.is_some());
        let data = res.data.unwrap();
        let state: Item = from_slice(data.as_slice()).unwrap();
        assert_eq!(state.value, 25);

        assert_eq!(get_count(&deps), 1);
        assert_eq!(get_sum(&deps), 17);
    }
}
