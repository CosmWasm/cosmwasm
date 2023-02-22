use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub enum ExecuteMsg {
    // Enqueue will add some value to the end of list
    Enqueue { value: i32 },
    // Dequeue will remove value from start of the list
    Dequeue {},
}

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // how many items are in the queue
    #[returns(CountResponse)]
    Count {},
    // total of all values in the queue
    #[returns(SumResponse)]
    Sum {},
    // Reducer holds open two iterators at once
    #[returns(ReducerResponse)]
    Reducer {},
    #[returns(ListResponse)]
    List {},
    /// Opens the given number of iterators for no reason other than testing.
    /// Returns and `Empty` response.
    #[returns(cosmwasm_std::Empty)]
    OpenIterators { count: u32 },
}

#[cw_serde]
pub struct CountResponse {
    pub count: u32,
}

#[cw_serde]
pub struct SumResponse {
    pub sum: i32,
}

#[cw_serde]
// the Vec contains pairs for every element in the queue
// (value of item i, sum of all elements where value > value[i])
pub struct ReducerResponse {
    pub counters: Vec<(i32, i32)>,
}

#[cw_serde]
pub struct ListResponse {
    /// List an empty range, both bounded
    pub empty: Vec<u32>,
    /// List all IDs lower than 0x20
    pub early: Vec<u32>,
    /// List all IDs starting from 0x20
    pub late: Vec<u32>,
}
