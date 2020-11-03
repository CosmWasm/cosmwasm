use std::fmt::Debug;
use std::ops::AddAssign;
use std::string::FromUtf8Error;
use thiserror::Error;

use cosmwasm_std::{Binary, CanonicalAddr, ContractResult, HumanAddr, SystemResult};
#[cfg(feature = "iterator")]
use cosmwasm_std::{Order, KV};

#[derive(Copy, Clone, Debug)]
pub struct GasInfo {
    /// The gas cost of a computation that was executed already but not yet charged
    pub cost: u64,
    /// Gas that was used and charged externally. This is needed to
    /// adjust the VM's gas limit but does not affect the gas usage.
    pub externally_used: u64,
}

impl GasInfo {
    pub fn with_cost(amount: u64) -> Self {
        GasInfo {
            cost: amount,
            externally_used: 0,
        }
    }

    pub fn with_externally_used(amount: u64) -> Self {
        GasInfo {
            cost: 0,
            externally_used: amount,
        }
    }

    /// Creates a gas information with no cost for the caller and with zero externally used gas.
    ///
    /// Caution: when using this you need to make sure no gas was metered externally to keep the gas values in sync.
    pub fn free() -> Self {
        GasInfo {
            cost: 0,
            externally_used: 0,
        }
    }
}

impl AddAssign for GasInfo {
    fn add_assign(&mut self, other: Self) {
        *self = GasInfo {
            cost: self.cost + other.cost,
            externally_used: self.externally_used + other.cost,
        };
    }
}

/// Holds all external dependencies of the contract.
/// Designed to allow easy dependency injection at runtime.
/// This cannot be copied or cloned since it would behave differently
/// for mock storages and a bridge storage in the VM.
pub struct Backend<S: Storage, A: Api, Q: Querier> {
    pub storage: S,
    pub api: A,
    pub querier: Q,
}

/// Access to the VM's backend storage, i.e. the chain
pub trait Storage {
    /// Returns Err on error.
    /// Returns Ok(None) when key does not exist.
    /// Returns Ok(Some(Vec<u8>)) when key exists.
    ///
    /// Note: Support for differentiating between a non-existent key and a key with empty value
    /// is not great yet and might not be possible in all backends. But we're trying to get there.
    fn get(&self, key: &[u8]) -> BackendResult<Option<Vec<u8>>>;

    /// Allows iteration over a set of key/value pairs, either forwards or backwards.
    /// Returns an interator ID that is unique within the Storage instance.
    ///
    /// The bound `start` is inclusive and `end` is exclusive.
    ///
    /// If `start` is lexicographically greater than or equal to `end`, an empty range is described, mo matter of the order.
    ///
    /// This call must not change data in the storage, but creating and storing a new iterator can be a mutating operation on
    /// the Storage implementation.
    /// The implementation must ensure that iterator IDs are assigned in a deterministic manner as this is
    /// environment data that is injected into the contract.
    #[cfg(feature = "iterator")]
    fn scan(
        &mut self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> BackendResult<u32>;

    /// Returns the next element of the iterator with the given ID.
    ///
    /// If the ID is not found, a BackendError::IteratorDoesNotExist is returned.
    ///
    /// This call must not change data in the storage, but incrementing an iterator can be a mutating operation on
    /// the Storage implementation.
    #[cfg(feature = "iterator")]
    fn next(&mut self, iterator_id: u32) -> BackendResult<Option<KV>>;

    fn set(&mut self, key: &[u8], value: &[u8]) -> BackendResult<()>;

    /// Removes a database entry at `key`.
    ///
    /// The current interface does not allow to differentiate between a key that existed
    /// before and one that didn't exist. See https://github.com/CosmWasm/cosmwasm/issues/290
    fn remove(&mut self, key: &[u8]) -> BackendResult<()>;
}

/// Api are callbacks to system functions defined outside of the wasm modules.
/// This is a trait to allow Mocks in the test code.
///
/// Currently it just supports address conversion, we could add eg. crypto functions here.
/// These should all be pure (stateless) functions. If you need state, you probably want
/// to use the Querier.
///
/// We can use feature flags to opt-in to non-essential methods
/// for backwards compatibility in systems that don't have them all.
pub trait Api: Copy + Clone + Send {
    fn canonical_address(&self, human: &HumanAddr) -> BackendResult<CanonicalAddr>;
    fn human_address(&self, canonical: &CanonicalAddr) -> BackendResult<HumanAddr>;
}

pub trait Querier {
    /// This is all that must be implemented for the Querier.
    /// This allows us to pass through binary queries from one level to another without
    /// knowing the custom format, or we can decode it, with the knowledge of the allowed
    /// types.
    ///
    /// The gas limit describes how much VM gas this particular query is allowed
    /// to comsume when measured separately from the rest of the contract.
    /// The returned gas info (in BackendResult) can exceed the gas limit in cases
    /// where the query could not be aborted exactly at the limit.
    fn query_raw(
        &self,
        request: &[u8],
        gas_limit: u64,
    ) -> BackendResult<SystemResult<ContractResult<Binary>>>;
}

/// A result type for calling into the backend. Such a call can cause
/// non-negligible computational cost in both success and faiure case and must always have gas information
/// attached.
pub type BackendResult<T> = (core::result::Result<T, BackendError>, GasInfo);

#[derive(Error, Debug)]
pub enum BackendError {
    #[error("Panic in FFI call")]
    ForeignPanic {},
    #[error("Bad argument")]
    BadArgument {},
    #[error("VM received invalid UTF-8 data from backend")]
    InvalidUtf8 {},
    #[error("Iterator with ID {id} does not exist")]
    IteratorDoesNotExist { id: u32 },
    #[error("Ran out of gas during call into backend")]
    OutOfGas {},
    #[error("Unknown error during call into backend: {msg:?}")]
    Unknown { msg: Option<String> },
    // This is the only error case of BackendError that is reported back to the contract.
    #[error("User error during call into backend: {msg}")]
    UserErr { msg: String },
}

impl BackendError {
    pub fn foreign_panic() -> Self {
        BackendError::ForeignPanic {}
    }

