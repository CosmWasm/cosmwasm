use std::vec::Vec;

use crate::errors::Result;
use crate::types::{CanonicalAddr, HumanAddr};

/// Holds all external dependencies of the contract.
/// Designed to allow easy dependency injection at runtime.
/// This cannot be copied or cloned since it would behave differently
/// for mock storages and a bridge storage in the VM.
pub struct Extern<S: Storage, A: Api> {
    pub storage: S,
    pub api: A,
}

// ReadonlyStorage is access to the contracts persistent data store
pub trait ReadonlyStorage {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
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
