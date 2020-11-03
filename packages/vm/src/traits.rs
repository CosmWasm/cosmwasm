use cosmwasm_std::{Binary, CanonicalAddr, ContractResult, HumanAddr, SystemResult};
#[cfg(feature = "iterator")]
use cosmwasm_std::{Order, KV};

use crate::ffi::BackendResult;

/// Holds all external dependencies of the contract.
/// Designed to allow easy dependency injection at runtime.
/// This cannot be copied or cloned since it would behave differently
/// for mock storages and a bridge storage in the VM.
pub struct Backend<S: Storage, A: Api, Q: Querier> {
    pub storage: S,
    pub api: A,
    pub querier: Q,
}

/// Access to the VM's backend storage, i.e. the chain
pub trait Storage {
    /// Returns Err on error.
    /// Returns Ok(None) when key does not exist.
    /// Returns Ok(Some(Vec<u8>)) when key exists.
    ///
    /// Note: Support for differentiating between a non-existent key and a key with empty value
    /// is not great yet and might not be possible in all backends. But we're trying to get there.
    fn get(&self, key: &[u8]) -> BackendResult<Option<Vec<u8>>>;

    /// Allows iteration over a set of key/value pairs, either forwards or backwards.
    /// Returns an interator ID that is unique within the Storage instance.
    ///
    /// The bound `start` is inclusive and `end` is exclusive.
    ///
    /// If `start` is lexicographically greater than or equal to `end`, an empty range is described, mo matter of the order.
    ///
    /// This call must not change data in the storage, but creating and storing a new iterator can be a mutating operation on
    /// the Storage implementation.
    /// The implementation must ensure that iterator IDs are assigned in a deterministic manner as this is
    /// environment data that is injected into the contract.
    #[cfg(feature = "iterator")]
    fn scan(
        &mut self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> BackendResult<u32>;

    /// Returns the next element of the iterator with the given ID.
    ///
    /// If the ID is not found, a BackendError::IteratorDoesNotExist is returned.
    ///
    /// This call must not change data in the storage, but incrementing an iterator can be a mutating operation on
    /// the Storage implementation.
    #[cfg(feature = "iterator")]
    fn next(&mut self, iterator_id: u32) -> BackendResult<Option<KV>>;

    fn set(&mut self, key: &[u8], value: &[u8]) -> BackendResult<()>;

    /// Removes a database entry at `key`.
    ///
    /// The current interface does not allow to differentiate between a key that existed
    /// before and one that didn't exist. See https://github.com/CosmWasm/cosmwasm/issues/290
    fn remove(&mut self, key: &[u8]) -> BackendResult<()>;
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
    fn canonical_address(&self, human: &HumanAddr) -> BackendResult<CanonicalAddr>;
    fn human_address(&self, canonical: &CanonicalAddr) -> BackendResult<HumanAddr>;
}

pub trait Querier {
    /// This is all that must be implemented for the Querier.
    /// This allows us to pass through binary queries from one level to another without
    /// knowing the custom format, or we can decode it, with the knowledge of the allowed
    /// types.
    ///
    /// The gas limit describes how much VM gas this particular query is allowed
    /// to comsume when measured separately from the rest of the contract.
    /// The returned gas info (in BackendResult) can exceed the gas limit in cases
    /// where the query could not be aborted exactly at the limit.
    fn query_raw(
        &self,
        request: &[u8],
        gas_limit: u64,
    ) -> BackendResult<SystemResult<ContractResult<Binary>>>;
}
