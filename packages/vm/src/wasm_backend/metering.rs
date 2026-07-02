//! # Metering infrastructure.

use crate::parsed_wasm::ParsedWasm;
use std::sync::{Arc, Mutex};
use wasmer::wasmparser::{BlockType, Operator};
use wasmer::{
    ExportIndex, FunctionMiddleware, GlobalInit, GlobalType, LocalFunctionIndex, MiddlewareError,
    MiddlewareReaderState, ModuleMiddleware, Mutability, Type,
};
use wasmer_types::{GlobalIndex, ModuleInfo};

/// Minimum number of local variables in a function
/// that incur charging with additional gas points.
const CHARGED_LOCALS_THRESHOLD: usize = 30;

/// Indexes of Wasm global variables for tracking metering data.
#[derive(Debug, Clone)]
struct MeteringGlobalIndexes(
    /// Remaining gas points.
    GlobalIndex,
    /// Points exhausted flag.
    GlobalIndex,
    /// Data length of bulk-memory operation.
    GlobalIndex,
    /// Dynamic cost of bulk-memory operation.
    GlobalIndex,
);

impl MeteringGlobalIndexes {
    /// The global index in the current module for tracking remaining gas points.
    fn remaining_points(&self) -> GlobalIndex {
        self.0
    }

    /// The global index in the current module for a boolean indicating
    /// whether points are exhausted or not.
    ///
    /// This boolean is represented as `i32` global variable:
    ///   * 0: there are remaining gas points,
    ///   * 1: gas points have been exhausted.
    fn points_exhausted(&self) -> GlobalIndex {
        self.1
    }

    /// The global index in the current module for tracking
    /// the length of data in the bulk-memory operation.
    /// This data length is originally available on the top of the stack
    /// just before the bulk-memory operation is executed.
    /// This variable saves this data length temporarily, to enable
    /// metering calculations and then restores its value
    /// back on the top of the stack.
    fn data_length(&self) -> GlobalIndex {
        self.2
    }

    /// The global index in the current module for tracking
    /// the total dynamic cost of the bulk-memory operation.
    fn dynamic_cost(&self) -> GlobalIndex {
        self.3
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
pub struct Metering<F: Fn(&Operator) -> (u64, u64, u64) + Send + Sync> {
    /// Initial limit of gas points.
    initial_limit: u64,
    /// Function that maps each operator to a cost in gas points.
    cost_function: Arc<F>,
    /// The global indexes for metering gas points.
    global_indexes: Mutex<Option<MeteringGlobalIndexes>>,
    /// Number of locals in all functions defined in the module.
    function_locals: Vec<usize>,
}

impl<F: Fn(&Operator) -> (u64, u64, u64) + Send + Sync> std::fmt::Debug for Metering<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Metering")
            .field("initial_limit", &self.initial_limit)
            .field("cost_function", &"<cost_function>")
            .field("global_indexes", &self.global_indexes)
            .field("function_locals", &self.function_locals)
            .finish()
    }
}

impl<F: Fn(&Operator) -> (u64, u64, u64) + Send + Sync> Metering<F> {
    /// Creates a `Metering` middleware.
    ///
    /// When providing a cost function, you should consider that branching operations do
    /// additional work to track the metering points and probably need to have a higher cost.
    /// To find out which operations are affected by this, you can call [`is_branching_operator`].
    pub fn new(initial_limit: u64, cost_function: F, parsed_wasm: Option<ParsedWasm>) -> Self {
        Self {
            initial_limit,
            cost_function: Arc::new(cost_function),
            global_indexes: Mutex::new(None),
            function_locals: parsed_wasm.map_or_else(Vec::new, |inner| inner.func_locals),
        }
    }
}

