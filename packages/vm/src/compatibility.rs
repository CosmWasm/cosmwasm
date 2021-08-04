use parity_wasm::elements::{External, ImportEntry, Module};
use std::collections::BTreeSet;
use std::collections::HashSet;

use crate::errors::{VmError, VmResult};
use crate::features::required_features_from_module;
use crate::limited::LimitedDisplay;
use crate::static_analysis::{deserialize_wasm, ExportInfo};

/// Lists all imports we provide upon instantiating the instance in Instance::from_module()
/// This should be updated when new imports are added
const SUPPORTED_IMPORTS: &[&str] = &[
    "env.db_read",
    "env.db_write",
    "env.db_remove",
    "env.addr_validate",
    "env.addr_canonicalize",
    "env.addr_humanize",
    "env.secp256k1_verify",
    "env.secp256k1_recover_pubkey",
    "env.ed25519_verify",
    "env.ed25519_batch_verify",
    "env.debug",
    "env.query_chain",
    #[cfg(feature = "iterator")]
    "env.db_scan",
    #[cfg(feature = "iterator")]
    "env.db_next",
];

/// Lists all entry points we expect to be present when calling a contract.
/// Other optional exports exist, e.g. "execute", "migrate" and "query".
/// The marker export interface_version_* is checked separately.
/// This is unlikely to change much, must be frozen at 1.0 to avoid breaking existing contracts
const REQUIRED_EXPORTS: &[&str] = &[
    // IO
    "allocate",
    "deallocate",
    // Required entry points
    "instantiate",
];

const MEMORY_LIMIT: u32 = 512; // in pages

