use serde::{de::DeserializeOwned, Serialize};

use crate::addresses::{CanonicalAddr, HumanAddr};
use crate::binary::Binary;
use crate::coins::Coin;
use crate::errors::{StdError, StdResult};
#[cfg(feature = "iterator")]
use crate::iterator::{Order, KV};
use crate::query::{
    AllBalanceResponse, BalanceResponse, BankQuery, CustomQuery, QueryRequest, WasmQuery,
};
#[cfg(feature = "staking")]
use crate::query::{
    AllDelegationsResponse, BondedDenomResponse, Delegation, DelegationResponse, FullDelegation,
    StakingQuery, Validator, ValidatorsResponse,
};
use crate::results::{ContractResult, SystemResult};
use crate::serde::{from_binary, to_binary, to_vec};
use crate::types::Empty;

/// Holds all external dependencies of the contract.
/// Designed to allow easy dependency injection at runtime.
/// This cannot be copied or cloned since it would behave differently
/// for mock storages and a bridge storage in the VM.
pub struct Extern<S: Storage, A: Api, Q: Querier> {
    pub storage: S,
    pub api: A,
    pub querier: Q,
}

impl<S: Storage, A: Api, Q: Querier> Extern<S, A, Q> {
    /// change_querier is a helper mainly for test code when swapping out the Querier
    /// from the auto-generated one from mock_dependencies. This changes the type of
    /// Extern so replaces requires some boilerplate.
    pub fn change_querier<T: Querier, F: Fn(Q) -> T>(self, transform: F) -> Extern<S, A, T> {
        Extern {
            storage: self.storage,
            api: self.api,
            querier: transform(self.querier),
        }
    }
}

/// ReadonlyStorage is access to the contracts persistent data store
pub trait ReadonlyStorage {
    /// Returns None when key does not exist.
    /// Returns Some(Vec<u8>) when key exists.
    ///
    /// Note: Support for differentiating between a non-existent key and a key with empty value
    /// is not great yet and might not be possible in all backends. But we're trying to get there.
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;

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
    ) -> Box<dyn Iterator<Item = KV> + 'a>;
}

// Storage extends ReadonlyStorage to give mutable access
pub trait Storage: ReadonlyStorage {
    fn set(&mut self, key: &[u8], value: &[u8]);
    /// Removes a database entry at `key`.
    ///
    /// The current interface does not allow to differentiate between a key that existed
    /// before and one that didn't exist. See https://github.com/CosmWasm/cosmwasm/issues/290
    fn remove(&mut self, key: &[u8]);
}

/// Api are callbacks to system functions implemented outside of the wasm modules.
/// Currently it just supports address conversion but we could add eg. crypto functions here.
///
/// This is a trait to allow mocks in the test code. Its members have a read-only
/// reference to the Api instance to allow accessing configuration.
/// Implementations must not have mutable state, such that an instance can freely
/// be copied and shared between threads without affecting the behaviour.
/// Given an Api instance, all members should return the same value when called with the same
/// arguments. In particular this means the result must not depend in the state of the chain.
/// If you need to access chaim state, you probably want to use the Querier.
/// Side effects (such as logging) are allowed.
///
/// We can use feature flags to opt-in to non-essential methods
/// for backwards compatibility in systems that don't have them all.
pub trait Api: Copy + Clone + Send {
    fn canonical_address(&self, human: &HumanAddr) -> StdResult<CanonicalAddr>;
    fn human_address(&self, canonical: &CanonicalAddr) -> StdResult<HumanAddr>;
    /// Emits a debugging message that is handled depending on the environment (typically printed to console or ignored).
    /// Those messages are not persisted to chain.
    fn debug(&self, message: &str);
}

/// A short-hand alias for the two-level query result (1. accessing the contract, 2. executing query in the contract)
pub type QuerierResult = SystemResult<ContractResult<Binary>>;

pub trait Querier {
    /// raw_query is all that must be implemented for the Querier.
    /// This allows us to pass through binary queries from one level to another without
    /// knowing the custom format, or we can decode it, with the knowledge of the allowed
    /// types. People using the querier probably want one of the simpler auto-generated
    /// helper methods
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult;

    /// query is a shorthand for custom_query when we are not using a custom type,
    /// this allows us to avoid specifying "Empty" in all the type definitions.
    fn query<T: DeserializeOwned>(&self, request: &QueryRequest<Empty>) -> StdResult<T> {
        self.custom_query(request)
    }

