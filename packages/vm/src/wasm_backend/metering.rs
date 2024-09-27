use std::fmt;
use std::sync::{Arc, Mutex};
use wasmer::wasmparser::{BlockType as WpTypeOrFuncType, Operator};
use wasmer::{
    ExportIndex, FunctionMiddleware, GlobalInit, GlobalType, LocalFunctionIndex, MiddlewareError,
    MiddlewareReaderState, ModuleMiddleware, Mutability, Type,
};
use wasmer_types::{GlobalIndex, ModuleInfo};

#[derive(Clone)]
struct MeteringGlobalIndexes(GlobalIndex, GlobalIndex);

impl MeteringGlobalIndexes {
    /// The global index in the current module for remaining points.
    fn remaining_points(&self) -> GlobalIndex {
        self.0
    }

    /// The global index in the current module for a boolean indicating whether points are exhausted
    /// or not.
    /// This boolean is represented as a i32 global:
    ///   * 0: there are remaining points
    ///   * 1: points have been exhausted
    fn points_exhausted(&self) -> GlobalIndex {
        self.1
    }
}

impl fmt::Debug for MeteringGlobalIndexes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MeteringGlobalIndexes")
            .field("remaining_points", &self.remaining_points())
            .field("points_exhausted", &self.points_exhausted())
            .finish()
    }
}

/// The module-level metering middleware.
///
/// # Panic
///
/// An instance of `Metering` should _not_ be shared among different
/// modules, since it tracks module-specific information like the
/// global index to store metering state. Attempts to use a `Metering`
/// instance from multiple modules will result in a panic.
///
/// # Example
///
/// ```rust
/// use std::sync::Arc;
/// use wasmer::{wasmparser::Operator, CompilerConfig};
/// use wasmer_middlewares::Metering;
///
/// fn create_metering_middleware(compiler_config: &mut dyn CompilerConfig) {
///     // Let's define a dummy cost function,
///     // which counts 1 for all operators.
///     let cost_function = |_operator: &Operator| -> u64 { 1 };
///
///     // Let's define the initial limit.
///     let initial_limit = 10;
///
///     // Let's creating the metering middleware.
///     let metering = Arc::new(Metering::new(
///         initial_limit,
///         cost_function
///     ));
///
///     // Finally, let's push the middleware.
///     compiler_config.push_middleware(metering);
/// }
/// ```
pub struct Metering<F: Fn(&Operator) -> u64 + Send + Sync> {
    /// Initial limit of points.
    initial_limit: u64,

    /// Function that maps each operator to a cost in "points".
    cost_function: Arc<F>,

    /// The global indexes for metering points.
    global_indexes: Mutex<Option<MeteringGlobalIndexes>>,
}

/// The function-level metering middleware.
pub struct FunctionMetering<F: Fn(&Operator) -> u64 + Send + Sync> {
    /// Function that maps each operator to a cost in "points".
    cost_function: Arc<F>,

    /// The global indexes for metering points.
    global_indexes: MeteringGlobalIndexes,

    /// Accumulated cost of the current basic block.
    accumulated_cost: u64,
}

impl<F: Fn(&Operator) -> u64 + Send + Sync> Metering<F> {
    /// Creates a `Metering` middleware.
    pub fn new(initial_limit: u64, cost_function: F) -> Self {
        Self {
            initial_limit,
            cost_function: Arc::new(cost_function),
            global_indexes: Mutex::new(None),
        }
    }
}

/// Returns `true` if and only if the given operator is an accounting operator.
/// Accounting operators do additional work to track the metering points.
pub fn is_accounting(operator: &Operator) -> bool {
    matches!(
        operator,
        Operator::Loop { .. } // loop headers are branch targets
            | Operator::End // block ends are branch targets
            | Operator::If { .. } // branch source, "if" can branch to else branch
            | Operator::Else // "else" is the "end" of an if branch
            | Operator::Br { .. } // branch source
            | Operator::BrTable { .. } // branch source
            | Operator::BrIf { .. } // branch source
            | Operator::Call { .. } // function call - branch source
            | Operator::CallIndirect { .. } // function call - branch source
            | Operator::Return // end of function - branch source
            // exceptions proposal
            | Operator::Throw { .. } // branch source
            | Operator::ThrowRef // branch source
            | Operator::Rethrow { .. } // branch source
            | Operator::Delegate { .. } // branch source
            | Operator::Catch { .. } // branch target
            // tail_call proposal
            | Operator::ReturnCall { .. } // branch source
            | Operator::ReturnCallIndirect { .. } // branch source
            // gc proposal
            | Operator::BrOnCast { .. } // branch source
            | Operator::BrOnCastFail { .. } // branch source
            // function_references proposal
            | Operator::CallRef { .. } // branch source
            | Operator::ReturnCallRef { .. } // branch source
            | Operator::BrOnNull { .. } // branch source
            | Operator::BrOnNonNull { .. } // branch source
    )
}

