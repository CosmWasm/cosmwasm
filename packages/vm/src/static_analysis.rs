use std::collections::HashSet;

use wasmer::wasmparser::{Export, ExportSectionReader, ExternalKind};

use crate::errors::VmResult;

pub const REQUIRED_IBC_EXPORTS: &[&str] = &[
    "ibc_channel_open",
    "ibc_channel_connect",
    "ibc_channel_close",
    "ibc_packet_receive",
    "ibc_packet_ack",
    "ibc_packet_timeout",
];

/// A small helper macro to validate the wasm module and extract a reader for a specific section.
macro_rules! extract_reader {
    ($wasm_code: expr, $payload: ident, $t: ty) => {{
        fn extract(wasm_code: &[u8]) -> crate::VmResult<Option<$t>> {
            use wasmer::wasmparser::{Parser, Payload, ValidPayload, Validator};

            let mut validator = Validator::new();
            let parser = Parser::new(0);

            let mut value = None;
            for p in parser.parse_all(wasm_code) {
                let p = p?;
                // validate the payload
                if let ValidPayload::Func(mut fv, body) = validator.payload(&p)? {
                    // also validate function bodies
                    fv.validate(&body)?;
                }
                if let Payload::$payload(e) = p {
                    // do not return immediately, as we want to validate the entire module
                    value = Some(e);
                }
            }

            Ok(value)
        }

        extract($wasm_code)
    }};
}

pub(crate) use extract_reader;

pub fn deserialize_exports(wasm_code: &[u8]) -> VmResult<Vec<Export<'_>>> {
    let exports = extract_reader!(wasm_code, ExportSection, ExportSectionReader<'_>)?;
    Ok(exports
        .map(|e| e.into_iter().collect::<Result<Vec<_>, _>>())
        .transpose()?
        .unwrap_or_default())
}

/// A trait that allows accessing shared functionality of `parity_wasm::elements::Module`
/// and `wasmer::Module` in a shared fashion.
pub trait ExportInfo {
    /// Returns all exported function names with the given prefix
    fn exported_function_names(self, prefix: Option<&str>) -> HashSet<String>;
}

impl ExportInfo for &[Export<'_>] {
    fn exported_function_names(self, prefix: Option<&str>) -> HashSet<String> {
        self.iter()
            .filter_map(|export| match export.kind {
                ExternalKind::Func => Some(export.name),
                _ => None,
            })
            .filter(|name| {
                if let Some(required_prefix) = prefix {
                    name.starts_with(required_prefix)
                } else {
                    true
                }
            })
            .map(|name| name.to_string())
            .collect()
    }
}

impl ExportInfo for &Vec<Export<'_>> {
    fn exported_function_names(self, prefix: Option<&str>) -> HashSet<String> {
        self[..].exported_function_names(prefix)
    }
}

impl ExportInfo for &wasmer::Module {
    fn exported_function_names(self, prefix: Option<&str>) -> HashSet<String> {
        self.exports()
            .functions()
            .filter_map(|function_export| {
                let name = function_export.name();
                if let Some(required_prefix) = prefix {
                    if name.starts_with(required_prefix) {
                        Some(name.to_string())
                    } else {
                        None
                    }
                } else {
                    Some(name.to_string())
                }
            })
            .collect()
    }
}

/// Returns true if and only if all IBC entry points ([`REQUIRED_IBC_EXPORTS`])
/// exist as exported functions. This does not guarantee the entry points
/// are functional and for simplicity does not even check their signatures.
pub fn has_ibc_entry_points(module: impl ExportInfo) -> bool {
    let available_exports = module.exported_function_names(None);
    REQUIRED_IBC_EXPORTS
        .iter()
        .all(|required| available_exports.contains(*required))
}

#[cfg(test)]
mod tests {
    use crate::VmError;

    use super::*;
    use wasmer::{Cranelift, Store};

    static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");
    static CORRUPTED: &[u8] = include_bytes!("../testdata/corrupted.wasm");

    #[test]
    fn deserialize_exports_works() {
        let module = deserialize_exports(CONTRACT).unwrap();
        // assert_eq!(module.version(), 1); // TODO: not implemented anymore

        let exported_functions = module
            .iter()
            .filter(|entry| matches!(entry.kind, ExternalKind::Func));
        assert_eq!(exported_functions.count(), 8); // 4 required exports plus "execute", "migrate", "query" and "sudo"

        let exported_memories = module
            .iter()
            .filter(|entry| matches!(entry.kind, ExternalKind::Memory));
        assert_eq!(exported_memories.count(), 1);
    }