impl<F: Fn(&Operator) -> (u64, u64, u64) + Send + Sync + 'static> ModuleMiddleware for Metering<F> {
    /// Generates a function middleware for a given function identified by provided index.
    fn generate_function_middleware(&self, idx: LocalFunctionIndex) -> Box<dyn FunctionMiddleware> {
        let locals_count = self
            .function_locals
            .get(idx.as_u32() as usize)
            .copied()
            .unwrap_or_default();
        Box::new(FunctionMetering {
            is_first_operator: true,
            charged_locals_count: locals_count.saturating_sub(CHARGED_LOCALS_THRESHOLD - 1) as u64,
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

        // Append a global variable for tracking remaining points and initialize it.
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

        // Append a global variable for the exhausted points boolean flag and initialize it.
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

        // Append a global variable for data length of the bulk-memory operation and initialize it.
        let data_length_global_index = module_info
            .globals
            .push(GlobalType::new(Type::I32, Mutability::Var));

        module_info
            .global_initializers
            .push(GlobalInit::I32Const(0));

        module_info.exports.insert(
            "wasmer_metering_data_length".to_string(),
            ExportIndex::Global(data_length_global_index),
        );

        // Append a global variable for dynamic cost of the bulk memory operation and initialize it.
        let dynamic_cost_global_index = module_info
            .globals
            .push(GlobalType::new(Type::I64, Mutability::Var));

        module_info
            .global_initializers
            .push(GlobalInit::I64Const(0));

        module_info.exports.insert(
            "wasmer_metering_dynamic_cost".to_string(),
            ExportIndex::Global(dynamic_cost_global_index),
        );

        // Initialize global indexes.
        *global_indexes = Some(MeteringGlobalIndexes(
            remaining_points_global_index,
            points_exhausted_global_index,
            data_length_global_index,
            dynamic_cost_global_index,
        ));

        Ok(())
    }
}

/// The function-level metering middleware.
pub struct FunctionMetering<F: Fn(&Operator) -> (u64, u64, u64) + Send + Sync> {
    /// Flag indicating if the first operator in function was encountered.
    is_first_operator: bool,
    /// Function that maps each operator to a cost in gas points.
    cost_function: Arc<F>,
    /// The global indexes for metering gas points.
    global_indexes: MeteringGlobalIndexes,
    /// Accumulated cost of the current basic block.
    accumulated_cost: u64,
    /// Number of local variables in function charged with additional gas points.
    charged_locals_count: u64,
}

impl<F: Fn(&Operator) -> (u64, u64, u64) + Send + Sync> std::fmt::Debug for FunctionMetering<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionMetering")
            .field("is_first_operator", &self.is_first_operator)
            .field("cost_function", &"<cost_function>")
            .field("global_indexes", &self.global_indexes)
            .field("accumulated_cost", &self.accumulated_cost)
            .field("charged_locals_count", &self.charged_locals_count)
            .finish()
    }
}

impl<F: Fn(&Operator) -> (u64, u64, u64) + Send + Sync> FunctionMiddleware for FunctionMetering<F> {
    fn feed<'a>(
        &mut self,
        operator: Operator<'a>,
        state: &mut MiddlewareReaderState<'a>,
    ) -> Result<(), MiddlewareError> {
        // If the first operator is encountered in a function
        // having a large number of locals, then charge additional gas.
        if self.is_first_operator && self.charged_locals_count > 0 {
            // Calculate the total gas cost for all charged locals in function.
            let (single_local_cost, _, _) = (self.cost_function)(&Operator::Nop);
            let locals_cost = single_local_cost.saturating_mul(self.charged_locals_count);
            if is_branching_operator(&operator) {
                // If the first operator is an accounting operator, then gas charging code
                // will be injected anyway, so it is enough to increase the accumulated cost.
                self.accumulated_cost += locals_cost;
            } else {
                // Otherwise, inject code for charging gas at the beginning of the function body.
                state.extend(gas_check_branching_wasm_code(
                    &self.global_indexes,
                    locals_cost,
                ));
            }
        }

        // Get the cost of the current operator, and add it to the accumulator.
        // This needs to be done before the metering logic, to prevent operators like `Call`
        // from escaping metering in some corner cases.
        let (operator_cost, unit_cost, unit_size) = (self.cost_function)(&operator);
        self.accumulated_cost += operator_cost;

        // For branching operator, finalize the cost of the previous basic block
        // and then perform necessary checks.
        if is_branching_operator(&operator) && self.accumulated_cost > 0 {
            // Inject code for charging gas before the accounting operator.
            state.extend(gas_check_branching_wasm_code(
                &self.global_indexes,
                self.accumulated_cost,
            ));
            self.accumulated_cost = 0;
        }

        // When the unit cost for the operator is non-zero (bulk-memory operator),
        // then inject dynamic cost calculations and perform necessary checks.
        if unit_cost > 0 {
            // Inject code for charging gas before bulk-memory operator.
            state.extend(gas_check_bulk_memory_wasm_code(
                &self.global_indexes,
                unit_cost,
                unit_size,
                self.accumulated_cost,
            ));
            self.accumulated_cost = 0;
        }

        // Push current operator.
        state.push_operator(operator);

        // Clear first operator flag.
        self.is_first_operator = false;
        Ok(())
    }
}

