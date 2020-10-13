use std::fmt::Debug;
use std::ops::AddAssign;
use std::string::FromUtf8Error;
use thiserror::Error;

/// A result type for calling into the backend via FFI. Such a call causes
/// non-negligible computational cost and must always have gas information
/// attached. In order to prevent new calls from forgetting such gas information
/// to be passed, the inner success and failure types contain gas information.
pub type FfiResult<T> = (core::result::Result<T, FfiError>, GasInfo);

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

#[derive(Error, Debug)]
pub enum FfiError {
    #[error("Panic in FFI call")]
    ForeignPanic {},
    #[error("bad argument passed to FFI")]
    BadArgument {},
    #[error("VM received invalid UTF-8 data from backend")]
    InvalidUtf8 {},
    #[error("Iterator with ID {id} does not exist")]
    IteratorDoesNotExist { id: u32 },
    #[error("Ran out of gas during FFI call")]
    OutOfGas {},
    #[error("Unknown error during FFI call: {msg:?}")]
    Unknown { msg: Option<String> },
    // This is the only error case of FfiError that is reported back to the contract.
    #[error("User error during FFI call: {msg}")]
    UserErr { msg: String },
}

impl FfiError {
    pub fn foreign_panic() -> Self {
        FfiError::ForeignPanic {}
    }

    pub fn bad_argument() -> Self {
        FfiError::BadArgument {}
    }

    pub fn iterator_does_not_exist(iterator_id: u32) -> Self {
        FfiError::IteratorDoesNotExist { id: iterator_id }
    }

    pub fn out_of_gas() -> Self {
        FfiError::OutOfGas {}
    }

    pub fn unknown<S: ToString>(msg: S) -> Self {
        FfiError::Unknown {
            msg: Some(msg.to_string()),
        }
    }

    /// Use `::unknown(msg: S)` if possible
    pub fn unknown_without_message() -> Self {
        FfiError::Unknown { msg: None }
    }

    pub fn user_err<S: ToString>(msg: S) -> Self {
        FfiError::UserErr {
            msg: msg.to_string(),
        }
    }
}

impl From<FromUtf8Error> for FfiError {
    fn from(_original: FromUtf8Error) -> Self {
        FfiError::InvalidUtf8 {}
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
        let error = FfiError::foreign_panic();
        match error {
            FfiError::ForeignPanic { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn ffi_error_bad_argument() {
        let error = FfiError::bad_argument();
        match error {
            FfiError::BadArgument { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn iterator_does_not_exist_works() {
        let error = FfiError::iterator_does_not_exist(15);
        match error {
            FfiError::IteratorDoesNotExist { id, .. } => assert_eq!(id, 15),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn ffi_error_out_of_gas() {
        let error = FfiError::out_of_gas();
        match error {
            FfiError::OutOfGas { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn ffi_error_unknown() {
        let error = FfiError::unknown("broken");
        match error {
            FfiError::Unknown { msg, .. } => assert_eq!(msg.unwrap(), "broken"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn ffi_error_unknown_without_message() {
        let error = FfiError::unknown_without_message();
        match error {
            FfiError::Unknown { msg, .. } => assert!(msg.is_none()),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn ffi_error_user_err() {
        let error = FfiError::user_err("invalid input");
        match error {
            FfiError::UserErr { msg, .. } => assert_eq!(msg, "invalid input"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    // conversions

    #[test]
    fn convert_from_fromutf8error() {
        let error: FfiError = String::from_utf8(vec![0x80]).unwrap_err().into();
        match error {
            FfiError::InvalidUtf8 { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }
}