    /// Makes the query and parses the response. Also handles custom queries,
    /// so you need to specify the custom query type in the function parameters.
    /// If you are no using a custom query, just use `query` for easier interface.
    ///
    /// Any error (System Error, Error or called contract, or Parse Error) are flattened into
    /// one level. Only use this if you don't need to check the SystemError
    /// eg. If you don't differentiate between contract missing and contract returned error
    fn custom_query<C: CustomQuery, U: DeserializeOwned>(
        &self,
        request: &QueryRequest<C>,
    ) -> StdResult<U> {
        let raw = to_vec(request).map_err(|serialize_err| {
            StdError::generic_err(format!("Serializing QueryRequest: {}", serialize_err))
        })?;
        match self.raw_query(&raw) {
            SystemResult::Err(system_err) => Err(StdError::generic_err(format!(
                "Querier system error: {}",
                system_err
            ))),
            SystemResult::Ok(ContractResult::Err(contract_err)) => Err(StdError::generic_err(
                format!("Querier contract error: {}", contract_err),
            )),
            SystemResult::Ok(ContractResult::Ok(value)) => from_binary(&value),
        }
    }

    fn query_balance<U: Into<HumanAddr>>(&self, address: U, denom: &str) -> StdResult<Coin> {
        let request = BankQuery::Balance {
            address: address.into(),
            denom: denom.to_string(),
        }
        .into();
        let res: BalanceResponse = self.query(&request)?;
        Ok(res.amount)
    }

    fn query_all_balances<U: Into<HumanAddr>>(&self, address: U) -> StdResult<Vec<Coin>> {
        let request = BankQuery::AllBalances {
            address: address.into(),
        }
        .into();
        let res: AllBalanceResponse = self.query(&request)?;
        Ok(res.amount)
    }

    // this queries another wasm contract. You should know a priori the proper types for T and U
    // (response and request) based on the contract API
    fn query_wasm_smart<T: DeserializeOwned, U: Serialize, V: Into<HumanAddr>>(
        &self,
        contract: V,
        msg: &U,
    ) -> StdResult<T> {
        let request = WasmQuery::Smart {
            contract_addr: contract.into(),
            msg: to_binary(msg)?,
        }
        .into();
        self.query(&request)
    }

    // this queries the raw storage from another wasm contract.
    // you must know the exact layout and are implementation dependent
    // (not tied to an interface like query_wasm_smart)
    // that said, if you are building a few contracts together, this is a much cheaper approach
    //
    // Similar return value to Storage.get(). Returns Some(val) or None if the data is there.
    // It only returns error on some runtime issue, not on any data cases.
    fn query_wasm_raw<T: Into<HumanAddr>, U: Into<Binary>>(
        &self,
        contract: T,
        key: U,
    ) -> StdResult<Option<Vec<u8>>> {
        let request: QueryRequest<Empty> = WasmQuery::Raw {
            contract_addr: contract.into(),
            key: key.into(),
        }
        .into();
        // we cannot use query, as it will try to parse the binary data, when we just want to return it,
        // so a bit of code copy here...
        let raw = to_vec(&request).map_err(|serialize_err| {
            StdError::generic_err(format!("Serializing QueryRequest: {}", serialize_err))
        })?;
        match self.raw_query(&raw) {
            SystemResult::Err(system_err) => Err(StdError::generic_err(format!(
                "Querier system error: {}",
                system_err
            ))),
            SystemResult::Ok(ContractResult::Err(contract_err)) => Err(StdError::generic_err(
                format!("Querier contract error: {}", contract_err),
            )),
            SystemResult::Ok(ContractResult::Ok(value)) => {
                if value.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(value.into()))
                }
            }
        }
    }

    #[cfg(feature = "staking")]
    fn query_validators(&self) -> StdResult<Vec<Validator>> {
        let request = StakingQuery::Validators {}.into();
        let res: ValidatorsResponse = self.query(&request)?;
        Ok(res.validators)
    }

    #[cfg(feature = "staking")]
    fn query_bonded_denom(&self) -> StdResult<String> {
        let request = StakingQuery::BondedDenom {}.into();
        let res: BondedDenomResponse = self.query(&request)?;
        Ok(res.denom)
    }

    #[cfg(feature = "staking")]
    fn query_all_delegations<U: Into<HumanAddr>>(
        &self,
        delegator: U,
    ) -> StdResult<Vec<Delegation>> {
        let request = StakingQuery::AllDelegations {
            delegator: delegator.into(),
        }
        .into();
        let res: AllDelegationsResponse = self.query(&request)?;
        Ok(res.delegations)
    }

    #[cfg(feature = "staking")]
    fn query_delegation<U: Into<HumanAddr>>(
        &self,
        delegator: U,
        validator: U,
    ) -> StdResult<Option<FullDelegation>> {
        let request = StakingQuery::Delegation {
            delegator: delegator.into(),
            validator: validator.into(),
        }
        .into();
        let res: DelegationResponse = self.query(&request)?;
        Ok(res.delegation)
    }
}
