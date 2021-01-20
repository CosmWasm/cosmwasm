use parity_wasm::elements::{deserialize_buffer, Module};

use crate::errors::{VmError, VmResult};

pub fn deserialize_wasm(wasm_code: &[u8]) -> VmResult<Module> {
    deserialize_buffer(&wasm_code).map_err(|err| {
        VmError::static_validation_err(format!(
            "Wasm bytecode could not be deserialized. Deserialization error: \"{}\"",
            err
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_wasm::elements::Internal;

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
}
