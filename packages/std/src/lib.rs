// Exposed on all platforms

mod encoding;
mod errors;
mod mock;
mod serde;
mod storage;
mod traits;
mod types;

pub use crate::encoding::Binary;
pub use crate::errors::{
    contract_err, dyn_contract_err, invalid, unauthorized, Error, NotFound, NullPointer, ParseErr,
    Result, SerializeErr,
};
pub use crate::mock::{dependencies, mock_env, MockApi, MockStorage};
pub use crate::serde::{from_slice, to_vec};
pub use crate::storage::{transactional, transactional_deps};
pub use crate::traits::{Api, Extern, ReadonlyStorage, Storage};
pub use crate::types::{
    coin, log, CanonicalAddr, ContractResult, CosmosMsg, Env, HumanAddr, QueryResult, Response,
};

// Exposed in wasm build only

#[cfg(target_arch = "wasm32")]
mod exports;
#[cfg(target_arch = "wasm32")]
mod imports;
#[cfg(target_arch = "wasm32")]
mod memory; // used by exports and imports only

#[cfg(target_arch = "wasm32")]
pub use crate::exports::{allocate, deallocate, do_handle, do_init, do_query};
#[cfg(target_arch = "wasm32")]
pub use crate::imports::{ExternalApi, ExternalStorage};

// Not exposed

mod demo;
