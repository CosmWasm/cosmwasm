use wasmer::wasmparser::Operator;
use wasmer::{
    FunctionMiddleware, LocalFunctionIndex, MiddlewareError, MiddlewareReaderState,
    ModuleMiddleware,
};

#[derive(Debug, Clone, Copy)]
struct GatekeeperConfig {
    /// True iff float operations are allowed.
    ///
    /// Note: there are float operations in the SIMD block as well and we do not yet handle
    /// any combination of `allow_floats` and `allow_feature_simd` properly.
    allow_floats: bool,
    //
    // Standardized features
    //
    /// True iff operations of the "Bulk memory operations" feature are allowed.
    /// See <https://webassembly.org/roadmap/> and <https://github.com/WebAssembly/bulk-memory-operations/blob/master/proposals/bulk-memory-operations/Overview.md>.
    allow_feature_bulk_memory_operations: bool,
    /// True iff operations of the "Reference types" feature are allowed.
    /// See <https://webassembly.org/roadmap/> and <https://github.com/WebAssembly/reference-types/blob/master/proposals/reference-types/Overview.md>.
    allow_feature_reference_types: bool,
    /// True iff operations of the "Fixed-width SIMD" feature are allowed.
    /// See <https://webassembly.org/roadmap/> and <https://github.com/WebAssembly/simd/blob/master/proposals/simd/SIMD.md>.
    allow_feature_simd: bool,
    //
    // In-progress proposals
    //
    /// True iff operations of the "Exception handling" feature are allowed.
    /// Note, this feature is not yet standardized!
    /// See <https://webassembly.org/roadmap/> and <https://github.com/WebAssembly/exception-handling/blob/master/proposals/exception-handling/Exceptions.md>.
    allow_feature_exception_handling: bool,
    /// True iff operations of the "Threads and atomics" feature are allowed.
    /// Note, this feature is not yet standardized!
    /// See <https://webassembly.org/roadmap/> and <https://github.com/WebAssembly/threads/blob/master/proposals/threads/Overview.md>.
    allow_feature_threads: bool,
    /// True iff operations of the "Shared-Everything Threads" feature are allowed.
    /// Note, this feature is not yet standardized!
    /// See <https://github.com/WebAssembly/shared-everything-threads>.
    allow_shared_everything_threads: bool,
}

/// A middleware that ensures only deterministic operations are used (i.e. no floats).
/// It also disallows the use of Wasm features that are not explicitly enabled.
#[derive(Debug)]
#[non_exhaustive]
pub struct Gatekeeper {
    config: GatekeeperConfig,
}

impl Gatekeeper {
    /// Creates a new Gatekeeper with a custom config.
    ///
    /// A custom configuration is potentially dangerous (non-final Wasm proposals, floats in SIMD operation).
    /// For this reason, only [`Gatekeeper::default()`] is public.
    fn new(config: GatekeeperConfig) -> Self {
        Self { config }
    }
}

impl Default for Gatekeeper {
    fn default() -> Self {
        Self::new(GatekeeperConfig {
            allow_floats: true,
            allow_feature_bulk_memory_operations: false,
            // we allow the reference types proposal during compatibility checking because a subset
            // of it is required since Rust 1.82, but we don't allow any of the instructions specific
            // to the proposal here. Especially `table.grow` and `table.fill` can be abused to cause
            // very long runtime and high memory usage.
            allow_feature_reference_types: false,
            allow_feature_simd: false,
            allow_feature_exception_handling: false,
            allow_feature_threads: false,
            allow_shared_everything_threads: false,
        })
    }
}

impl ModuleMiddleware for Gatekeeper {
    /// Generates a `FunctionMiddleware` for a given function.
    fn generate_function_middleware(&self, _: LocalFunctionIndex) -> Box<dyn FunctionMiddleware> {
        Box::new(FunctionGatekeeper::new(self.config))
    }
}

#[derive(Debug)]
#[non_exhaustive]
struct FunctionGatekeeper {
    config: GatekeeperConfig,
}

impl FunctionGatekeeper {
    fn new(config: GatekeeperConfig) -> Self {
        Self { config }
    }
}

