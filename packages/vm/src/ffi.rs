use snafu::Snafu;
use std::fmt::Debug;
use std::string::FromUtf8Error;

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
}

#[derive(Debug, Snafu)]
pub enum FfiError {
    #[snafu(display("Panic in FFI call"))]
    ForeignPanic { backtrace: snafu::Backtrace },
    #[snafu(display("bad argument passed to FFI"))]
    BadArgument { backtrace: snafu::Backtrace },
    #[snafu(display("VM received invalid UTF-8 data from backend"))]
    InvalidUtf8 { backtrace: snafu::Backtrace },
    #[snafu(display("Ran out of gas during FFI call"))]
    OutOfGas {},
    #[snafu(display("Unknown error during FFI call: {:?}", msg))]
    Unknown {
        msg: Option<String>,
        backtrace: snafu::Backtrace,
    },
    // This is the only error case of FfiError that is reported back to the contract.
    #[snafu(display("User error during FFI call: {}", msg))]
    UserErr {
        msg: String,
        backtrace: snafu::Backtrace,
    },
}

impl FfiError {
    pub fn foreign_panic() -> Self {
        ForeignPanic {}.build()
    }

    pub fn bad_argument() -> Self {
        BadArgument {}.build()
    }

    pub fn out_of_gas() -> Self {
        OutOfGas {}.build()
    }

    pub fn unknown<S: ToString>(msg: S) -> Self {
        Unknown {
            msg: Some(msg.to_string()),
        }
        .build()
    }

    /// Use `::unknown(msg: S)` if possible
    pub fn unknown_without_message() -> Self {
        Unknown { msg: None }.build()
    }

    pub fn user_err<S: ToString>(msg: S) -> Self {
        UserErr {
            msg: msg.to_string(),
        }
        .build()
    }
}

impl From<FromUtf8Error> for FfiError {
    fn from(_original: FromUtf8Error) -> Self {
        InvalidUtf8 {}.build()
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