    pub fn bad_argument() -> Self {
        BackendError::BadArgument {}
    }

    pub fn iterator_does_not_exist(iterator_id: u32) -> Self {
        BackendError::IteratorDoesNotExist { id: iterator_id }
    }

    pub fn out_of_gas() -> Self {
        BackendError::OutOfGas {}
    }

    pub fn unknown<S: ToString>(msg: S) -> Self {
        BackendError::Unknown {
            msg: Some(msg.to_string()),
        }
    }

    /// Use `::unknown(msg: S)` if possible
    pub fn unknown_without_message() -> Self {
        BackendError::Unknown { msg: None }
    }

    pub fn user_err<S: ToString>(msg: S) -> Self {
        BackendError::UserErr {
            msg: msg.to_string(),
        }
    }
}

impl From<FromUtf8Error> for BackendError {
    fn from(_original: FromUtf8Error) -> Self {
        BackendError::InvalidUtf8 {}
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn gas_info_with_cost_works() {
        let gas_info = GasInfo::with_cost(21);
        assert_eq!(gas_info.cost, 21);
        assert_eq!(gas_info.externally_used, 0);
    }

    #[test]
    fn gas_info_with_externally_used_works() {
        let gas_info = GasInfo::with_externally_used(65);
        assert_eq!(gas_info.cost, 0);
        assert_eq!(gas_info.externally_used, 65);
    }

    #[test]
    fn gas_info_free_works() {
        let gas_info = GasInfo::free();
        assert_eq!(gas_info.cost, 0);
        assert_eq!(gas_info.externally_used, 0);
    }

    // constructors

    #[test]
    fn ffi_error_foreign_panic() {
        let error = BackendError::foreign_panic();
        match error {
            BackendError::ForeignPanic { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn ffi_error_bad_argument() {
        let error = BackendError::bad_argument();
        match error {
            BackendError::BadArgument { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn iterator_does_not_exist_works() {
        let error = BackendError::iterator_does_not_exist(15);
        match error {
            BackendError::IteratorDoesNotExist { id, .. } => assert_eq!(id, 15),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn ffi_error_out_of_gas() {
        let error = BackendError::out_of_gas();
        match error {
            BackendError::OutOfGas { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn ffi_error_unknown() {
        let error = BackendError::unknown("broken");
        match error {
            BackendError::Unknown { msg, .. } => assert_eq!(msg.unwrap(), "broken"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn ffi_error_unknown_without_message() {
        let error = BackendError::unknown_without_message();
        match error {
            BackendError::Unknown { msg, .. } => assert!(msg.is_none()),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn ffi_error_user_err() {
        let error = BackendError::user_err("invalid input");
        match error {
            BackendError::UserErr { msg, .. } => assert_eq!(msg, "invalid input"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    // conversions

    #[test]
    fn convert_from_fromutf8error() {
        let error: BackendError = String::from_utf8(vec![0x80]).unwrap_err().into();
        match error {
            BackendError::InvalidUtf8 { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }
}
