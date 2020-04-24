use serde::de::DeserializeOwned;

use crate::api::{ApiResult, SystemResult};
use crate::encoding::Binary;
use crate::errors::{dyn_contract_err, StdResult};
#[cfg(feature = "iterator")]
use crate::iterator::{Order, KV};
use crate::query::{AllBalanceResponse, BalanceResponse, BankQuery, QueryRequest};
use crate::serde::from_binary;
use crate::to_vec;
use crate::types::{CanonicalAddr, HumanAddr, Never};
use serde::Serialize;

/// Holds all external dependencies of the contract.
/// Designed to allow easy dependency injection at runtime.
/// This cannot be copied or cloned since it would behave differently
/// for mock storages and a bridge storage in the VM.
pub struct Extern<S: Storage, A: Api, Q: Querier> {
    pub storage: S,
    pub api: A,
    pub querier: Q,
}

/// ReadonlyStorage is access to the contracts persistent data store
pub trait ReadonlyStorage {
    /// Returns Err on error.
    /// Returns Ok(None) when key does not exist.
    /// Returns Ok(Some(Vec<u8>)) when key exists.
    ///
    /// Note: Support for differentiating between a non-existent key and a key with empty value
    /// is not great yet and might not be possible in all backends. But we're trying to get there.
    fn get(&self, key: &[u8]) -> StdResult<Option<Vec<u8>>>;

    #[cfg(feature = "iterator")]
    /// Allows iteration over a set of key/value pairs, either forwards or backwards.
    ///
    /// The bound `start` is inclusive and `end` is exclusive.
    ///
    /// If `start` is lexicographically greater than or equal to `end`, an empty range is described, mo matter of the order.
    fn range<'a>(
        &'a self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> StdResult<Box<dyn Iterator<Item = StdResult<KV>> + 'a>>;
}

// Storage extends ReadonlyStorage to give mutable access
pub trait Storage: ReadonlyStorage {
    fn set(&mut self, key: &[u8], value: &[u8]) -> StdResult<()>;
    /// Removes a database entry at `key`.
    ///
    /// The current interface does not allow to differentiate between a key that existed
    /// before and one that didn't exist. See https://github.com/CosmWasm/cosmwasm/issues/290
    fn remove(&mut self, key: &[u8]) -> StdResult<()>;
}

/// Api are callbacks to system functions defined outside of the wasm modules.
/// This is a trait to allow Mocks in the test code.
///
/// Currently it just supports address conversion, we could add eg. crypto functions here.
/// These should all be pure (stateless) functions. If you need state, you probably want
/// to use the Querier.
///
/// We can use feature flags to opt-in to non-essential methods
/// for backwards compatibility in systems that don't have them all.
pub trait Api: Copy + Clone + Send {
    fn canonical_address(&self, human: &HumanAddr) -> StdResult<CanonicalAddr>;
    fn human_address(&self, canonical: &CanonicalAddr) -> StdResult<HumanAddr>;
}

/// A short-hand alias for the two-level query result (1. accessing the contract, 2. executing query in the contract)
pub type QuerierResult = SystemResult<ApiResult<Binary>>;

pub trait Querier: Clone + Send {
    // TODO: rename this... right now this is demo to see if we can pipe it through unparsed in the VM
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult;

    fn query<T: Serialize>(&self, request: &QueryRequest<T>) -> QuerierResult {
        match to_vec(request) {
            Ok(raw) => self.raw_query(&raw),
            // TODO: maybe I want to make this a SystemError::InvalidRequest ?
            Err(e) => Ok(Err(e.into())),
        }
    }

    /// Makes the query and parses the response.
    /// Any error (System Error, Error or called contract, or Parse Error) are flattened into
    /// one level. Only use this if you don't have checks on other side.
    ///
    /// eg. When querying another contract, you will often want some way to detect/handle if there
    /// is no contract there.
    fn parse_query<T: Serialize, U: DeserializeOwned>(
        &self,
        request: &QueryRequest<T>,
    ) -> StdResult<U> {
        match self.query(&request) {
            Err(sys_err) => dyn_contract_err(format!("Querier system error: {}", sys_err)),
            Ok(Err(err)) => dyn_contract_err(format!("Querier contract error: {}", err)),
            // in theory we would process the response, but here it is the same type, so just pass through
            Ok(Ok(res)) => from_binary(&res),
        }
    }

    fn query_balance<U: Into<HumanAddr>>(
        &self,
        address: U,
        denom: &str,
    ) -> StdResult<BalanceResponse> {
        let request = QueryRequest::Bank(BankQuery::Balance {
            address: address.into(),
            denom: denom.to_string(),
        });
        self.parse_query::<Never, _>(&request)
    }

    fn query_all_balances<U: Into<HumanAddr>>(&self, address: U) -> StdResult<AllBalanceResponse> {
        let request = QueryRequest::Bank(BankQuery::AllBalances {
            address: address.into(),
        });
        self.parse_query::<Never, _>(&request)
    }
}
