use parity_wasm::elements::{deserialize_buffer, External, ImportEntry, Module};
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::iter::FromIterator;

use crate::errors::{VmError, VmResult};
use crate::features::required_features_from_module;
use crate::limited::LimitedDisplay;

/// Lists all imports we provide upon instantiating the instance in Instance::from_module()
/// This should be updated when new imports are added
const SUPPORTED_IMPORTS: &[&str] = &[
    "env.db_read",
    "env.db_write",
    "env.db_remove",
    "env.canonicalize_address",
    "env.humanize_address",
    "env.debug",
    "env.query_chain",
    #[cfg(feature = "iterator")]
    "env.db_scan",
    #[cfg(feature = "iterator")]
    "env.db_next",
];

/// Lists all entry points we expect to be present when calling a contract.
/// Basically, anything that is used in calls.rs
/// This is unlikely to change much, must be frozen at 1.0 to avoid breaking existing contracts
const REQUIRED_EXPORTS: &[&str] = &[
    "cosmwasm_vm_version_4",
    "query",
    "init",
    "handle",
    "allocate",
    "deallocate",
];

const MEMORY_LIMIT: u32 = 512; // in pages

fn deserialize(wasm_code: &[u8]) -> VmResult<Module> {
    deserialize_buffer(&wasm_code).map_err(|err| {
        VmError::static_validation_err(format!(
            "Wasm bytecode could not be deserialized. Deserialization error: \"{}\"",
            err
        ))
    })
}

/// Checks if the data is valid wasm and compatibility with the CosmWasm API (imports and exports)
pub fn check_wasm(wasm_code: &[u8], supported_features: &HashSet<String>) -> VmResult<()> {
    let module = deserialize(wasm_code)?;
    check_wasm_memories(&module)?;
    check_wasm_exports(&module)?;
    check_wasm_imports(&module)?;
    check_wasm_features(&module, supported_features)?;
    Ok(())
}

fn check_wasm_memories(module: &Module) -> VmResult<()> {
    let section = match module.memory_section() {
        Some(section) => section,
        None => {
            return Err(VmError::static_validation_err(
                "Wasm contract doesn't have a memory section",
            ));
        }
    };

    let memories = section.entries();
    if memories.len() != 1 {
        return Err(VmError::static_validation_err(
            "Wasm contract must contain exactly one memory",
        ));
    }

    let memory = memories[0];
    // println!("Memory: {:?}", memory);
    let limits = memory.limits();

    if limits.initial() > MEMORY_LIMIT {
        return Err(VmError::static_validation_err(format!(
            "Wasm contract memory's minimum must not exceed {} pages.",
            MEMORY_LIMIT
        )));
    }

    if limits.maximum() != None {
        return Err(VmError::static_validation_err(
            "Wasm contract memory's maximum must be unset. The host will set it for you.",
        ));
    }
    Ok(())
}

fn check_wasm_exports(module: &Module) -> VmResult<()> {
    let available_exports: Vec<String> = module.export_section().map_or(vec![], |export_section| {
        export_section
            .entries()
            .iter()
            .map(|entry| entry.field().to_string())
            .collect()
    });

    for required_export in REQUIRED_EXPORTS {
        if !available_exports.iter().any(|x| x == required_export) {
            return Err(VmError::static_validation_err(format!(
                "Wasm contract doesn't have required export: \"{}\". Exports required by VM: {:?}. Contract version too old for this VM?",
                required_export, REQUIRED_EXPORTS
            )));
        }
    }
    Ok(())
}

/// Checks if the import requirements of the contract are satisfied.
/// When this is not the case, we either have an incompatibility between contract and VM
/// or a error in the contract.
fn check_wasm_imports(module: &Module) -> VmResult<()> {
    let required_imports: Vec<ImportEntry> = module
        .import_section()
        .map_or(vec![], |import_section| import_section.entries().to_vec());
    let required_import_names = BTreeSet::from_iter(required_imports.iter().map(full_import_name));

    for required_import in required_imports {
        let full_name = full_import_name(&required_import);
        if !SUPPORTED_IMPORTS.contains(&full_name.as_str()) {
            return Err(VmError::static_validation_err(format!(
                "Wasm contract requires unsupported import: \"{}\". Required imports: {}. Available imports: {:?}.",
                full_name, required_import_names.to_string_limited(200), SUPPORTED_IMPORTS
            )));
        }

        match required_import.external() {
            External::Function(_) => {}, // ok
            _ => return Err(VmError::static_validation_err(format!(
                "Wasm contract requires non-function import: \"{}\". Right now, all supported imports are functions.",
                full_name
            ))),
        };
    }
    Ok(())
}

fn full_import_name(ie: &ImportEntry) -> String {
    format!("{}.{}", ie.module(), ie.field())
}

