use parity_wasm::elements::{deserialize_buffer, Internal, Module};
use std::collections::HashSet;

use crate::errors::{VmError, VmResult};

pub fn deserialize_wasm(wasm_code: &[u8]) -> VmResult<Module> {
    deserialize_buffer(&wasm_code).map_err(|err| {
        VmError::static_validation_err(format!(
            "Wasm bytecode could not be deserialized. Deserialization error: \"{}\"",
            err
        ))
    })
}

pub fn exported_functions(module: &Module) -> HashSet<String> {
    module
        .export_section()
        .map_or(HashSet::default(), |export_section| {
            export_section
                .entries()
                .iter()
                .filter_map(|entry| match entry.internal() {
                    Internal::Function(_) => Some(entry.field().to_string()),
                    _ => None,
                })
                .collect()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_wasm::elements::Internal;
    use std::iter::FromIterator;

    static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");
    static CORRUPTED: &[u8] = include_bytes!("../testdata/corrupted.wasm");

    #[test]
    fn deserialize_wasm_works() {
        let module = deserialize_wasm(CONTRACT).unwrap();
        assert_eq!(module.version(), 1);

        let exported_functions =
            module
                .export_section()
                .unwrap()
                .entries()
                .iter()
                .filter(|entry| {
                    if let Internal::Function(_) = entry.internal() {
                        true
                    } else {
                        false
                    }
                });
        assert_eq!(exported_functions.count(), 7); // 6 required export plus "migrate"

        let exported_memories = module
            .export_section()
            .unwrap()
            .entries()
            .iter()
            .filter(|entry| {
                if let Internal::Memory(_) = entry.internal() {
                    true
                } else {
                    false
                }
            });
        assert_eq!(exported_memories.count(), 1);
    }

    #[test]
    fn deserialize_wasm_corrupted_data() {
        match deserialize_wasm(CORRUPTED).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                assert!(msg.starts_with("Wasm bytecode could not be deserialized."))
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn exported_functions_works() {
        let wasm = wat::parse_str(r#"(module)"#).unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        let exports = exported_functions(&module);
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
        let module = deserialize_wasm(&wasm).unwrap();
        let exports = exported_functions(&module);
        assert_eq!(
            exports,
            HashSet::from_iter(vec!["foo".to_string(), "bar".to_string(),])
        );
    }
}
