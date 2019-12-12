use std::vec::Vec;

use crate::errors::Result;

// Storage is access to the contracts persistent data store
pub trait Storage {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
    fn set(&mut self, key: &[u8], value: &[u8]);
}

// Addresser provides platform-specific callbacks for converting addresses
pub trait Addresser {
    fn canonicalize(&self, human: &str) -> Result<Vec<u8>>;
    fn humanize(&self, canonical: &[u8]) -> Result<String>;
}
