use wasmer::wasmparser::Operator;
use wasmer::{
    FunctionMiddleware, LocalFunctionIndex, MiddlewareError, MiddlewareReaderState,
    ModuleMiddleware,
};

/// A middleware that ensures only deterministic operations are used (i.e. no floats)
#[derive(Debug)]
pub struct Deterministic {}

impl Deterministic {
    pub fn new() -> Self {
        Self {}
    }
}

impl ModuleMiddleware for Deterministic {
    /// Generates a `FunctionMiddleware` for a given function.
    fn generate_function_middleware(&self, _: LocalFunctionIndex) -> Box<dyn FunctionMiddleware> {
        Box::new(FunctionDeterministic {})
    }
}

#[derive(Debug)]
pub struct FunctionDeterministic {}

impl FunctionMiddleware for FunctionDeterministic {
    fn feed<'a>(
        &mut self,
        operator: Operator<'a>,
        state: &mut MiddlewareReaderState<'a>,
    ) -> Result<(), MiddlewareError> {
        match operator {
            Operator::Unreachable
            | Operator::Nop
            | Operator::Block { .. }
            | Operator::Loop { .. }
            | Operator::If { .. }
            | Operator::Else
            | Operator::End
            | Operator::Br { .. }
            | Operator::BrIf { .. }
            | Operator::BrTable { .. }
            | Operator::Return
            | Operator::Call { .. }
            | Operator::CallIndirect { .. }
            | Operator::Drop
            | Operator::Select
            | Operator::LocalGet { .. }
            | Operator::LocalSet { .. }
            | Operator::LocalTee { .. }
            | Operator::GlobalGet { .. }
            | Operator::GlobalSet { .. }
            | Operator::I32Load { .. }
            | Operator::I64Load { .. }
            | Operator::I32Load8S { .. }
            | Operator::I32Load8U { .. }
            | Operator::I32Load16S { .. }
            | Operator::I32Load16U { .. }
            | Operator::I64Load8S { .. }
            | Operator::I64Load8U { .. }
            | Operator::I64Load16S { .. }
            | Operator::I64Load16U { .. }
            | Operator::I64Load32S { .. }
            | Operator::I64Load32U { .. }
            | Operator::I32Store { .. }
            | Operator::I64Store { .. }
            | Operator::I32Store8 { .. }
            | Operator::I32Store16 { .. }
            | Operator::I64Store8 { .. }
            | Operator::I64Store16 { .. }
            | Operator::I64Store32 { .. }
            | Operator::MemorySize { .. }
            | Operator::MemoryGrow { .. }
            | Operator::I32Const { .. }
            | Operator::I64Const { .. }
            | Operator::I32Eqz
            | Operator::I32Eq
            | Operator::I32Ne
            | Operator::I32LtS
            | Operator::I32LtU
            | Operator::I32GtS
            | Operator::I32GtU
            | Operator::I32LeS
            | Operator::I32LeU
            | Operator::I32GeS
            | Operator::I32GeU
            | Operator::I64Eqz
            | Operator::I64Eq
            | Operator::I64Ne
            | Operator::I64LtS
            | Operator::I64LtU
            | Operator::I64GtS
            | Operator::I64GtU
            | Operator::I64LeS
            | Operator::I64LeU
            | Operator::I64GeS
            | Operator::I64GeU
            | Operator::I32Clz
            | Operator::I32Ctz
            | Operator::I32Popcnt
            | Operator::I32Add
            | Operator::I32Sub
            | Operator::I32Mul
            | Operator::I32DivS
            | Operator::I32DivU
            | Operator::I32RemS
            | Operator::I32RemU
            | Operator::I32And
            | Operator::I32Or
            | Operator::I32Xor
            | Operator::I32Shl
            | Operator::I32ShrS
            | Operator::I32ShrU
            | Operator::I32Rotl
            | Operator::I32Rotr
            | Operator::I64Clz
            | Operator::I64Ctz
            | Operator::I64Popcnt
            | Operator::I64Add
            | Operator::I64Sub
            | Operator::I64Mul
            | Operator::I64DivS
            | Operator::I64DivU
            | Operator::I64RemS
            | Operator::I64RemU
            | Operator::I64And
            | Operator::I64Or
            | Operator::I64Xor
            | Operator::I64Shl
            | Operator::I64ShrS
            | Operator::I64ShrU
            | Operator::I64Rotl
            | Operator::I64Rotr
            | Operator::I32WrapI64
            | Operator::I32Extend8S
            | Operator::I32Extend16S
            | Operator::I64Extend8S
            | Operator::I64Extend16S
            | Operator::I64ExtendI32S
            | Operator::I64ExtendI32U => {
                state.push_operator(operator);
                Ok(())
            }
            _ => {
                let msg = format!("Non-deterministic operator detected: {:?}", operator);
                Err(MiddlewareError::new("Deterministic", msg))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use wasmer::{CompilerConfig, Cranelift, Module, Store, JIT};

    #[test]
    fn valid_wasm_instance_sanity() {
        let wasm = wat::parse_str(
            r#"
            (module
                (func (export "sum") (param i32 i32) (result i32)
                    get_local 0
                    get_local 1
                    i32.add
                ))
            "#,
        )
        .unwrap();

        let deterministic = Arc::new(Deterministic::new());
        let mut compiler_config = Cranelift::default();
        compiler_config.push_middleware(deterministic);
        let store = Store::new(&JIT::new(compiler_config).engine());
        let result = Module::new(&store, &wasm);
        assert!(result.is_ok());
    }

    #[test]
    fn parser_floats_are_not_supported() {
        let wasm = wat::parse_str(
            r#"
            (module
                (func $to_float (param i32) (result f32)
                    get_local 0
                    f32.convert_u/i32
                ))
            "#,
        )
        .unwrap();

        let deterministic = Arc::new(Deterministic::new());
        let mut compiler_config = Cranelift::default();
        compiler_config.push_middleware(deterministic);
        let store = Store::new(&JIT::new(compiler_config).engine());
        let result = Module::new(&store, &wasm);
        assert!(result.is_err());
    }
}