fn check_wasm_features(module: &Module, supported_features: &HashSet<String>) -> VmResult<()> {
    let required_features = required_features_from_module(module);
    if !required_features.is_subset(supported_features) {
        // We switch to BTreeSet to get a sorted error message
        let unsupported = BTreeSet::from_iter(required_features.difference(&supported_features));
        return Err(VmError::static_validation_err(format!(
            "Wasm contract requires unsupported features: {}",
            unsupported.to_string_limited(200)
        )));
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::errors::VmError;
    use parity_wasm::elements::Internal;
    use std::iter::FromIterator;

    static CONTRACT_0_6: &[u8] = include_bytes!("../testdata/contract_0.6.wasm");
    static CONTRACT_0_7: &[u8] = include_bytes!("../testdata/contract_0.7.wasm");
    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");
    static CORRUPTED: &[u8] = include_bytes!("../testdata/corrupted.wasm");

    fn default_features() -> HashSet<String> {
        HashSet::from_iter(["staking".to_string()].iter().cloned())
    }

    #[test]
    fn test_deserialize_works() {
        let module = deserialize(CONTRACT).unwrap();
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
    fn test_deserialize_corrupted_data() {
        match deserialize(CORRUPTED).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                assert!(msg.starts_with("Wasm bytecode could not be deserialized."))
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn test_check_wasm() {
        // this is our reference check, must pass
        check_wasm(CONTRACT, &default_features()).unwrap();
    }

    #[test]
    fn test_check_wasm_old_contract() {
        match check_wasm(CONTRACT_0_7, &default_features()) {
            Err(VmError::StaticValidationErr { msg, .. }) => assert!(msg.starts_with(
                "Wasm contract doesn't have required export: \"cosmwasm_vm_version_4\""
            )),
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("This must not succeeed"),
        };

        match check_wasm(CONTRACT_0_6, &default_features()) {
            Err(VmError::StaticValidationErr { msg, .. }) => assert!(msg.starts_with(
                "Wasm contract doesn't have required export: \"cosmwasm_vm_version_4\""
            )),
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("This must not succeeed"),
        };
    }

    #[test]
    fn test_check_wasm_memories_ok() {
        let wasm = wat::parse_str("(module (memory 1))").unwrap();
        check_wasm_memories(&deserialize(&wasm).unwrap()).unwrap()
    }

    #[test]
    fn test_check_wasm_memories_no_memory() {
        let wasm = wat::parse_str("(module)").unwrap();
        match check_wasm_memories(&deserialize(&wasm).unwrap()) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm contract doesn't have a memory section"));
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn test_check_wasm_memories_two_memories() {
        // Generated manually because wat2wasm protects us from creating such Wasm:
        // "error: only one memory block allowed"
        let wasm = hex::decode(concat!(
            "0061736d", // magic bytes
            "01000000", // binary version (uint32)
            "05",       // section type (memory)
            "05",       // section length
            "02",       // number of memories
            "0009",     // element of type "resizable_limits", min=9, max=unset
            "0009",     // element of type "resizable_limits", min=9, max=unset
        ))
        .unwrap();

        match check_wasm_memories(&deserialize(&wasm).unwrap()) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm contract must contain exactly one memory"));
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn test_check_wasm_memories_zero_memories() {
        // Generated manually because wat2wasm would not create an empty memory section
        let wasm = hex::decode(concat!(
            "0061736d", // magic bytes
            "01000000", // binary version (uint32)
            "05",       // section type (memory)
            "01",       // section length
            "00",       // number of memories
        ))
        .unwrap();

        match check_wasm_memories(&deserialize(&wasm).unwrap()) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm contract must contain exactly one memory"));
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn test_check_wasm_memories_initial_size() {
        let wasm_ok = wat::parse_str("(module (memory 512))").unwrap();
        check_wasm_memories(&deserialize(&wasm_ok).unwrap()).unwrap();

        let wasm_too_big = wat::parse_str("(module (memory 513))").unwrap();
        match check_wasm_memories(&deserialize(&wasm_too_big).unwrap()) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm contract memory's minimum must not exceed 512 pages"));
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn test_check_wasm_memories_maximum_size() {
        let wasm_max = wat::parse_str("(module (memory 1 5))").unwrap();
        match check_wasm_memories(&deserialize(&wasm_max).unwrap()) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm contract memory's maximum must be unset"));
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn test_check_wasm_exports() {
        // this is invalid, as it doesn't contain all required exports
        const WAT_MISSING_EXPORTS: &'static str = r#"
            (module
              (type $t0 (func (param i32) (result i32)))
              (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
                get_local $p0
                i32.const 1
                i32.add))
        "#;
        let wasm_missing_exports = wat::parse_str(WAT_MISSING_EXPORTS).unwrap();

        let module = deserialize(&wasm_missing_exports).unwrap();
        match check_wasm_exports(&module) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with(
                    "Wasm contract doesn't have required export: \"cosmwasm_vm_version_4\""
                ));
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn test_check_wasm_exports_of_old_contract() {
        let module = deserialize(CONTRACT_0_7).unwrap();
        match check_wasm_exports(&module) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with(
                    "Wasm contract doesn't have required export: \"cosmwasm_vm_version_4\""
                ));
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn check_wasm_imports_ok() {
        let wasm = wat::parse_str(
            r#"(module
            (import "env" "db_read" (func (param i32 i32) (result i32)))
            (import "env" "db_write" (func (param i32 i32) (result i32)))
            (import "env" "db_remove" (func (param i32) (result i32)))
            (import "env" "canonicalize_address" (func (param i32 i32) (result i32)))
            (import "env" "humanize_address" (func (param i32 i32) (result i32)))
        )"#,
        )
        .unwrap();
        check_wasm_imports(&deserialize(&wasm).unwrap()).unwrap();
    }

    #[test]
    fn test_check_wasm_imports_missing() {
        let wasm = wat::parse_str(
            r#"(module
            (import "env" "foo" (func (param i32 i32) (result i32)))
            (import "env" "bar" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam01" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam02" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam03" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam04" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam05" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam06" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam07" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam08" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam09" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam10" (func (param i32 i32) (result i32)))
        )"#,
        )
        .unwrap();
        let result = check_wasm_imports(&deserialize(&wasm).unwrap());
        match result.unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                println!("{}", msg);
                assert_eq!(
                    msg,
                    r#"Wasm contract requires unsupported import: "env.foo". Required imports: {"env.bar", "env.foo", "env.spammyspam01", "env.spammyspam02", "env.spammyspam03", "env.spammyspam04", "env.spammyspam05", "env.spammyspam06", "env.spammyspam07", "env.spammyspam08", ... 2 more}. Available imports: ["env.db_read", "env.db_write", "env.db_remove", "env.canonicalize_address", "env.humanize_address", "env.debug", "env.query_chain", "env.db_scan", "env.db_next"]."#
                );
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn test_check_wasm_imports_of_old_contract() {
        let module = deserialize(CONTRACT_0_7).unwrap();
        let result = check_wasm_imports(&module);
        match result.unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                assert!(
                    msg.starts_with("Wasm contract requires unsupported import: \"env.read_db\"")
                );
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn test_check_wasm_imports_wrong_type() {
        let wasm = wat::parse_str(r#"(module (import "env" "db_read" (memory 1 1)))"#).unwrap();
        let result = check_wasm_imports(&deserialize(&wasm).unwrap());
        match result.unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                assert!(
                    msg.starts_with("Wasm contract requires non-function import: \"env.db_read\"")
                );
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn check_wasm_features_ok() {
        let wasm = wat::parse_str(
            r#"(module
            (type (func))
            (func (type 0) nop)
            (export "requires_water" (func 0))
            (export "requires_" (func 0))
            (export "requires_nutrients" (func 0))
            (export "require_milk" (func 0))
            (export "REQUIRES_air" (func 0))
            (export "requires_sun" (func 0))
        )"#,
        )
        .unwrap();
        let module = deserialize(&wasm).unwrap();
        let supported = HashSet::from_iter(
            [
                "water".to_string(),
                "nutrients".to_string(),
                "sun".to_string(),
                "freedom".to_string(),
            ]
            .iter()
            .cloned(),
        );
        check_wasm_features(&module, &supported).unwrap();
    }

    #[test]
    fn check_wasm_features_fails_for_missing() {
        let wasm = wat::parse_str(
            r#"(module
            (type (func))
            (func (type 0) nop)
            (export "requires_water" (func 0))
            (export "requires_" (func 0))
            (export "requires_nutrients" (func 0))
            (export "require_milk" (func 0))
            (export "REQUIRES_air" (func 0))
            (export "requires_sun" (func 0))
        )"#,
        )
        .unwrap();
        let module = deserialize(&wasm).unwrap();

        // Support set 1
        let supported = HashSet::from_iter(
            [
                "water".to_string(),
                "nutrients".to_string(),
                "freedom".to_string(),
            ]
            .iter()
            .cloned(),
        );
        match check_wasm_features(&module, &supported).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => assert_eq!(
                msg,
                "Wasm contract requires unsupported features: {\"sun\"}"
            ),
            _ => panic!("Got unexpected error"),
        }

        // Support set 2
        let supported = HashSet::from_iter(
            [
                "nutrients".to_string(),
                "freedom".to_string(),
                "Water".to_string(), // features are case sensitive (and lowercase by convention)
            ]
            .iter()
            .cloned(),
        );
        match check_wasm_features(&module, &supported).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => assert_eq!(
                msg,
                "Wasm contract requires unsupported features: {\"sun\", \"water\"}"
            ),
            _ => panic!("Got unexpected error"),
        }

        // Support set 3
        let supported = HashSet::from_iter(["freedom".to_string()].iter().cloned());
        match check_wasm_features(&module, &supported).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => assert_eq!(
                msg,
                "Wasm contract requires unsupported features: {\"nutrients\", \"sun\", \"water\"}"
            ),
            _ => panic!("Got unexpected error"),
        }

        // Support set 4
        let supported = HashSet::from_iter([].iter().cloned());
        match check_wasm_features(&module, &supported).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => assert_eq!(
                msg,
                "Wasm contract requires unsupported features: {\"nutrients\", \"sun\", \"water\"}"
            ),
            _ => panic!("Got unexpected error"),
        }
    }
}
