use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    from_slice, to_binary, to_vec, Api, Binary, Env, Extern, HandleResponse, InitResponse, Order,
    Querier, QueryResponse, StdResult, Storage,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {}

// we store one entry for each item in the queue
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Item {
    pub value: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // Enqueue will add some value to the end of list
    Enqueue { value: i32 },
    // Dequeue will remove value from start of the list
    Dequeue {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // how many items are in the queue
    Count {},
    // total of all values in the queue
    Sum {},
    // Reducer holds open two iterators at once
    Reducer {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
    pub count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SumResponse {
    pub sum: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// the Vec contains pairs for every element in the queue
// (value of item i, sum of all elements where value > value[i])
pub struct ReducerResponse {
    pub counters: Vec<(i32, i32)>,
}

// init is a no-op, just empty data
pub fn init<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: InitMsg,
) -> StdResult<InitResponse> {
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Enqueue { value } => enqueue(deps, env, value),
        HandleMsg::Dequeue {} => dequeue(deps, env),
    }
}

const FIRST_KEY: u8 = 0;

fn enqueue<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    value: i32,
) -> StdResult<HandleResponse> {
    // find the last element in the queue and extract key
    let last_item = deps.storage.range(None, None, Order::Descending).next();

    let new_key = match last_item {
        None => FIRST_KEY,
        Some((key, _)) => {
            key[0] + 1 // all keys are one byte
        }
    };
    let new_value = to_vec(&Item { value })?;

    deps.storage.set(&[new_key], &new_value);
    Ok(HandleResponse::default())
}

fn dequeue<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
) -> StdResult<HandleResponse> {
    // find the first element in the queue and extract value
    let first = deps.storage.range(None, None, Order::Ascending).next();

    let mut res = HandleResponse::default();
    if let Some((key, value)) = first {
        // remove from storage and return old value
        deps.storage.remove(&key);
        res.data = Some(Binary(value));
        Ok(res)
    } else {
        Ok(res)
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Count {} => to_binary(&query_count(deps)?),
        QueryMsg::Sum {} => to_binary(&query_sum(deps)?),
        QueryMsg::Reducer {} => to_binary(&query_reducer(deps)?),
    }
}

fn query_count<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<CountResponse> {
    let count = deps.storage.range(None, None, Order::Ascending).count() as u32;
    Ok(CountResponse { count })
}

fn query_sum<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<SumResponse> {
    let values: StdResult<Vec<Item>> = deps
        .storage
        .range(None, None, Order::Ascending)
        .map(|(_, v)| from_slice(&v))
        .collect();
    let sum = values?.iter().fold(0, |s, v| s + v.value);
    Ok(SumResponse { sum })
}

fn query_reducer<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<ReducerResponse> {
    let mut out: Vec<(i32, i32)> = vec![];
    // val: StdResult<Item>
    for val in deps
        .storage
        .range(None, None, Order::Ascending)
        .map(|(_, v)| from_slice::<Item>(&v))
    {
        // this returns error on parse error
        let my_val = val?.value;
        // now, let's do second iterator
        let sum: i32 = deps
            .storage
            .range(None, None, Order::Ascending)
            // get value. ignore parse errors, just count as 0
            .map(|(_, v)| {
                from_slice::<Item>(&v)
                    .map(|v| v.value)
                    .expect("error in item")
            })
            .filter(|v| *v > my_val)
            .sum();
        out.push((my_val, sum))
    }
    Ok(ReducerResponse { counters: out })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage};

    fn create_contract() -> (Extern<MockStorage, MockApi, MockQuerier>, Env) {
        let mut deps = mock_dependencies(20, &coins(1000, "earth"));
        let env = mock_env(&deps.api, "creator", &coins(1000, "earth"));
        let res = init(&mut deps, env.clone(), InitMsg {}).unwrap();
        assert_eq!(0, res.messages.len());
        (deps, env)
    }

    fn get_count(deps: &Extern<MockStorage, MockApi, MockQuerier>) -> u32 {
        query_count(deps).unwrap().count
    }

    fn get_sum(deps: &Extern<MockStorage, MockApi, MockQuerier>) -> i32 {
        query_sum(deps).unwrap().sum
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

    #[test]
    fn push_and_reduce() {
        let (mut deps, env) = create_contract();
        handle(&mut deps, env.clone(), HandleMsg::Enqueue { value: 40 }).unwrap();
        handle(&mut deps, env.clone(), HandleMsg::Enqueue { value: 15 }).unwrap();
        handle(&mut deps, env.clone(), HandleMsg::Enqueue { value: 85 }).unwrap();
        handle(&mut deps, env.clone(), HandleMsg::Enqueue { value: -10 }).unwrap();
        assert_eq!(get_count(&deps), 4);
        assert_eq!(get_sum(&deps), 130);
        let counters = query_reducer(&deps).unwrap().counters;
        assert_eq!(counters, vec![(40, 85), (15, 125), (85, 0), (-10, 140)]);
    }
}