impl<F: Fn(&Operator) -> u64 + Send + Sync> fmt::Debug for Metering<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Metering")
            .field("initial_limit", &self.initial_limit)
            .field("cost_function", &"<function>")
            .field("global_indexes", &self.global_indexes)
            .finish()
    }
}

impl<F: Fn(&Operator) -> u64 + Send + Sync + 'static> ModuleMiddleware for Metering<F> {
    /// Generates a `FunctionMiddleware` for a given function.
    fn generate_function_middleware(&self, _: LocalFunctionIndex) -> Box<dyn FunctionMiddleware> {
        Box::new(FunctionMetering {
            cost_function: self.cost_function.clone(),
            global_indexes: self.global_indexes.lock().unwrap().clone().unwrap(),
            accumulated_cost: 0,
        })
    }

    /// Transforms a `ModuleInfo` struct in-place. This is called before application on functions begins.
    fn transform_module_info(&self, module_info: &mut ModuleInfo) -> Result<(), MiddlewareError> {
        let mut global_indexes = self.global_indexes.lock().unwrap();

        if global_indexes.is_some() {
            panic!("Metering::transform_module_info: Attempting to use a `Metering` middleware from multiple modules.");
        }

        // Append a global for remaining points and initialize it.
        let remaining_points_global_index = module_info
            .globals
            .push(GlobalType::new(Type::I64, Mutability::Var));

        module_info
            .global_initializers
            .push(GlobalInit::I64Const(self.initial_limit as i64));

        module_info.exports.insert(
            "wasmer_metering_remaining_points".to_string(),
            ExportIndex::Global(remaining_points_global_index),
        );

        // Append a global for the exhausted points boolean and initialize it.
        let points_exhausted_global_index = module_info
            .globals
            .push(GlobalType::new(Type::I32, Mutability::Var));

        module_info
            .global_initializers
            .push(GlobalInit::I32Const(0));

        module_info.exports.insert(
            "wasmer_metering_points_exhausted".to_string(),
            ExportIndex::Global(points_exhausted_global_index),
        );

        *global_indexes = Some(MeteringGlobalIndexes(
            remaining_points_global_index,
            points_exhausted_global_index,
        ));

        Ok(())
    }
}

impl<F: Fn(&Operator) -> u64 + Send + Sync> fmt::Debug for FunctionMetering<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FunctionMetering")
            .field("cost_function", &"<function>")
            .field("global_indexes", &self.global_indexes)
            .finish()
    }
}

impl<F: Fn(&Operator) -> u64 + Send + Sync> FunctionMiddleware for FunctionMetering<F> {
    fn feed<'a>(
        &mut self,
        operator: Operator<'a>,
        state: &mut MiddlewareReaderState<'a>,
    ) -> Result<(), MiddlewareError> {
        // Get the cost of the current operator, and add it to the accumulator.
        // This needs to be done before the metering logic, to prevent operators like `Call` from escaping metering in some
        // corner cases.
        self.accumulated_cost += (self.cost_function)(&operator);

        // Possible sources and targets of a branch. Finalize the cost of the previous basic block and perform necessary checks.
        if is_accounting(&operator) && self.accumulated_cost > 0 {
            state.extend(&[
                // if unsigned(globals[remaining_points_index]) < unsigned(self.accumulated_cost) { throw(); }
                Operator::GlobalGet {
                    global_index: self.global_indexes.remaining_points().as_u32(),
                },
                Operator::I64Const {
                    value: self.accumulated_cost as i64,
                },
                Operator::I64LtU,
                Operator::If {
                    blockty: WpTypeOrFuncType::Empty,
                },
                Operator::I32Const { value: 1 },
                Operator::GlobalSet {
                    global_index: self.global_indexes.points_exhausted().as_u32(),
                },
                Operator::Unreachable,
                Operator::End,
                // globals[remaining_points_index] -= self.accumulated_cost;
                Operator::GlobalGet {
                    global_index: self.global_indexes.remaining_points().as_u32(),
                },
                Operator::I64Const {
                    value: self.accumulated_cost as i64,
                },
                Operator::I64Sub,
                Operator::GlobalSet {
                    global_index: self.global_indexes.remaining_points().as_u32(),
                },
            ]);

            self.accumulated_cost = 0;
        }
        state.push_operator(operator);

        Ok(())
    }
}