/// The name used in errors
const MIDDLEWARE_NAME: &str = "Gatekeeper";

impl FunctionMiddleware for FunctionGatekeeper {
    fn feed<'a>(
        &mut self,
        operator: Operator<'a>,
        state: &mut MiddlewareReaderState<'a>,
    ) -> Result<(), MiddlewareError> {
        /// Creates a match arm for the given operator.
        /// Intended to be used inside a `match`.
        macro_rules! match_op {
            ($op:ident { $($payload:tt)* }) => {
                $op { .. }
            };
            ($op:ident) => {
                $op
            };
        }
        /// Matches on the given operator and calls the corresponding handler function.
        macro_rules! gatekeep {
            ($( @$proposal:ident $op:ident $({ $($payload:tt)* })? => $visit:ident)*) => {{
                use wasmer::wasmparser::Operator::*;

                let mut proposal_validator = ProposalValidator {
                    config: &self.config,
                    state,
                };


                match operator {
                    $(
                        match_op!($op $({ $($payload)* })?) => {
                            proposal_validator.$proposal(operator)
                        }
                    )*
                }
            }}
        }

        wasmer::wasmparser::for_each_operator!(gatekeep)
    }
}

struct ProposalValidator<'a, 'b> {
    config: &'b GatekeeperConfig,
    state: &'b mut MiddlewareReaderState<'a>,
}

impl<'a, 'b> ProposalValidator<'a, 'b> {
    /// Internal helper to deduplicate code
    fn _ref_types(&'b mut self, operator: Operator<'a>) -> Result<(), MiddlewareError> {
        if self.config.allow_feature_reference_types {
            self.state.push_operator(operator);
            Ok(())
        } else {
            let msg = format!("Reference type operation detected: {operator:?}. Reference types are not supported.");
            Err(MiddlewareError::new(MIDDLEWARE_NAME, msg))
        }
    }

    /// Internal helper to deduplicate code
    fn _floats(&'b mut self, operator: Operator<'a>) -> Result<(), MiddlewareError> {
        if self.config.allow_floats {
            self.state.push_operator(operator);
            Ok(())
        } else {
            let msg = format!(
                "Float operator detected: {operator:?}. The use of floats is not supported."
            );
            Err(MiddlewareError::new(MIDDLEWARE_NAME, msg))
        }
    }

    /// Internal helper to deduplicate code
    fn _exceptions(&'b mut self, operator: Operator<'a>) -> Result<(), MiddlewareError> {
        if self.config.allow_feature_exception_handling {
            self.state.push_operator(operator);
            Ok(())
        } else {
            let msg = format!("Exception handling operation detected: {operator:?}. Exception handling is not supported.");
            Err(MiddlewareError::new(MIDDLEWARE_NAME, msg))
        }
    }

    fn mvp(&'b mut self, operator: Operator<'a>) -> Result<(), MiddlewareError> {
        // special handling for float operators
        match operator {
            Operator::F32Load { .. }
            | Operator::F64Load { .. }
            | Operator::F32Store { .. }
            | Operator::F64Store { .. }
            | Operator::F32Const { .. }
            | Operator::F64Const { .. }
            | Operator::F32Eq
            | Operator::F32Ne
            | Operator::F32Lt
            | Operator::F32Gt
            | Operator::F32Le
            | Operator::F32Ge
            | Operator::F64Eq
            | Operator::F64Ne
            | Operator::F64Lt
            | Operator::F64Gt
            | Operator::F64Le
            | Operator::F64Ge
            | Operator::F32Abs
            | Operator::F32Neg
            | Operator::F32Ceil
            | Operator::F32Floor
            | Operator::F32Trunc
            | Operator::F32Nearest
            | Operator::F32Sqrt
            | Operator::F32Add
            | Operator::F32Sub
            | Operator::F32Mul
            | Operator::F32Div
            | Operator::F32Min
            | Operator::F32Max
            | Operator::F32Copysign
            | Operator::F64Abs
            | Operator::F64Neg
            | Operator::F64Ceil
            | Operator::F64Floor
            | Operator::F64Trunc
            | Operator::F64Nearest
            | Operator::F64Sqrt
            | Operator::F64Add
            | Operator::F64Sub
            | Operator::F64Mul
            | Operator::F64Div
            | Operator::F64Min
            | Operator::F64Max
            | Operator::F64Copysign
            | Operator::I32TruncF32S
            | Operator::I32TruncF32U
            | Operator::I32TruncF64S
            | Operator::I32TruncF64U
            | Operator::I64TruncF32S
            | Operator::I64TruncF32U
            | Operator::I64TruncF64S
            | Operator::I64TruncF64U
            | Operator::F32ConvertI32S
            | Operator::F32ConvertI32U
            | Operator::F32ConvertI64S
            | Operator::F32ConvertI64U
            | Operator::F32DemoteF64
            | Operator::F64ConvertI32S
            | Operator::F64ConvertI32U
            | Operator::F64ConvertI64S
            | Operator::F64ConvertI64U
            | Operator::F64PromoteF32
            | Operator::I32ReinterpretF32
            | Operator::I64ReinterpretF64
            | Operator::F32ReinterpretI32
            | Operator::F64ReinterpretI64 => self._floats(operator),
            // all other mvp operators
            _ => {
                self.state.push_operator(operator);
                Ok(())
            }
        }
    }