/// Returns `true` when the given operator is an accounting operator.
/// Accounting operators need additional work to track the metering points.
pub fn is_branching_operator(operator: &Operator) -> bool {
    // Possible sources and targets of a branch.
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

/// Returns Wasm code for charging accumulated cost
/// and checking remaining gas points.
fn gas_check_branching_wasm_code<'a>(
    global_indexes: &MeteringGlobalIndexes,
    accumulated_cost: u64,
) -> [Operator<'a>; 12] {
    let idx_remaining_points = global_indexes.remaining_points().as_u32();
    let idx_points_exhausted = global_indexes.points_exhausted().as_u32();
    [
        Operator::GlobalGet {
            global_index: idx_remaining_points,
        },
        Operator::I64Const {
            value: accumulated_cost as i64,
        },
        Operator::I64LtU,
        Operator::If {
            blockty: BlockType::Empty,
        },
        Operator::I32Const { value: 1 },
        Operator::GlobalSet {
            global_index: idx_points_exhausted,
        },
        Operator::Unreachable,
        Operator::End,
        Operator::GlobalGet {
            global_index: idx_remaining_points,
        },
        Operator::I64Const {
            value: accumulated_cost as i64,
        },
        Operator::I64Sub,
        Operator::GlobalSet {
            global_index: idx_remaining_points,
        },
    ]
}

