#![allow(deprecated)]

mod bucket;
mod namespace_helpers;
mod prefixed_storage;
mod sequence;
mod singleton;
mod type_helpers;

pub use bucket::{bucket, bucket_read, Bucket, ReadonlyBucket};
pub use prefixed_storage::{prefixed, prefixed_read, PrefixedStorage, ReadonlyPrefixedStorage};
pub use sequence::{currval, nextval, sequence};
pub use singleton::{singleton, singleton_read, ReadonlySingleton, Singleton};

// Re-exported for backwads compatibility.
// See https://github.com/CosmWasm/cosmwasm/pull/1676.
pub use cosmwasm_std::storage_keys::{to_length_prefixed, to_length_prefixed_nested};
