use std::fmt::{Debug, Display};

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum VmError {
    #[snafu(display("Cache error: {}", msg))]
    CacheErr {
        msg: String,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Error in guest/host communication: {}", source))]
    CommunicationErr {
        #[snafu(backtrace)]
        source: CommunicationError,
    },
    #[snafu(display("Error compiling Wasm: {}", msg))]
    CompileErr {
        msg: String,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Couldn't convert from {} to {}. Input: {}", from_type, to_type, input))]
    ConversionErr {
        from_type: String,
        to_type: String,
        input: String,
        backtrace: snafu::Backtrace,
    },
    /// Whenever there is no specific error type available
    #[snafu(display("Generic error: {}", msg))]
    GenericErr {
        msg: String,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Error instantiating a Wasm module: {}", msg))]
    InstantiationErr {
        msg: String,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Hash doesn't match stored data"))]
    IntegrityErr { backtrace: snafu::Backtrace },
    #[snafu(display("Iterator with ID {} does not exist", id))]
    IteratorDoesNotExist {
        id: u32,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Error parsing into type {}: {}", target, msg))]
    ParseErr {
        /// the target type that was attempted
        target: String,
        msg: String,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Error serializing type {}: {}", source, msg))]
    SerializeErr {
        /// the source type that was attempted
        #[snafu(source(false))]
        source: String,
        msg: String,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Error resolving Wasm function: {}", msg))]
    ResolveErr {
        msg: String,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Region length too big. Got {}, limit {}", length, max_length))]
    // Note: this only checks length, not capacity
    RegionLengthTooBig {
        length: usize,
        max_length: usize,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Region too small. Got {}, required {}", size, required))]
    RegionTooSmall {
        size: usize,
        required: usize,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Error executing Wasm: {}", msg))]
    RuntimeErr {
        msg: String,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Error during static Wasm validation: {}", msg))]
    StaticValidationErr {
        msg: String,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Uninitialized Context Data: {}", kind))]
    UninitializedContextData {
        kind: String,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Calling external function through FFI: {}", source))]
    FfiErr {
        #[snafu(backtrace)]
        source: FfiError,
    },
    #[snafu(display("Ran out of gas during contract execution"))]
    GasDepletion,
    #[snafu(display("Must not call a writing storage function in this context."))]
    WriteAccessDenied { backtrace: snafu::Backtrace },
}

impl VmError {
    pub(crate) fn cache_err<S: Into<String>>(msg: S) -> Self {
        CacheErr { msg: msg.into() }.build()
    }

    pub(crate) fn compile_err<S: Into<String>>(msg: S) -> Self {
        CompileErr { msg: msg.into() }.build()
    }

    pub(crate) fn conversion_err<S: Into<String>, T: Into<String>, U: Into<String>>(
        from_type: S,
        to_type: T,
        input: U,
    ) -> Self {
        ConversionErr {
            from_type: from_type.into(),
            to_type: to_type.into(),
            input: input.into(),
        }
        .build()
    }

    pub(crate) fn generic_err<S: Into<String>>(msg: S) -> Self {
        GenericErr { msg: msg.into() }.build()
    }

    pub(crate) fn instantiation_err<S: Into<String>>(msg: S) -> Self {
        InstantiationErr { msg: msg.into() }.build()
    }

    pub(crate) fn integrity_err() -> Self {
        IntegrityErr {}.build()
    }

    #[cfg(feature = "iterator")]
    pub(crate) fn iterator_does_not_exist(iterator_id: u32) -> Self {
        IteratorDoesNotExist { id: iterator_id }.build()
    }

    pub(crate) fn parse_err<T: Into<String>, M: Display>(target: T, msg: M) -> Self {
        ParseErr {
            target: target.into(),
            msg: msg.to_string(),
        }
        .build()
    }

    pub(crate) fn serialize_err<S: Into<String>, M: Display>(source: S, msg: M) -> Self {
        SerializeErr {
            source: source.into(),
            msg: msg.to_string(),
        }
        .build()
    }

    pub(crate) fn resolve_err<S: Into<String>>(msg: S) -> Self {
        ResolveErr { msg: msg.into() }.build()
    }

    pub(crate) fn region_length_too_big(length: usize, max_length: usize) -> Self {
        RegionLengthTooBig { length, max_length }.build()
    }

    pub(crate) fn region_too_small(size: usize, required: usize) -> Self {
        RegionTooSmall { size, required }.build()
    }

    pub(crate) fn runtime_err<S: Into<String>>(msg: S) -> Self {
        RuntimeErr { msg: msg.into() }.build()
    }

    pub(crate) fn static_validation_err<S: Into<String>>(msg: S) -> Self {
        StaticValidationErr { msg: msg.into() }.build()
    }

    pub(crate) fn uninitialized_context_data<S: Into<String>>(kind: S) -> Self {
        UninitializedContextData { kind: kind.into() }.build()
    }

    pub(crate) fn write_access_denied() -> Self {
        WriteAccessDenied {}.build()
    }
}

impl From<wasmer_runtime_core::cache::Error> for VmError {
    fn from(original: wasmer_runtime_core::cache::Error) -> Self {
        VmError::cache_err(format!("Wasmer cache error: {:?}", original))
    }
}

impl From<wasmer_runtime_core::error::CompileError> for VmError {
    fn from(original: wasmer_runtime_core::error::CompileError) -> Self {
        VmError::compile_err(format!("Wasmer compile error: {:?}", original))
    }
}

impl From<wasmer_runtime_core::error::ResolveError> for VmError {
    fn from(original: wasmer_runtime_core::error::ResolveError) -> Self {
        VmError::resolve_err(format!("Wasmer resolve error: {:?}", original))
    }
}

impl From<wasmer_runtime_core::error::RuntimeError> for VmError {
    fn from(original: wasmer_runtime_core::error::RuntimeError) -> Self {
        use wasmer_runtime_core::error::{InvokeError, RuntimeError};

        fn runtime_error(err: RuntimeError) -> VmError {
            VmError::runtime_err(format!("Wasmer runtime error: {:?}", err))
        }

        match original {
            // TODO: fix the issue described below:
            // `InvokeError::FailedWithNoError` happens when running out of gas in singlepass v0.17
            // but it's supposed to indicate bugs in Wasmer...
            // https://github.com/wasmerio/wasmer/issues/1452
            // https://github.com/CosmWasm/cosmwasm/issues/375
            RuntimeError::InvokeError(InvokeError::FailedWithNoError) => VmError::GasDepletion,
            // This variant contains the error we return from imports.
            RuntimeError::User(err) => match err.downcast::<VmError>() {
                Ok(err) => *err,
                Err(err) => runtime_error(RuntimeError::User(err)),
            },
            _ => runtime_error(original),
        }
    }
}

pub type VmResult<T> = core::result::Result<T, VmError>;

/// An error in the communcation between contract and host. Those happen around imports and exports.
#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum CommunicationError {
    #[snafu(display("Got a zero Wasm address"))]
    ZeroAddress { backtrace: snafu::Backtrace },
    #[snafu(display(
        "A Wasm memory address provided by the contract could not be dereferenced: {}",
        msg
    ))]
    DerefErr {
        msg: String,
        backtrace: snafu::Backtrace,
    },
}

impl CommunicationError {
    pub fn zero_address() -> Self {
        ZeroAddress {}.build()
    }

    pub fn deref_err<S: Into<String>>(msg: S) -> Self {
        DerefErr { msg: msg.into() }.build()
    }
}

pub type CommunicationResult<T> = core::result::Result<T, CommunicationError>;

impl From<CommunicationError> for VmError {
    fn from(communication_error: CommunicationError) -> Self {
        VmError::CommunicationErr {
            source: communication_error,
        }
    }
}

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

    pub fn set_message<S>(&mut self, message: S) -> &mut Self
    where
        S: Into<String>,
    {
        if let FfiError::Other { error, .. } = self {
            *error = message.into()
        }
        self
    }
}

impl From<FfiError> for VmError {
    fn from(ffi_error: FfiError) -> Self {
        match ffi_error {
            FfiError::OutOfGas {} => VmError::GasDepletion,
            _ => VmError::FfiErr { source: ffi_error },
        }
    }
}

pub type FfiResult<T> = core::result::Result<T, FfiError>;

#[cfg(test)]
mod test {
    use super::*;

    // VmError constructors

    #[test]
    fn vm_error_cache_err_works() {
        let error = VmError::cache_err("something went wrong");
        match error {
            VmError::CacheErr { msg, .. } => assert_eq!(msg, "something went wrong"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn vm_error_compile_err_works() {
        let error = VmError::compile_err("something went wrong");
        match error {
            VmError::CompileErr { msg, .. } => assert_eq!(msg, "something went wrong"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn vm_error_conversion_err_works() {
        let error = VmError::conversion_err("i32", "u32", "-9");
        match error {
            VmError::ConversionErr {
                from_type,
                to_type,
                input,
                ..
            } => {
                assert_eq!(from_type, "i32");
                assert_eq!(to_type, "u32");
                assert_eq!(input, "-9");
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn vm_error_generic_err_works() {
        let guess = 7;
        let error = VmError::generic_err(format!("{} is too low", guess));
        match error {
            VmError::GenericErr { msg, .. } => {
                assert_eq!(msg, String::from("7 is too low"));
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn vm_error_instantiation_err_works() {
        let error = VmError::instantiation_err("something went wrong");
        match error {
            VmError::InstantiationErr { msg, .. } => assert_eq!(msg, "something went wrong"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn vm_error_integrity_err_works() {
        let error = VmError::integrity_err();
        match error {
            VmError::IntegrityErr { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn vm_error_iterator_does_not_exist_works() {
        let error = VmError::iterator_does_not_exist(15);
        match error {
            VmError::IteratorDoesNotExist { id, .. } => assert_eq!(id, 15),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn vm_error_parse_err_works() {
        let error = VmError::parse_err("Book", "Missing field: title");
        match error {
            VmError::ParseErr { target, msg, .. } => {
                assert_eq!(target, "Book");
                assert_eq!(msg, "Missing field: title");
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn vm_error_serialize_err_works() {
        let error = VmError::serialize_err("Book", "Content too long");
        match error {
            VmError::SerializeErr { source, msg, .. } => {
                assert_eq!(source, "Book");
                assert_eq!(msg, "Content too long");
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn vm_error_resolve_err_works() {
        let error = VmError::resolve_err("function has different signature");
        match error {
            VmError::ResolveErr { msg, .. } => assert_eq!(msg, "function has different signature"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn vm_error_region_length_too_big_works() {
        let error = VmError::region_length_too_big(50, 20);
        match error {
            VmError::RegionLengthTooBig {
                length, max_length, ..
            } => {
                assert_eq!(length, 50);
                assert_eq!(max_length, 20);
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn vm_error_region_too_small_works() {
        let error = VmError::region_too_small(12, 33);
        match error {
            VmError::RegionTooSmall { size, required, .. } => {
                assert_eq!(size, 12);
                assert_eq!(required, 33);
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn vm_error_runtime_err_works() {
        let error = VmError::runtime_err("something went wrong");
        match error {
            VmError::RuntimeErr { msg, .. } => assert_eq!(msg, "something went wrong"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn vm_error_static_validation_err_works() {
        let error = VmError::static_validation_err("export xy missing");
        match error {
            VmError::StaticValidationErr { msg, .. } => assert_eq!(msg, "export xy missing"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn vm_error_uninitialized_context_data_works() {
        let error = VmError::uninitialized_context_data("foo");
        match error {
            VmError::UninitializedContextData { kind, .. } => assert_eq!(kind, "foo"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn vm_error_write_access_denied() {
        let error = VmError::write_access_denied();
        match error {
            VmError::WriteAccessDenied { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    // CommunicationError constructors

    #[test]
    fn communication_error_zero_address() {
        let error = CommunicationError::zero_address();
        match error {
            CommunicationError::ZeroAddress { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn communication_error_deref_err() {
        let error = CommunicationError::deref_err("broken stuff");
        match error {
            CommunicationError::DerefErr { msg, .. } => assert_eq!(msg, "broken stuff"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    // FfiError constructors

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
