use std::{fmt, mem};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use wasmer::wasmparser::{
    BinaryReaderError, CompositeType, Export, FuncToValidate, FunctionBody, Import, MemoryType,
    Parser, Payload, TableType, ValidPayload, Validator, ValidatorResources, WasmFeatures,
};

use crate::{VmError, VmResult};

/// Opaque wrapper type implementing `Debug`
///
/// The purpose of this type is to wrap types that do not implement `Debug` themselves.
/// For example, you have a large struct and derive `Debug` on it but one member does not implement the trait, that's where this type comes in.
///
/// Instead of printing a full debug representation of the underlying data, it simply prints something akin to this:
///
/// ```ignore
/// WrappedType { ... }
/// ```
#[derive(Default)]
pub struct OpaqueDebug<T>(pub T);

impl<T> fmt::Debug for OpaqueDebug<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(std::any::type_name::<T>())
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub enum FunctionValidator<'a> {
    Pending(OpaqueDebug<Vec<(FuncToValidate<ValidatorResources>, FunctionBody<'a>)>>),
    Success,
    Error(BinaryReaderError),
}

impl<'a> FunctionValidator<'a> {
    fn push(&mut self, item: (FuncToValidate<ValidatorResources>, FunctionBody<'a>)) {
        let Self::Pending(OpaqueDebug(ref mut funcs)) = self else {
            panic!("attempted to push function into non-pending validator");
        };

        funcs.push(item);
    }
}

/// A parsed and validated wasm module.
/// It keeps track of the parts that are important for our static analysis and compatibility checks.
#[derive(Debug)]
pub struct ParsedWasm<'a> {
    pub version: u16,
    pub exports: Vec<Export<'a>>,
    pub imports: Vec<Import<'a>>,
    pub tables: Vec<TableType>,
    pub memories: Vec<MemoryType>,
    pub function_count: usize,
    pub type_count: u32,
    /// How many parameters a type has.
    /// The index is the type id
    pub type_params: Vec<usize>,
    /// How many parameters the function with the most parameters has
    pub max_func_params: usize,
    /// How many results the function with the most results has
    pub max_func_results: usize,
    /// How many function parameters are used in the module
    pub total_func_params: usize,
    /// Collections of functions that are potentially pending validation
    pub func_validator: FunctionValidator<'a>,
}

impl<'a> ParsedWasm<'a> {
    pub fn parse(wasm: &'a [u8]) -> VmResult<Self> {
        let mut validator = Validator::new_with_features(WasmFeatures {
            mutable_global: true,
            saturating_float_to_int: true,
            sign_extension: true,
            multi_value: true,
            floats: true,

            reference_types: false,
            bulk_memory: false,
            simd: false,
            relaxed_simd: false,
            threads: false,
            tail_call: false,
            multi_memory: false,
            exceptions: false,
            memory64: false,
            extended_const: false,
            component_model: false,
            function_references: false,
            memory_control: false,
            gc: false,
            component_model_values: false,
            component_model_nested_names: false,
        });

        let mut this = Self {
            version: 0,
            exports: vec![],
            imports: vec![],
            tables: vec![],
            memories: vec![],
            function_count: 0,
            type_count: 0,
            type_params: Vec::new(),
            max_func_params: 0,
            max_func_results: 0,
            total_func_params: 0,
            func_validator: FunctionValidator::Pending(OpaqueDebug::default()),
        };

        for p in Parser::new(0).parse_all(wasm) {
            let p = p?;
            // validate the payload
            if let ValidPayload::Func(fv, body) = validator.payload(&p)? {
                // also validate function bodies
                this.func_validator.push((fv, body));
                this.function_count += 1;
            }

            match p {
                Payload::TypeSection(t) => {
                    this.type_count = 0;
                    // t.count() is a lower bound
                    this.type_params = Vec::with_capacity(t.count() as usize);
                    for group in t.into_iter() {
                        let types = group?.into_types();
                        // update count
                        this.type_count += types.len() as u32;

                        for ty in types {
                            match ty.composite_type {
                                CompositeType::Func(ft) => {
                                    this.type_params.push(ft.params().len());

                                    this.max_func_params =
                                        core::cmp::max(ft.params().len(), this.max_func_params);
                                    this.max_func_results =
                                        core::cmp::max(ft.results().len(), this.max_func_results);
                                }
                                CompositeType::Array(_) | CompositeType::Struct(_) => {
                                    // ignoring these for now, as they are only available with the GC
                                    // proposal and we explicitly disabled that above
                                }
                            }
                        }
                    }
                }
                Payload::FunctionSection(section) => {
                    // In valid Wasm, the function section always has to come after the type section
                    // (see https://www.w3.org/TR/2019/REC-wasm-core-1-20191205/#modules%E2%91%A0%E2%93%AA),
                    // so we can assume that the type_params map is already filled at this point

                    for a in section {
                        let type_index = a? as usize;
                        this.total_func_params +=
                            this.type_params.get(type_index).ok_or_else(|| {
                                // this will also be thrown if the wasm section order is invalid
                                VmError::static_validation_err(
                                    "Wasm bytecode error: function uses unknown type index",
                                )
                            })?
                    }
                }
                Payload::Version { num, .. } => this.version = num,
                Payload::ImportSection(i) => {
                    this.imports = i.into_iter().collect::<Result<Vec<_>, _>>()?;
                }
                Payload::TableSection(t) => {
                    this.tables = t
                        .into_iter()
                        .map(|r| r.map(|t| t.ty))
                        .collect::<Result<Vec<_>, _>>()?;
                }
                Payload::MemorySection(m) => {
                    this.memories = m.into_iter().collect::<Result<Vec<_>, _>>()?;
                }
                Payload::ExportSection(e) => {
                    this.exports = e.into_iter().collect::<Result<Vec<_>, _>>()?;
                }
                _ => {} // ignore everything else
            }
        }

        Ok(this)
    }

    /// Perform the expensive operation of validating each function body
    ///
    /// Note: This function caches the output of this function into the field `func_validator` so repeated invocations are cheap.
    pub fn validate_funcs(&mut self) -> VmResult<()> {
        match self.func_validator {
            FunctionValidator::Pending(OpaqueDebug(ref mut funcs)) => {
                let result = mem::take(funcs) // This is fine since `Vec::default()` doesn't allocate
                    .into_par_iter()
                    .try_for_each(|(func, body)| {
                        // Reusing the allocations between validations only results in an ~6% performance improvement
                        // The parallelization is blowing this out of the water by a magnitude of 5x
                        // Especially when combining this with a high-performance allocator, the missing buffer reuse will be pretty much irrelevant
                        let mut validator = func.into_validator(Default::default());
                        validator.validate(&body)?;
                        Ok(())
                    });

                self.func_validator = match result {
                    Ok(()) => FunctionValidator::Success,
                    Err(err) => FunctionValidator::Error(err),
                };

                self.validate_funcs()
            }
            FunctionValidator::Success => Ok(()),
            FunctionValidator::Error(ref err) => Err(err.clone().into()),
        }
    }
}