    /// Sign-extension
    /// https://github.com/bytecodealliance/wasm-tools/blob/wasmparser-0.107.0/crates/wasmparser/src/lib.rs#L307-L311
    #[inline]
    fn sign_extension(&'b mut self, operator: Operator<'a>) -> Result<(), MiddlewareError> {
        self.state.push_operator(operator);
        Ok(())
    }

    #[inline]
    fn function_references(&'b mut self, operator: Operator<'a>) -> Result<(), MiddlewareError> {
        self._ref_types(operator)
    }

    #[inline]
    fn reference_types(&'b mut self, operator: Operator<'a>) -> Result<(), MiddlewareError> {
        self._ref_types(operator)
    }

    #[inline]
    fn tail_call(&'b mut self, operator: Operator<'a>) -> Result<(), MiddlewareError> {
        self._ref_types(operator)
    }

    #[inline]
    fn threads(&'b mut self, operator: Operator<'a>) -> Result<(), MiddlewareError> {
        if self.config.allow_feature_threads {
            self.state.push_operator(operator);
            Ok(())
        } else {
            let msg = format!("Threads operator detected: {operator:?}. The Wasm Threads extension is not supported.");
            Err(MiddlewareError::new(MIDDLEWARE_NAME, msg))
        }
    }

    #[inline]
    fn shared_everything_threads(
        &'b mut self,
        operator: Operator<'a>,
    ) -> Result<(), MiddlewareError> {
        if self.config.allow_shared_everything_threads {
            self.state.push_operator(operator);
            Ok(())
        } else {
            let msg = format!("Shared-Everything threads operator detected: {operator:?}. The Wasm Shared-Everything Threads extension is not supported.");
            Err(MiddlewareError::new(MIDDLEWARE_NAME, msg))
        }
    }

    #[inline]
    fn simd(&'b mut self, operator: Operator<'a>) -> Result<(), MiddlewareError> {
        if self.config.allow_feature_simd {
            self.state.push_operator(operator);
            Ok(())
        } else {
            let msg = format!(
                "SIMD operator detected: {operator:?}. The Wasm SIMD extension is not supported."
            );
            Err(MiddlewareError::new(MIDDLEWARE_NAME, msg))
        }
    }

    #[inline]
    fn relaxed_simd(&'b mut self, operator: Operator<'a>) -> Result<(), MiddlewareError> {
        let msg = format!(
            "Relaxed SIMD operator detected: {operator:?}. The Wasm Relaxed SIMD extension is not supported."
        );
        Err(MiddlewareError::new(MIDDLEWARE_NAME, msg))
    }

    #[inline]
    fn saturating_float_to_int(
        &'b mut self,
        operator: Operator<'a>,
    ) -> Result<(), MiddlewareError> {
        self._floats(operator)
    }

