use std::vec::Vec;

use crate::errors::Result;

// Storage is access to the contracts persistent data store
pub trait Storage: Clone {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
    fn set(&mut self, key: &[u8], value: &[u8]);
}

// Precompiles are callbacks to system functions defined outside of the wasm modules.
// This is a trait to allow Mocks in the test code.
//
// Currently it just supports address conversion, we could add eg. crypto functions here.
// These should all be pure (stateless) functions. If you need state, you probably want
// to use the Querier (TODO)
//
// We should consider if there is a way for modules to opt-in to only a subset of these
// Precompiles for backwards compatibility in systems that don't have them all.
pub trait Precompiles: Copy + Clone {
    fn canonical_address(&self, human: &str) -> Result<Vec<u8>>;
    fn human_address(&self, canonical: &[u8]) -> Result<String>;
}
