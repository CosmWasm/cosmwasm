use crate::errors::FfiResult;
use cosmwasm_std::{Binary, CanonicalAddr, HumanAddr, StdResult, SystemResult};
#[cfg(feature = "iterator")]
use cosmwasm_std::{Order, KV};

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

#[cfg(feature = "iterator")]
pub type StorageIteratorItem = FfiResult<(KV, u64)>;

/// ReadonlyStorage is access to the contracts persistent data store
pub trait ReadonlyStorage
where
    Self: 'static,
{
    /// Returns Err on error.
    /// Returns Ok(None) when key does not exist.
    /// Returns Ok(Some(Vec<u8>)) when key exists.
    ///
    /// Note: Support for differentiating between a non-existent key and a key with empty value
    /// is not great yet and might not be possible in all backends. But we're trying to get there.
    fn get(&self, key: &[u8]) -> FfiResult<(Option<Vec<u8>>, u64)>;

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
    ) -> FfiResult<Box<dyn Iterator<Item = StorageIteratorItem> + 'a>>;
}

// Storage extends ReadonlyStorage to give mutable access
pub trait Storage: ReadonlyStorage {
    fn set(&mut self, key: &[u8], value: &[u8]) -> FfiResult<u64>;
    /// Removes a database entry at `key`.
    ///
    /// The current interface does not allow to differentiate between a key that existed
    /// before and one that didn't exist. See https://github.com/CosmWasm/cosmwasm/issues/290
    fn remove(&mut self, key: &[u8]) -> FfiResult<u64>;
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
    fn canonical_address(&self, human: &HumanAddr) -> FfiResult<CanonicalAddr>;
    fn human_address(&self, canonical: &CanonicalAddr) -> FfiResult<HumanAddr>;
}

/// A short-hand alias for the three-level query result
/// 1. Passing the query message to the backend
/// 2. Accessing the contract
/// 3. Executing query in the contract
pub type QuerierResult = FfiResult<SystemResult<StdResult<Binary>>>;

pub trait Querier: Clone + Send + 'static {
    /// raw_query is all that must be implemented for the Querier.
    /// This allows us to pass through binary queries from one level to another without
    /// knowing the custom format, or we can decode it, with the knowledge of the allowed
    /// types. People using the querier probably want one of the simpler auto-generated
    /// helper methods
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult;
}
