use std::fmt::{Debug, Display};
use thiserror::Error;

use super::communication_error::CommunicationError;
use crate::backend::BackendError;
use crate::wasm_backend::InsufficientGasLeft;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum VmError {
    #[error("Cache error: {msg}")]
    CacheErr { msg: String },
    #[error("Error in guest/host communication: {source}")]
    CommunicationErr {
        #[from]
        source: CommunicationError,
    },
    #[error("Error compiling Wasm: {msg}")]
    CompileErr { msg: String },
    #[error("Couldn't convert from {} to {}. Input: {}", from_type, to_type, input)]
    ConversionErr {
        from_type: String,
        to_type: String,
        input: String,
    },
    /// Whenever there is no specific error type available
    #[error("Generic error: {msg}")]
    GenericErr { msg: String },
    #[error("Error instantiating a Wasm module: {msg}")]
    InstantiationErr { msg: String },
    #[error("Hash doesn't match stored data")]
    IntegrityErr {},
    #[error("Error parsing into type {target_type}: {msg}")]
    ParseErr {
        /// the target type that was attempted
        target_type: String,
        msg: String,
    },
    #[error("Error serializing type {source_type}: {msg}")]
    SerializeErr {
        /// the source type that was attempted
        source_type: String,
        msg: String,
    },
    #[error("Error resolving Wasm function: {}", msg)]
    ResolveErr { msg: String },
    #[error("Error executing Wasm: {}", msg)]
    RuntimeErr { msg: String },
    #[error("Error during static Wasm validation: {}", msg)]
    StaticValidationErr { msg: String },
    #[error("Uninitialized Context Data: {}", kind)]
    UninitializedContextData { kind: String },
    #[error("Error calling into the VM's backend: {}", source)]
    BackendErr { source: BackendError },
    #[error("Ran out of gas during contract execution")]
    GasDepletion,
    #[error("Must not call a writing storage function in this context.")]
    WriteAccessDenied {},
}

impl VmError {
    pub(crate) fn cache_err<S: Into<String>>(msg: S) -> Self {
        VmError::CacheErr { msg: msg.into() }
    }

    pub(crate) fn compile_err<S: Into<String>>(msg: S) -> Self {
        VmError::CompileErr { msg: msg.into() }
    }

    pub(crate) fn conversion_err<S: Into<String>, T: Into<String>, U: Into<String>>(
        from_type: S,
        to_type: T,
        input: U,
    ) -> Self {
        VmError::ConversionErr {
            from_type: from_type.into(),
            to_type: to_type.into(),
            input: input.into(),
        }
    }

    pub(crate) fn generic_err<S: Into<String>>(msg: S) -> Self {
        VmError::GenericErr { msg: msg.into() }
    }

    pub(crate) fn instantiation_err<S: Into<String>>(msg: S) -> Self {
        VmError::InstantiationErr { msg: msg.into() }
    }

    pub(crate) fn integrity_err() -> Self {
        VmError::IntegrityErr {}
    }

    pub(crate) fn parse_err<T: Into<String>, M: Display>(target: T, msg: M) -> Self {
        VmError::ParseErr {
            target_type: target.into(),
            msg: msg.to_string(),
        }
    }

    pub(crate) fn serialize_err<S: Into<String>, M: Display>(source: S, msg: M) -> Self {
        VmError::SerializeErr {
            source_type: source.into(),
            msg: msg.to_string(),
        }
    }

    pub(crate) fn resolve_err<S: Into<String>>(msg: S) -> Self {
        VmError::ResolveErr { msg: msg.into() }
    }

    pub(crate) fn runtime_err<S: Into<String>>(msg: S) -> Self {
        VmError::RuntimeErr { msg: msg.into() }
    }

    pub(crate) fn static_validation_err<S: Into<String>>(msg: S) -> Self {
        VmError::StaticValidationErr { msg: msg.into() }
    }

    pub(crate) fn uninitialized_context_data<S: Into<String>>(kind: S) -> Self {
        VmError::UninitializedContextData { kind: kind.into() }
    }

