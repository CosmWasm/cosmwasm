#![cfg_attr(feature = "backtraces", feature(backtrace))]

mod backend;
mod cache;
mod calls;
mod checksum;
mod compatibility;
mod conversion;
mod environment;
mod errors;
mod features;
mod imports;
mod instance;
mod limited;
mod memory;
mod middleware;
mod modules;
mod serde;
mod signatures;
mod size;
pub mod testing;
mod wasm_backend;

pub use crate::backend::{Api, Backend, BackendError, BackendResult, GasInfo, Querier, Storage};
pub use crate::cache::{Cache, CacheOptions, Stats};
pub use crate::calls::{
    call_handle, call_handle_raw, call_init, call_init_raw, call_migrate, call_migrate_raw,
    call_query, call_query_raw,
};
pub use crate::checksum::Checksum;
pub use crate::errors::{
    CommunicationError, CommunicationResult, RegionValidationError, RegionValidationResult,
    VmError, VmResult,
};
pub use crate::features::features_from_csv;
pub use crate::instance::{GasReport, Instance, InstanceOptions};
pub use crate::serde::{from_slice, to_vec};
pub use crate::size::Size;
