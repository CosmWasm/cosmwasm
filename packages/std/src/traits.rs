use crate::encoding::Binary;
use crate::errors::{ApiError, Result};
use crate::query::QueryRequest;
use crate::types::{CanonicalAddr, HumanAddr};

#[cfg(feature = "iterator")]
pub use iter_support::{KVRef, Order, KV};

/// Holds all external dependencies of the contract.
/// Designed to allow easy dependency injection at runtime.
/// This cannot be copied or cloned since it would behave differently
/// for mock storages and a bridge storage in the VM.
pub struct Extern<S: Storage, A: Api> {
    pub storage: S,
    pub api: A,
}

/// ReadonlyStorage is access to the contracts persistent data store
pub trait ReadonlyStorage {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
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
    fn set(&mut self, key: &[u8], value: &[u8]);
    fn remove(&mut self, key: &[u8]);
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

pub trait Querier {
    // Note: ApiError type can be serialized, and the below can be reconstituted over a WASM/FFI call.
    // Since this is information that is returned from outside, we define it this way.
    fn query(&self, request: QueryRequest) -> Result<Binary, ApiError>;
}

// put them here to avoid so many feature flags
#[cfg(feature = "iterator")]
mod iter_support {
    use crate::errors::{contract_err, Error};
    use std::convert::TryFrom;

    /// KV is a Key-Value pair, returned from our iterators
    pub type KV<T = Vec<u8>> = (Vec<u8>, T);

    /// KVRef is a Key-Value pair reference, returned from underlying btree iterators
    pub type KVRef<'a, T = Vec<u8>> = (&'a Vec<u8>, &'a T);

    #[derive(Copy, Clone)]
    // We assign these to integers to provide a stable API for passing over FFI (to wasm and Go)
    pub enum Order {
        Ascending = 1,
        Descending = 2,
    }

    impl TryFrom<i32> for Order {
        type Error = Error;

        fn try_from(value: i32) -> Result<Self, Self::Error> {
            match value {
                1 => Ok(Order::Ascending),
                2 => Ok(Order::Descending),
                _ => contract_err("Order must be 1 or 2"),
            }
        }
    }

    impl Into<i32> for Order {
        fn into(self) -> i32 {
            self as i32
        }
    }
}