    #[inline]
    fn bulk_memory(&'b mut self, operator: Operator<'a>) -> Result<(), MiddlewareError> {
        if self.config.allow_feature_bulk_memory_operations {
            self.state.push_operator(operator);
            Ok(())
        } else {
            let msg = format!("Bulk memory operation detected: {operator:?}. Bulk memory operations are not supported.");
            Err(MiddlewareError::new(MIDDLEWARE_NAME, msg))
        }
    }

    #[inline]
    fn legacy_exceptions(&'b mut self, operator: Operator<'a>) -> Result<(), MiddlewareError> {
        self._exceptions(operator)
    }

    #[inline]
    fn exceptions(&'b mut self, operator: Operator<'a>) -> Result<(), MiddlewareError> {
        self._exceptions(operator)
    }

    #[inline]
    fn gc(&'b mut self, operator: Operator<'a>) -> Result<(), MiddlewareError> {
        let msg = format!("GC operation detected: {operator:?}. GC Proposal is not supported.");
        Err(MiddlewareError::new(MIDDLEWARE_NAME, msg))
    }

    #[inline]
    fn memory_control(&'b mut self, operator: Operator<'a>) -> Result<(), MiddlewareError> {
        let msg = format!(
            "Memory control operation detected: {operator:?}. Memory control is not supported."
        );
        Err(MiddlewareError::new(MIDDLEWARE_NAME, msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wasm_backend::make_compiler_config;
    use std::sync::Arc;
    use wasmer::{CompilerConfig, Module, Store};

    #[test]
    fn valid_wasm_instance_sanity() {
        let wasm = wat::parse_str(
            r#"
            (module
                (func (export "sum") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add
                ))
            "#,
        )
        .unwrap();

        let deterministic = Arc::new(Gatekeeper::default());
        let mut compiler = make_compiler_config();
        compiler.push_middleware(deterministic);
        let store = Store::new(compiler);
        let result = Module::new(&store, wasm);
        assert!(result.is_ok());
    }

    #[test]
    fn parser_floats_are_supported() {
        let wasm = wat::parse_str(
            r#"
            (module
                (func $to_float (param i32) (result f32)
                    local.get 0
                    f32.convert_i32_u
                ))
            "#,
        )
        .unwrap();

        let deterministic = Arc::new(Gatekeeper::default());
        let mut compiler = make_compiler_config();
        compiler.push_middleware(deterministic);
        let store = Store::new(compiler);
        let result = Module::new(&store, wasm);
        assert!(result.is_ok());
    }

    #[test]
    fn bulk_operations_not_supported() {
        let wasm = wat::parse_str(
            r#"
            (module
              (memory (export "memory") 1)
              (func (param $dst i32) (param $src i32) (param $size i32) (result i32)
                local.get $dst
                local.get $src
                local.get $size
                memory.copy
                local.get $dst))
            "#,
        )
        .unwrap();

        let deterministic = Arc::new(Gatekeeper::default());
        let mut compiler = make_compiler_config();
        compiler.push_middleware(deterministic);
        let store = Store::new(compiler);
        let result = Module::new(&store, wasm);
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Bulk memory operation"));
    }

    #[test]
    fn bulk_table_operations_not_supported() {
        // these operations can take a long time with big tables
        let deterministic = Arc::new(Gatekeeper::default());
        let mut compiler = make_compiler_config();
        compiler.push_middleware(deterministic);
        let store = Store::new(compiler);

        let wasm = wat::parse_str(
            r#"
            (module
                (table 2 funcref)
                (func (export "test") (param $i i32) (result i32)
                    ;; grow table to size of $i
                    ref.null func
                    local.get $i
                    table.grow 0))
            "#,
        )
        .unwrap();

        let result = Module::new(&store, wasm);
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Reference type operation"));

        let wasm = wat::parse_str(
            r#"
            (module
                (table 1000000000 funcref)
                (func (export "test") (param $i i32)
                    ;; fill with nulls
                    i32.const 0
                    ref.null func
                    i32.const 1000000000
                    table.fill 0))
            "#,
        )
        .unwrap();

        let result = Module::new(&store, wasm);
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Reference type operation"));
    }
}
