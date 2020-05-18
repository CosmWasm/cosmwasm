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
}

impl From<wasmer_runtime_core::cache::Error> for VmError {
    fn from(original: wasmer_runtime_core::cache::Error) -> Self {
        make_cache_err(format!("Wasmer cache error: {:?}", original))
    }
}

impl From<wasmer_runtime_core::error::CompileError> for VmError {
    fn from(original: wasmer_runtime_core::error::CompileError) -> Self {
        make_compile_err(format!("Wasmer compile error: {:?}", original))
    }
}

impl From<wasmer_runtime_core::error::ResolveError> for VmError {
    fn from(original: wasmer_runtime_core::error::ResolveError) -> Self {
        make_resolve_err(format!("Wasmer resolve error: {:?}", original))
    }
}

impl From<wasmer_runtime_core::error::RuntimeError> for VmError {
    fn from(original: wasmer_runtime_core::error::RuntimeError) -> Self {
        make_runtime_err(format!("Wasmer runtime error: {:?}", original))
    }
}

pub type VmResult<T> = core::result::Result<T, VmError>;

pub fn make_cache_err<S: Into<String>>(msg: S) -> VmError {
    CacheErr { msg: msg.into() }.build()
}

pub fn make_compile_err<S: Into<String>>(msg: S) -> VmError {
    CompileErr { msg: msg.into() }.build()
}

pub fn make_conversion_err<S: Into<String>, T: Into<String>, U: Into<String>>(
    from_type: S,
    to_type: T,
    input: U,
) -> VmError {
    ConversionErr {
        from_type: from_type.into(),
        to_type: to_type.into(),
        input: input.into(),
    }
    .build()
}

pub fn make_generic_err<S: Into<String>>(msg: S) -> VmError {
    GenericErr { msg: msg.into() }.build()
}

pub fn make_instantiation_err<S: Into<String>>(msg: S) -> VmError {
    InstantiationErr { msg: msg.into() }.build()
}

pub fn make_integrity_err() -> VmError {
    IntegrityErr {}.build()
}

#[cfg(feature = "iterator")]
pub fn make_iterator_does_not_exist(iterator_id: u32) -> VmError {
    IteratorDoesNotExist { id: iterator_id }.build()
}

pub fn make_parse_err<T: Into<String>, M: Display>(target: T, msg: M) -> VmError {
    ParseErr {
        target: target.into(),
        msg: msg.to_string(),
    }
    .build()
}

pub fn make_serialize_err<S: Into<String>, M: Display>(source: S, msg: M) -> VmError {
    SerializeErr {
        source: source.into(),
        msg: msg.to_string(),
    }
    .build()
}

pub fn make_resolve_err<S: Into<String>>(msg: S) -> VmError {
    ResolveErr { msg: msg.into() }.build()
}

pub fn make_region_length_too_big(length: usize, max_length: usize) -> VmError {
    RegionLengthTooBig { length, max_length }.build()
}

pub fn make_region_too_small(size: usize, required: usize) -> VmError {
    RegionTooSmall { size, required }.build()
}

pub fn make_runtime_err<S: Into<String>>(msg: S) -> VmError {
    RuntimeErr { msg: msg.into() }.build()
}

pub fn make_static_validation_err<S: Into<String>>(msg: S) -> VmError {
    StaticValidationErr { msg: msg.into() }.build()
}

pub fn make_uninitialized_context_data<S: Into<String>>(kind: S) -> VmError {
    UninitializedContextData { kind: kind.into() }.build()
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

pub fn make_ffi_foreign_panic() -> FfiError {
    ForeignPanic {}.build()
}

pub fn make_ffi_bad_argument() -> FfiError {
    BadArgument {}.build()
}

pub fn make_ffi_out_of_gas() -> FfiError {
    FfiError::OutOfGas {}
}

pub fn make_ffi_other<S>(error: S) -> FfiError
where
    S: Into<String>,
{
    Other {
        error: error.into(),
    }
    .build()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn make_cache_err_works() {
        let err = make_cache_err("something went wrong");
        match err {
            VmError::CacheErr { msg, .. } => assert_eq!(msg, "something went wrong"),
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn make_compile_err_works() {
        let err = make_compile_err("something went wrong");
        match err {
            VmError::CompileErr { msg, .. } => assert_eq!(msg, "something went wrong"),
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn make_conversion_err_works() {
        let err = make_conversion_err("i32", "u32", "-9");
        match err {
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
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn make_generic_err_works() {
        let guess = 7;
        let error = make_generic_err(format!("{} is too low", guess));
        match error {
            VmError::GenericErr { msg, .. } => {
                assert_eq!(msg, String::from("7 is too low"));
            }
            e => panic!("unexpected error, {:?}", e),
        }
    }

    #[test]
    fn make_instantiation_err_works() {
        let err = make_instantiation_err("something went wrong");
        match err {
            VmError::InstantiationErr { msg, .. } => assert_eq!(msg, "something went wrong"),
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn make_integrity_err_works() {
        let err = make_integrity_err();
        match err {
            VmError::IntegrityErr { .. } => {}
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn make_iterator_does_not_exist_works() {
        let err = make_iterator_does_not_exist(15);
        match err {
            VmError::IteratorDoesNotExist { id, .. } => assert_eq!(id, 15),
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn make_parse_err_works() {
        let error = make_parse_err("Book", "Missing field: title");
        match error {
            VmError::ParseErr { target, msg, .. } => {
                assert_eq!(target, "Book");
                assert_eq!(msg, "Missing field: title");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn make_serialize_err_works() {
        let error = make_serialize_err("Book", "Content too long");
        match error {
            VmError::SerializeErr { source, msg, .. } => {
                assert_eq!(source, "Book");
                assert_eq!(msg, "Content too long");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn make_resolve_err_works() {
        let error = make_resolve_err("function has different signature");
        match error {
            VmError::ResolveErr { msg, .. } => assert_eq!(msg, "function has different signature"),
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn make_region_length_too_big_works() {
        let err = make_region_length_too_big(50, 20);
        match err {
            VmError::RegionLengthTooBig {
                length, max_length, ..
            } => {
                assert_eq!(length, 50);
                assert_eq!(max_length, 20);
            }
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn make_region_too_small_works() {
        let err = make_region_too_small(12, 33);
        match err {
            VmError::RegionTooSmall { size, required, .. } => {
                assert_eq!(size, 12);
                assert_eq!(required, 33);
            }
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn make_runtime_err_works() {
        let err = make_runtime_err("something went wrong");
        match err {
            VmError::RuntimeErr { msg, .. } => assert_eq!(msg, "something went wrong"),
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn make_static_validation_err_works() {
        let error = make_static_validation_err("export xy missing");
        match error {
            VmError::StaticValidationErr { msg, .. } => assert_eq!(msg, "export xy missing"),
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn make_uninitialized_context_data_works() {
        let err = make_uninitialized_context_data("foo");
        match err {
            VmError::UninitializedContextData { kind, .. } => assert_eq!(kind, "foo"),
            _ => panic!("Unexpected error"),
        }
    }
}
