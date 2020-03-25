#[cfg(feature = "iterator")]
use crate::errors::{contract_err, Error};
#[cfg(feature = "iterator")]
use std::convert::TryFrom;

use crate::errors::Result;
use crate::types::{CanonicalAddr, HumanAddr};

#[cfg(feature = "iterator")]
pub type Pair = (Vec<u8>, Vec<u8>);

#[cfg(feature = "iterator")]
#[derive(Copy, Clone)]
// We assign these to integers to provide a stable API for passing over FFI (to wasm and Go)
pub enum Order {
    Ascending = 1,
    Descending = 2,
}

#[cfg(feature = "iterator")]
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

#[cfg(feature = "iterator")]
impl Into<i32> for Order {
    fn into(self) -> i32 {
        self as i32
    }
}

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
    fn range(
        &self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = Pair>>;
}

// Storage extends ReadonlyStorage to give mutable access
pub trait Storage: ReadonlyStorage {
    fn set(&mut self, key: &[u8], value: &[u8]);
    fn remove(&mut self, key: &[u8]);
}

// Api are callbacks to system functions defined outside of the wasm modules.
// This is a trait to allow Mocks in the test code.
//
// Currently it just supports address conversion, we could add eg. crypto functions here.
// These should all be pure (stateless) functions. If you need state, you probably want
// to use the Querier (TODO)
//
// We should consider if there is a way for modules to opt-in to only a subset of these
// Api for backwards compatibility in systems that don't have them all.
pub trait Api: Copy + Clone + Send {
    fn canonical_address(&self, human: &HumanAddr) -> Result<CanonicalAddr>;
    fn human_address(&self, canonical: &CanonicalAddr) -> Result<HumanAddr>;
}