    pub(crate) fn write_access_denied() -> Self {
        VmError::WriteAccessDenied {}
    }
}

impl From<BackendError> for VmError {
    fn from(original: BackendError) -> Self {
        match original {
            BackendError::OutOfGas {} => VmError::GasDepletion,
            _ => VmError::BackendErr { source: original },
        }
    }
}

impl From<wasmer::ExportError> for VmError {
    fn from(original: wasmer::ExportError) -> Self {
        VmError::resolve_err(format!("Could not get export: {:?}", original))
    }
}

impl From<wasmer::SerializeError> for VmError {
    fn from(original: wasmer::SerializeError) -> Self {
        VmError::cache_err(format!("Could not serialize module: {:?}", original))
    }
}

impl From<wasmer::DeserializeError> for VmError {
    fn from(original: wasmer::DeserializeError) -> Self {
        VmError::cache_err(format!("Could not deserialize module: {:?}", original))
    }
}

impl From<wasmer::RuntimeError> for VmError {
    fn from(original: wasmer::RuntimeError) -> Self {
        VmError::runtime_err(format!("Wasmer runtime error: {:?}", original))
    }
}

impl From<wasmer_compiler::CompileError> for VmError {
    fn from(original: wasmer_compiler::CompileError) -> Self {
        VmError::compile_err(format!("Could not compile: {:?}", original))
    }
}

impl From<InsufficientGasLeft> for VmError {
    fn from(_original: InsufficientGasLeft) -> Self {
        VmError::GasDepletion
    }
}

impl From<std::convert::Infallible> for VmError {
    fn from(_original: std::convert::Infallible) -> Self {
        unreachable!();
    }
}

impl From<VmError> for wasmer::RuntimeError {
    fn from(original: VmError) -> wasmer::RuntimeError {
        let msg: String = original.to_string();
        wasmer::RuntimeError::new(msg)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // constructors

    #[test]
    fn cache_err_works() {
        let error = VmError::cache_err("something went wrong");
        match error {
            VmError::CacheErr { msg, .. } => assert_eq!(msg, "something went wrong"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn compile_err_works() {
        let error = VmError::compile_err("something went wrong");
        match error {
            VmError::CompileErr { msg, .. } => assert_eq!(msg, "something went wrong"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn conversion_err_works() {
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
    fn generic_err_works() {
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
    fn instantiation_err_works() {
        let error = VmError::instantiation_err("something went wrong");
        match error {
            VmError::InstantiationErr { msg, .. } => assert_eq!(msg, "something went wrong"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn integrity_err_works() {
        let error = VmError::integrity_err();
        match error {
            VmError::IntegrityErr { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn parse_err_works() {
        let error = VmError::parse_err("Book", "Missing field: title");
        match error {
            VmError::ParseErr {
                target_type, msg, ..
            } => {
                assert_eq!(target_type, "Book");
                assert_eq!(msg, "Missing field: title");
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn serialize_err_works() {
        let error = VmError::serialize_err("Book", "Content too long");
        match error {
            VmError::SerializeErr {
                source_type, msg, ..
            } => {
                assert_eq!(source_type, "Book");
                assert_eq!(msg, "Content too long");
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn resolve_err_works() {
        let error = VmError::resolve_err("function has different signature");
        match error {
            VmError::ResolveErr { msg, .. } => assert_eq!(msg, "function has different signature"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn runtime_err_works() {
        let error = VmError::runtime_err("something went wrong");
        match error {
            VmError::RuntimeErr { msg, .. } => assert_eq!(msg, "something went wrong"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn static_validation_err_works() {
        let error = VmError::static_validation_err("export xy missing");
        match error {
            VmError::StaticValidationErr { msg, .. } => assert_eq!(msg, "export xy missing"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn uninitialized_context_data_works() {
        let error = VmError::uninitialized_context_data("foo");
        match error {
            VmError::UninitializedContextData { kind, .. } => assert_eq!(kind, "foo"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn write_access_denied() {
        let error = VmError::write_access_denied();
        match error {
            VmError::WriteAccessDenied { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }
}
