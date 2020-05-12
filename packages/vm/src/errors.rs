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
    #[snafu(display("Error executing Wasm: {}", msg))]
    WasmerRuntimeErr {
        msg: String,
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

impl From<wasmer_runtime_core::error::ResolveError> for VmError {
    fn from(original: wasmer_runtime_core::error::ResolveError) -> Self {
        make_resolve_err(format!("Resolve error: {:?}", original))
    }
}

impl From<wasmer_runtime_core::error::RuntimeError> for VmError {
    fn from(original: wasmer_runtime_core::error::RuntimeError) -> Self {
        make_wasmer_runtime_err(format!("Wasmer runtime error: {:?}", original))
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

pub fn make_wasmer_runtime_err<S: Into<String>>(msg: S) -> VmError {
    WasmerRuntimeErr { msg: msg.into() }.build()
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
    fn make_uninitialized_context_data_works() {
        let err = make_uninitialized_context_data("foo");
        match err {
            VmError::UninitializedContextData { kind, .. } => assert_eq!(kind, "foo"),
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn make_wasmer_runtime_err_works() {
        let err = make_wasmer_runtime_err("something went wrong");
        match err {
            VmError::WasmerRuntimeErr { msg, .. } => assert_eq!(msg, "something went wrong"),
            _ => panic!("Unexpected error"),
        }
    }
}
