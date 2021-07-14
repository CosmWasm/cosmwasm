use parity_wasm::elements::{deserialize_buffer, Internal, Module};
use std::collections::HashSet;

use crate::errors::{VmError, VmResult};

pub const REQUIRED_IBC_EXPORTS: &[&str] = &[
    "ibc_channel_open",
    "ibc_channel_connect",
    "ibc_channel_close",
    "ibc_packet_receive",
    "ibc_packet_ack",
    "ibc_packet_timeout",
];

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

/// Returns true if and only if all IBC entry points ([`REQUIRED_IBC_EXPORTS`])
/// exist as exported functions. This does not guarantee the entry points
/// are functional and for simplicity does not even check their signatures.
pub fn has_ibc_entry_points(module: &Module) -> bool {
    let available_exports = exported_functions(module);
    REQUIRED_IBC_EXPORTS
        .iter()
        .all(|required| available_exports.contains(*required))
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

        let exported_functions = module
            .export_section()
            .unwrap()
            .entries()
            .iter()
            .filter(|entry| matches!(entry.internal(), Internal::Function(_)));
        assert_eq!(exported_functions.count(), 8); // 4 required exports plus "execute", "migrate", "query" and "sudo"

        let exported_memories = module
            .export_section()
            .unwrap()
            .entries()
            .iter()
            .filter(|entry| matches!(entry.internal(), Internal::Memory(_)));
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

    #[test]
    fn has_ibc_entry_points_works() {
        // Non-IBC contract
        let wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "interface_version_7" (func 0))
                (export "instantiate" (func 0))
                (export "allocate" (func 0))
                (export "deallocate" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        assert_eq!(has_ibc_entry_points(&module), false);

        // IBC contract
        let wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "interface_version_7" (func 0))
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
        let module = deserialize_wasm(&wasm).unwrap();
        assert_eq!(has_ibc_entry_points(&module), true);

        // Missing packet ack
        let wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "interface_version_7" (func 0))
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
        let module = deserialize_wasm(&wasm).unwrap();
        assert_eq!(has_ibc_entry_points(&module), false);
    }
}
