use cosmwasm_std::{Binary, CanonicalAddr, ContractResult, HumanAddr, SystemResult};
#[cfg(feature = "iterator")]
use cosmwasm_std::{Order, KV};

#[cfg(feature = "iterator")]
use crate::ffi::FfiError;
use crate::ffi::FfiResult;

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
pub trait StorageIterator {
    fn next(&mut self) -> FfiResult<Option<KV>>;

    /// Collects all elements, ignoring gas costs
    fn elements(mut self) -> Result<Vec<KV>, FfiError>
    where
        Self: Sized,
    {
        let mut out: Vec<KV> = Vec::new();
        loop {
            let (result, _gas_info) = self.next();
            match result {
                Ok(Some(kv)) => out.push(kv),
                Ok(None) => break,
                Err(err) => return Err(err),
            }
        }
        Ok(out)
    }
}

#[cfg(feature = "iterator")]
impl<I: StorageIterator + ?Sized> StorageIterator for Box<I> {
    fn next(&mut self) -> FfiResult<Option<KV>> {
        (**self).next()
    }
}

/// Access to the VM's backend storage, i.e. the chain
pub trait Storage
where
    Self: 'static,
{
    /// Returns Err on error.
    /// Returns Ok(None) when key does not exist.
    /// Returns Ok(Some(Vec<u8>)) when key exists.
    ///
    /// Note: Support for differentiating between a non-existent key and a key with empty value
    /// is not great yet and might not be possible in all backends. But we're trying to get there.
    fn get(&self, key: &[u8]) -> FfiResult<Option<Vec<u8>>>;

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
    ) -> FfiResult<Box<dyn StorageIterator + 'a>>;

    fn set(&mut self, key: &[u8], value: &[u8]) -> FfiResult<()>;

    /// Removes a database entry at `key`.
    ///
    /// The current interface does not allow to differentiate between a key that existed
    /// before and one that didn't exist. See https://github.com/CosmWasm/cosmwasm/issues/290
    fn remove(&mut self, key: &[u8]) -> FfiResult<()>;
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

pub trait Querier {
    /// This is all that must be implemented for the Querier.
    /// This allows us to pass through binary queries from one level to another without
    /// knowing the custom format, or we can decode it, with the knowledge of the allowed
    /// types.
    ///
    /// The gas limit describes how much VM gas this particular query is allowed
    /// to comsume when measured separately from the rest of the contract.
    /// The returned gas info (in FfiResult) can exceed the gas limit in cases
    /// where the query could not be aborted exactly at the limit.
    fn query_raw(
        &self,
        request: &[u8],
        gas_limit: u64,
    ) -> FfiResult<SystemResult<ContractResult<Binary>>>;
}