    #[test]
    fn deserialize_wasm_corrupted_data() {
        match deserialize_exports(CORRUPTED).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                assert!(msg.starts_with("Wasm bytecode could not be deserialized."))
            }
            err => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn exported_function_names_works_for_parity_with_no_prefix() {
        let wasm = wat::parse_str(r#"(module)"#).unwrap();
        let module = deserialize_exports(&wasm).unwrap();
        let exports = module.exported_function_names(None);
        assert_eq!(exports, HashSet::new());

        let wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "foo" (func 0))
                (export "bar" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_exports(&wasm).unwrap();
        let exports = module.exported_function_names(None);
        assert_eq!(
            exports,
            HashSet::from_iter(vec!["foo".to_string(), "bar".to_string()])
        );
    }

    #[test]
    fn exported_function_names_works_for_parity_with_prefix() {
        let wasm = wat::parse_str(r#"(module)"#).unwrap();
        let module = deserialize_exports(&wasm).unwrap();
        let exports = module.exported_function_names(Some("b"));
        assert_eq!(exports, HashSet::new());

        let wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "foo" (func 0))
                (export "bar" (func 0))
                (export "baz" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_exports(&wasm).unwrap();
        let exports = module.exported_function_names(Some("b"));
        assert_eq!(
            exports,
            HashSet::from_iter(vec!["bar".to_string(), "baz".to_string()])
        );
    }

    #[test]
    fn exported_function_names_works_for_wasmer_with_no_prefix() {
        let wasm = wat::parse_str(r#"(module)"#).unwrap();
        let compiler = Cranelift::default();
        let store = Store::new(compiler);
        let module = wasmer::Module::new(&store, wasm).unwrap();
        let exports = module.exported_function_names(None);
        assert_eq!(exports, HashSet::new());

        let wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "foo" (func 0))
                (export "bar" (func 0))
            )"#,
        )
        .unwrap();
        let compiler = Cranelift::default();
        let store = Store::new(compiler);
        let module = wasmer::Module::new(&store, wasm).unwrap();
        let exports = module.exported_function_names(None);
        assert_eq!(
            exports,
            HashSet::from_iter(vec!["foo".to_string(), "bar".to_string()])
        );
    }

    #[test]
    fn exported_function_names_works_for_wasmer_with_prefix() {
        let wasm = wat::parse_str(r#"(module)"#).unwrap();
        let compiler = Cranelift::default();
        let store = Store::new(compiler);
        let module = wasmer::Module::new(&store, wasm).unwrap();
        let exports = module.exported_function_names(Some("b"));
        assert_eq!(exports, HashSet::new());

        let wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "foo" (func 0))
                (export "bar" (func 0))
                (export "baz" (func 0))
            )"#,
        )
        .unwrap();
        let compiler = Cranelift::default();
        let store = Store::new(compiler);
        let module = wasmer::Module::new(&store, wasm).unwrap();
        let exports = module.exported_function_names(Some("b"));
        assert_eq!(
            exports,
            HashSet::from_iter(vec!["bar".to_string(), "baz".to_string()])
        );
    }

    #[test]
    fn has_ibc_entry_points_works() {
        // Non-IBC contract
        let wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "interface_version_8" (func 0))
                (export "instantiate" (func 0))
                (export "allocate" (func 0))
                (export "deallocate" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_exports(&wasm).unwrap();
        assert!(!has_ibc_entry_points(&module));

        // IBC contract
        let wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "interface_version_8" (func 0))
                (export "instantiate" (func 0))
                (export "execute" (func 0))
                (export "allocate" (func 0))
                (export "deallocate" (func 0))
                (export "ibc_channel_open" (func 0))
                (export "ibc_channel_connect" (func 0))
                (export "ibc_channel_close" (func 0))
                (export "ibc_packet_receive" (func 0))
                (export "ibc_packet_ack" (func 0))
                (export "ibc_packet_timeout" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_exports(&wasm).unwrap();
        assert!(has_ibc_entry_points(&module));

        // Missing packet ack
        let wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "interface_version_8" (func 0))
                (export "instantiate" (func 0))
                (export "execute" (func 0))
                (export "allocate" (func 0))
                (export "deallocate" (func 0))
                (export "ibc_channel_open" (func 0))
                (export "ibc_channel_connect" (func 0))
                (export "ibc_channel_close" (func 0))
                (export "ibc_packet_receive" (func 0))
                (export "ibc_packet_timeout" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_exports(&wasm).unwrap();
        assert!(!has_ibc_entry_points(&module));
    }
}
