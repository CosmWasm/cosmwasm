use snafu::Snafu;
use std::fmt::Debug;

#[derive(Debug, Snafu)]
pub enum FfiError {
    #[snafu(display("Panic in FFI call"))]
    ForeignPanic { backtrace: snafu::Backtrace },
    #[snafu(display("bad argument passed to FFI"))]
    BadArgument { backtrace: snafu::Backtrace },
    #[snafu(display("Ran out of gas during FFI call"))]
    OutOfGas {},
    #[snafu(display("Error during FFI call: {}", error))]
    Other {
        error: String,
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

    pub fn other<S>(error: S) -> Self
    where
        S: Into<String>,
    {
        Other {
            error: error.into(),
        }
        .build()
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
    fn ffi_error_other() {
        let error = FfiError::other("broken");
        match error {
            FfiError::Other { error, .. } => assert_eq!(error, "broken"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }
}
