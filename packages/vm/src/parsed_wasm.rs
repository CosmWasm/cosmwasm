use wasmer::wasmparser::{
    CompositeType, Export, Import, MemoryType, Parser, Payload, TableType, ValidPayload, Validator,
    WasmFeatures,
};

use crate::{VmError, VmResult};

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
        };

        let mut fun_allocations = Default::default();
        for p in Parser::new(0).parse_all(wasm) {
            let p = p?;
            // validate the payload
            if let ValidPayload::Func(fv, body) = validator.payload(&p)? {
                // also validate function bodies
                let mut fun_validator = fv.into_validator(fun_allocations);
                fun_validator.validate(&body)?;
                fun_allocations = fun_validator.into_allocations();

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
}
