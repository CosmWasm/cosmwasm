use crate::api::{ApiError, ApiResult, ApiSystemError};
use crate::encoding::Binary;
use crate::errors::Result;
#[cfg(feature = "iterator")]
use crate::iterator::{Order, KV};
use crate::query::QueryRequest;
use crate::types::{CanonicalAddr, HumanAddr};

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
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;

    #[cfg(feature = "iterator")]
    /// range allows iteration over a set of keys, either forwards or backwards
    /// start is inclusive and end is exclusive
    /// start must be lexicographically before end
    fn range<'a>(
        &'a self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = KV> + 'a>;
}

// Storage extends ReadonlyStorage to give mutable access
pub trait Storage: ReadonlyStorage {
    fn set(&mut self, key: &[u8], value: &[u8]) -> Result<()>;
    fn remove(&mut self, key: &[u8]) -> Result<()>;
}

/// Api are callbacks to system functions defined outside of the wasm modules.
/// This is a trait to allow Mocks in the test code.
///
/// Currently it just supports address conversion, we could add eg. crypto functions here.
/// These should all be pure (stateless) functions. If you need state, you probably want
/// to use the Querier (TODO)
///
/// We can use feature flags to opt-in to non-essential methods
/// for backwards compatibility in systems that don't have them all.
pub trait Api: Copy + Clone + Send {
    fn canonical_address(&self, human: &HumanAddr) -> Result<CanonicalAddr>;
    fn human_address(&self, canonical: &CanonicalAddr) -> Result<HumanAddr>;
}

// QuerierResponse is a short-hand alias as this type is long to write
pub type QuerierResponse = Result<Result<Binary, ApiError>, ApiSystemError>;

// ApiQuerierResponse is QuerierResponse converted to be serialized (short-hand for other modules)
pub type ApiQuerierResponse = ApiResult<ApiResult<Binary, ApiError>, ApiSystemError>;

pub trait Querier: Clone + Send {
    // Note: ApiError type can be serialized, and the below can be reconstituted over a WASM/FFI call.
    // Since this is information that is returned from outside, we define it this way.
    //
    // ApiResult is a format that can capture this info in a serialized form. We parse it into
    // a typical Result for the implementing object
    fn query(&self, request: &QueryRequest) -> QuerierResponse;
}
