use wasmer::wasmparser::{
    Export, Import, MemoryType, Parser, Payload, TableType, Type, ValidPayload, Validator,
    WasmFeatures,
};

use crate::{VmError, VmResult};

/// A parsed and validated wasm module.
/// It keeps track of the parts that are important for our static analysis and compatibility checks.
#[derive(Debug)]
pub struct ParsedWasm<'a> {
    pub version: u32,
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
            deterministic_only: true,
            component_model: false,
            simd: false,
            relaxed_simd: false,
            threads: false,
            multi_memory: false,
            memory64: false,
            ..Default::default()
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
                    this.type_count = t.get_count();
                    this.type_params = Vec::with_capacity(t.get_count() as usize);
                    for t_res in t.into_iter() {
                        let ty: Type = t_res?;
                        match ty {
                            Type::Func(ft) => {
                                this.type_params.push(ft.params().len());

                                this.max_func_params =
                                    core::cmp::max(ft.params().len(), this.max_func_params);
                                this.max_func_results =
                                    core::cmp::max(ft.results().len(), this.max_func_results);
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
                    this.tables = t.into_iter().collect::<Result<Vec<_>, _>>()?;
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
