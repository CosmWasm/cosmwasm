use std::fmt::Debug;

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
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
    #[snafu(display("Iterator with ID {} does not exist", id))]
    IteratorDoesNotExist {
        id: u32,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Hash doesn't match stored data"))]
    IntegrityErr { backtrace: snafu::Backtrace },
    #[snafu(display("Parse error: {}", source))]
    ParseErr {
        kind: &'static str,
        source: serde_json::Error,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Serialize error: {}", source))]
    SerializeErr {
        kind: &'static str,
        source: serde_json::Error,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Resolving wasm function: {}", source))]
    ResolveErr {
        source: wasmer_runtime_core::error::ResolveError,
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
    #[snafu(display("Runtime error: {}", msg))]
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
    #[snafu(display("Wasmer error: {}", source))]
    WasmerErr {
        source: wasmer_runtime_core::error::Error,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Calling wasm function: {}", source))]
    WasmerRuntimeErr {
        source: wasmer_runtime_core::error::RuntimeError,
        backtrace: snafu::Backtrace,
    },
}

impl From<wasmer_runtime_core::cache::Error> for VmError {
    fn from(original: wasmer_runtime_core::cache::Error) -> Self {
        make_cache_err(format!("Wasmer cache error: {:?}", original))
    }
}

impl From<wasmer_runtime_core::error::CompileError> for VmError {
    fn from(original: wasmer_runtime_core::error::CompileError) -> Self {
        make_compile_err(format!("Compile error: {:?}", original))
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

#[cfg(feature = "iterator")]
pub fn make_iterator_does_not_exist(iterator_id: u32) -> VmError {
    IteratorDoesNotExist { id: iterator_id }.build()
}

pub fn make_integrity_err() -> VmError {
    IntegrityErr {}.build()
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
    #[cfg(feature = "iterator")]
    fn make_iterator_does_not_exist_works() {
        let err = make_iterator_does_not_exist(15);
        match err {
            VmError::IteratorDoesNotExist { id, .. } => assert_eq!(id, 15),
            _ => panic!("Unexpected error"),
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
    fn make_uninitialized_context_data_works() {
        let err = make_uninitialized_context_data("foo");
        match err {
            VmError::UninitializedContextData { kind, .. } => assert_eq!(kind, "foo"),
            _ => panic!("Unexpected error"),
        }
    }
}