/// Checks if the data is valid wasm and compatibility with the CosmWasm API (imports and exports)
pub fn check_wasm(wasm_code: &[u8], supported_features: &HashSet<String>) -> VmResult<()> {
    let module = deserialize_wasm(wasm_code)?;
    check_wasm_memories(&module)?;
    check_interface_version(&module)?;
    check_wasm_exports(&module)?;
    check_wasm_imports(&module, SUPPORTED_IMPORTS)?;
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

fn check_interface_version(module: &Module) -> VmResult<()> {
    let mut interface_version_exports: Vec<String> = module
        .exported_function_names(Some("interface_version_"))
        .into_iter()
        .collect();
    if let Some(interface_version_export) = interface_version_exports.pop() {
        if !interface_version_exports.is_empty() {
            Err(VmError::static_validation_err(
                "Wasm contract contains more than one marker export: interface_version_*",
            ))
        } else {
            // Exactly one interface version found

            match interface_version_export.as_str() {
                // Ok
                "interface_version_7" => Ok(()),
                // Well known old versions for better error messages
                "interface_version_6" => Err(VmError::static_validation_err(
                    "Wasm contract has incompatible CosmWasm 0.15 marker export interface_version_6 (see https://github.com/CosmWasm/cosmwasm/blob/main/packages/vm/README.md)"
                )),
                "interface_version_5" => Err(VmError::static_validation_err(
                    "Wasm contract has incompatible CosmWasm 0.14 marker export interface_version_5 (see https://github.com/CosmWasm/cosmwasm/blob/main/packages/vm/README.md)"
                )),
                // Unknown version
                _ => Err(VmError::static_validation_err(
                    "Wasm contract has unknown interface_version_* marker export (see https://github.com/CosmWasm/cosmwasm/blob/main/packages/vm/README.md)",
                )),
            }
        }
    } else {
        Err(VmError::static_validation_err(
            "Wasm contract missing a required marker export: interface_version_*",
        ))
    }
}

fn check_wasm_exports(module: &Module) -> VmResult<()> {
    let available_exports: HashSet<String> = module.exported_function_names(None);
    for required_export in REQUIRED_EXPORTS {
        if !available_exports.contains(*required_export) {
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
fn check_wasm_imports(module: &Module, supported_imports: &[&str]) -> VmResult<()> {
    let required_imports: Vec<ImportEntry> = module
        .import_section()
        .map_or(vec![], |import_section| import_section.entries().to_vec());
    let required_import_names: BTreeSet<_> =
        required_imports.iter().map(full_import_name).collect();

    for required_import in required_imports {
        let full_name = full_import_name(&required_import);
        if !supported_imports.contains(&full_name.as_str()) {
            return Err(VmError::static_validation_err(format!(
                "Wasm contract requires unsupported import: \"{}\". Required imports: {}. Available imports: {:?}.",
                full_name, required_import_names.to_string_limited(200), supported_imports
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
        let unsupported: BTreeSet<_> = required_features.difference(&supported_features).collect();
        return Err(VmError::static_validation_err(format!(
            "Wasm contract requires unsupported features: {}",
            unsupported.to_string_limited(200)
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::VmError;

    static CONTRACT_0_6: &[u8] = include_bytes!("../testdata/hackatom_0.6.wasm");
    static CONTRACT_0_7: &[u8] = include_bytes!("../testdata/hackatom_0.7.wasm");
    static CONTRACT_0_12: &[u8] = include_bytes!("../testdata/hackatom_0.12.wasm");
    static CONTRACT_0_14: &[u8] = include_bytes!("../testdata/hackatom_0.14.wasm");
    static CONTRACT_0_15: &[u8] = include_bytes!("../testdata/hackatom_0.15.wasm");
    static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");

    fn default_features() -> HashSet<String> {
        ["staking".to_string()].iter().cloned().collect()
    }

    #[test]
    fn check_wasm_passes_for_latest_contract() {
        // this is our reference check, must pass
        check_wasm(CONTRACT, &default_features()).unwrap();
    }

    #[test]
    fn check_wasm_old_contract() {
        match check_wasm(CONTRACT_0_15, &default_features()) {
            Err(VmError::StaticValidationErr { msg, .. }) => assert_eq!(
                msg,
                "Wasm contract has incompatible CosmWasm 0.15 marker export interface_version_6 (see https://github.com/CosmWasm/cosmwasm/blob/main/packages/vm/README.md)"
            ),
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("This must not succeeed"),
        };

        match check_wasm(CONTRACT_0_14, &default_features()) {
            Err(VmError::StaticValidationErr { msg, .. }) => assert_eq!(
                msg,
                "Wasm contract has incompatible CosmWasm 0.14 marker export interface_version_5 (see https://github.com/CosmWasm/cosmwasm/blob/main/packages/vm/README.md)"
            ),
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("This must not succeeed"),
        };

        match check_wasm(CONTRACT_0_12, &default_features()) {
            Err(VmError::StaticValidationErr { msg, .. }) => assert_eq!(
                msg,
                "Wasm contract missing a required marker export: interface_version_*"
            ),
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("This must not succeeed"),
        };

        match check_wasm(CONTRACT_0_7, &default_features()) {
            Err(VmError::StaticValidationErr { msg, .. }) => assert_eq!(
                msg,
                "Wasm contract missing a required marker export: interface_version_*"
            ),
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("This must not succeeed"),
        };

        match check_wasm(CONTRACT_0_6, &default_features()) {
            Err(VmError::StaticValidationErr { msg, .. }) => assert_eq!(
                msg,
                "Wasm contract missing a required marker export: interface_version_*"
            ),
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("This must not succeeed"),
        };
    }

    #[test]
    fn check_wasm_memories_ok() {
        let wasm = wat::parse_str("(module (memory 1))").unwrap();
        check_wasm_memories(&deserialize_wasm(&wasm).unwrap()).unwrap()
    }

    #[test]
    fn check_wasm_memories_no_memory() {
        let wasm = wat::parse_str("(module)").unwrap();
        match check_wasm_memories(&deserialize_wasm(&wasm).unwrap()) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm contract doesn't have a memory section"));
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn check_wasm_memories_two_memories() {
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

        match check_wasm_memories(&deserialize_wasm(&wasm).unwrap()) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm contract must contain exactly one memory"));
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn check_wasm_memories_zero_memories() {
        // Generated manually because wat2wasm would not create an empty memory section
        let wasm = hex::decode(concat!(
            "0061736d", // magic bytes
            "01000000", // binary version (uint32)
            "05",       // section type (memory)
            "01",       // section length
            "00",       // number of memories
        ))
        .unwrap();

        match check_wasm_memories(&deserialize_wasm(&wasm).unwrap()) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm contract must contain exactly one memory"));
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn check_wasm_memories_initial_size() {
        let wasm_ok = wat::parse_str("(module (memory 512))").unwrap();
        check_wasm_memories(&deserialize_wasm(&wasm_ok).unwrap()).unwrap();

        let wasm_too_big = wat::parse_str("(module (memory 513))").unwrap();
        match check_wasm_memories(&deserialize_wasm(&wasm_too_big).unwrap()) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm contract memory's minimum must not exceed 512 pages"));
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn check_wasm_memories_maximum_size() {
        let wasm_max = wat::parse_str("(module (memory 1 5))").unwrap();
        match check_wasm_memories(&deserialize_wasm(&wasm_max).unwrap()) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm contract memory's maximum must be unset"));
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn check_wasm_exports_works() {
        // valid
        let wasm = wat::parse_str(
            r#"(module
                (type (func))
                (func (type 0) nop)
                (export "add_one" (func 0))
                (export "allocate" (func 0))
                (export "deallocate" (func 0))
                (export "instantiate" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        check_wasm_exports(&module).unwrap();

        // this is invalid, as it doesn't any required export
        let wasm = wat::parse_str(
            r#"(module
                (type (func))
                (func (type 0) nop)
                (export "add_one" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        match check_wasm_exports(&module) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm contract doesn't have required export: \"allocate\""));
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }

        // this is invalid, as it doesn't contain all required exports
        let wasm = wat::parse_str(
            r#"(module
                (type (func))
                (func (type 0) nop)
                (export "add_one" (func 0))
                (export "allocate" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        match check_wasm_exports(&module) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(
                    msg.starts_with("Wasm contract doesn't have required export: \"deallocate\"")
                );
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn check_wasm_exports_of_old_contract() {
        let module = deserialize_wasm(CONTRACT_0_7).unwrap();
        match check_wasm_exports(&module) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(
                    msg.starts_with("Wasm contract doesn't have required export: \"instantiate\"")
                )
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
            (import "env" "addr_validate" (func (param i32) (result i32)))
            (import "env" "addr_canonicalize" (func (param i32 i32) (result i32)))
            (import "env" "addr_humanize" (func (param i32 i32) (result i32)))
            (import "env" "secp256k1_verify" (func (param i32 i32 i32) (result i32)))
            (import "env" "secp256k1_recover_pubkey" (func (param i32 i32 i32) (result i64)))
            (import "env" "ed25519_verify" (func (param i32 i32 i32) (result i32)))
            (import "env" "ed25519_batch_verify" (func (param i32 i32 i32) (result i32)))
        )"#,
        )
        .unwrap();
        check_wasm_imports(&deserialize_wasm(&wasm).unwrap(), SUPPORTED_IMPORTS).unwrap();
    }

    #[test]
    fn check_wasm_imports_missing() {
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
        let supported_imports: &[&str] = &[
            "env.db_read",
            "env.db_write",
            "env.db_remove",
            "env.addr_canonicalize",
            "env.addr_humanize",
            "env.debug",
            "env.query_chain",
        ];
        let result = check_wasm_imports(&deserialize_wasm(&wasm).unwrap(), supported_imports);
        match result.unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                println!("{}", msg);
                assert_eq!(
                    msg,
                    r#"Wasm contract requires unsupported import: "env.foo". Required imports: {"env.bar", "env.foo", "env.spammyspam01", "env.spammyspam02", "env.spammyspam03", "env.spammyspam04", "env.spammyspam05", "env.spammyspam06", "env.spammyspam07", "env.spammyspam08", ... 2 more}. Available imports: ["env.db_read", "env.db_write", "env.db_remove", "env.addr_canonicalize", "env.addr_humanize", "env.debug", "env.query_chain"]."#
                );
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn check_wasm_imports_of_old_contract() {
        let module = deserialize_wasm(CONTRACT_0_7).unwrap();
        let result = check_wasm_imports(&module, SUPPORTED_IMPORTS);
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
    fn check_wasm_imports_wrong_type() {
        let wasm = wat::parse_str(r#"(module (import "env" "db_read" (memory 1 1)))"#).unwrap();
        let result = check_wasm_imports(&deserialize_wasm(&wasm).unwrap(), SUPPORTED_IMPORTS);
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
        let module = deserialize_wasm(&wasm).unwrap();
        let supported = [
            "water".to_string(),
            "nutrients".to_string(),
            "sun".to_string(),
            "freedom".to_string(),
        ]
        .iter()
        .cloned()
        .collect();
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
        let module = deserialize_wasm(&wasm).unwrap();

        // Support set 1
        let supported = [
            "water".to_string(),
            "nutrients".to_string(),
            "freedom".to_string(),
        ]
        .iter()
        .cloned()
        .collect();
        match check_wasm_features(&module, &supported).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => assert_eq!(
                msg,
                "Wasm contract requires unsupported features: {\"sun\"}"
            ),
            _ => panic!("Got unexpected error"),
        }

        // Support set 2
        let supported = [
            "nutrients".to_string(),
            "freedom".to_string(),
            "Water".to_string(), // features are case sensitive (and lowercase by convention)
        ]
        .iter()
        .cloned()
        .collect();
        match check_wasm_features(&module, &supported).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => assert_eq!(
                msg,
                "Wasm contract requires unsupported features: {\"sun\", \"water\"}"
            ),
            _ => panic!("Got unexpected error"),
        }

        // Support set 3
        let supported = ["freedom".to_string()].iter().cloned().collect();
        match check_wasm_features(&module, &supported).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => assert_eq!(
                msg,
                "Wasm contract requires unsupported features: {\"nutrients\", \"sun\", \"water\"}"
            ),
            _ => panic!("Got unexpected error"),
        }

        // Support set 4
        let supported = [].iter().cloned().collect();
        match check_wasm_features(&module, &supported).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => assert_eq!(
                msg,
                "Wasm contract requires unsupported features: {\"nutrients\", \"sun\", \"water\"}"
            ),
            _ => panic!("Got unexpected error"),
        }
    }
}
