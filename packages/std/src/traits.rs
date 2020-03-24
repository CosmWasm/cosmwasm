#[cfg(feature = "iterator")]
use std::ops::RangeBounds;

use crate::errors::Result;
use crate::types::{CanonicalAddr, HumanAddr};

#[cfg(feature = "iterator")]
pub type KVPair = (Vec<u8>, Vec<u8>);

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
    /// uses standard rust range notation eg db.range(b"bar"..b"foo")
    /// returns a DoubleEndedIterator, so range(..).rev() is efficient to get the end
    fn range<R: Clone + RangeBounds<Vec<u8>>>(
        &self,
        bounds: R,
        // TODO: use Asc/Desc as enum for clarity
        reverse: bool,
    ) -> Box<dyn Iterator<Item = KVPair>>;
}

// Storage extends ReadonlyStorage to give mutable access
pub trait Storage: ReadonlyStorage {
    fn set(&mut self, key: &[u8], value: &[u8]);
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