/// Returns Wasm code for charging bulk memory operation cost,
/// accumulated cost and checking remaining gas points.
///
/// # Algorithm
///
/// ```wat
/// global.set 2         ;; Pop $length and save in global
/// global.get 2         ;; Push $length
/// i64.extend_i32_u     ;; Convert i32 $length to i64 value
/// i64.const 31         ;; Push $decrUnitSize
/// i64.add              ;; Add $length + $decUnitSize
/// i64.const 32         ;; Push $unitSize
/// i64.div_u            ;; Div ($length + $decUnitSize) / $unitSize
/// i64.const 13         ;; Push $unitCost
/// i64.mul              ;; Mul (($length + $decUnitSize) / $unitSize) * $unitCost
/// i64.const 3          ;; Push $accumulatedCost
/// i64.add              ;; $dynamicCost is on top of the stack
/// global.set 3         ;; Pop $dynamicCost and save in global
/// global.get 0         ;; Push $remainingPoints
/// global.get 3         ;; Push $dynamicCost from global
/// i64.lt_u             ;; bool($remainingPoints < $dynamicCost)
/// if                   ;; if 1
///   i32.const 1        ;; Prepare exhausted flag
///   global.set 1       ;; Save exhausted flag in global
///   unreachable        ;; Break execution
/// end                  ;; end if 1
/// global.get 0         ;; Push $remainingPoints from global
/// global.get 3         ;; Push $dynamicCost from global
/// i64.sub              ;; Subtract $remainingPoints - $dynamicCost
/// global.set 0         ;; Save $remainingPoints in global
/// global.get 2         ;; Push $length
/// ```
fn gas_check_bulk_memory_wasm_code<'a>(
    global_indexes: &MeteringGlobalIndexes,
    unit_cost: u64,
    unit_size: u64,
    accumulated_cost: u64,
) -> [Operator<'a>; 25] {
    let idx_remaining_points = global_indexes.remaining_points().as_u32();
    let idx_points_exhausted = global_indexes.points_exhausted().as_u32();
    let idx_data_length = global_indexes.data_length().as_u32();
    let idx_dynamic_cost = global_indexes.dynamic_cost().as_u32();
    let dec_unit_size = unit_size.saturating_sub(1);
    [
        Operator::GlobalSet {
            global_index: idx_data_length,
        },
        Operator::GlobalGet {
            global_index: idx_data_length,
        },
        Operator::I64ExtendI32U,
        Operator::I64Const {
            value: dec_unit_size as i64,
        },
        Operator::I64Add,
        Operator::I64Const {
            value: unit_size as i64,
        },
        Operator::I64DivU,
        Operator::I64Const {
            value: unit_cost as i64,
        },
        Operator::I64Mul,
        Operator::I64Const {
            value: accumulated_cost as i64,
        },
        Operator::I64Add,
        Operator::GlobalSet {
            global_index: idx_dynamic_cost,
        },
        Operator::GlobalGet {
            global_index: idx_remaining_points,
        },
        Operator::GlobalGet {
            global_index: idx_dynamic_cost,
        },
        Operator::I64LtU,
        Operator::If {
            blockty: BlockType::Empty,
        },
        Operator::I32Const { value: 1 },
        Operator::GlobalSet {
            global_index: idx_points_exhausted,
        },
        Operator::Unreachable,
        Operator::End,
        Operator::GlobalGet {
            global_index: idx_remaining_points,
        },
        Operator::GlobalGet {
            global_index: idx_dynamic_cost,
        },
        Operator::I64Sub,
        Operator::GlobalSet {
            global_index: idx_remaining_points,
        },
        Operator::GlobalGet {
            global_index: idx_data_length,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Dummy cost function to be used in the following tests.
    fn cost(_: &Operator) -> (u64, u64, u64) {
        (1, 0, 0)
    }

    #[test]
    fn debug_for_metering_works() {
        assert_eq!((1, 0, 0), cost(&Operator::Nop));
        assert_eq!(
            "Metering { initial_limit: 0, cost_function: \"<cost_function>\", global_indexes: Mutex { data: None, poisoned: false, .. }, function_locals: [] }",
            format!("{:?}", Metering::new(0, cost, None))
        );
    }

    #[test]
    fn debug_for_function_metering_works() {
        assert_eq!((1, 0, 0), cost(&Operator::Nop));
        let metering = Metering::new(0, cost, None);
        metering
            .transform_module_info(&mut ModuleInfo::new())
            .unwrap();
        assert_eq!(
            "FunctionMetering { is_first_operator: true, cost_function: \"<cost_function>\", global_indexes: MeteringGlobalIndexes(GlobalIndex(0), GlobalIndex(1), GlobalIndex(2), GlobalIndex(3)), accumulated_cost: 0, charged_locals_count: 0 }",
            format!("{:?}", metering.generate_function_middleware(LocalFunctionIndex::from_u32(0)))
        );
    }

    #[test]
    #[should_panic(
        expected = "Metering::transform_module_info: Attempting to use a `Metering` middleware from multiple modules."
    )]
    fn using_metering_multiple_times_should_panic() {
        assert_eq!((1, 0, 0), cost(&Operator::Nop));
        let metering = Metering::new(0, cost, None);
        let mut module_1 = ModuleInfo::new();
        let mut module_2 = ModuleInfo::new();
        metering.transform_module_info(&mut module_1).unwrap();
        metering.transform_module_info(&mut module_2).unwrap();
    }

    #[test]
    fn branching_and_bulk_memory_operators_must_be_disjoint() {
        // TODO Implement test.
    }
}
