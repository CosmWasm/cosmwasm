use parity_wasm::elements::{deserialize_buffer, Module};

use crate::errors::{make_validation_err, Result};

/// Lists all imports we provide upon instantiating the instance in Instance::from_module()
/// This should be updated when new imports are added
static SUPPORTED_IMPORTS: &[&str] = &[
    "env.db_read",
    "env.db_write",
    "env.db_remove",
    "env.canonicalize_address",
    "env.humanize_address",
    "env.query_chain",
    #[cfg(feature = "iterator")]
    "env.db_scan",
    #[cfg(feature = "iterator")]
    "env.db_next",
];

/// Lists all entry points we expect to be present when calling a contract.
/// Basically, anything that is used in calls.rs
/// This is unlikely to change much, must be frozen at 1.0 to avoid breaking existing contracts
static REQUIRED_EXPORTS: &[&str] = &[
    "cosmwasm_vm_version_1",
    "query",
    "init",
    "handle",
    "allocate",
    "deallocate",
];

/// Checks if the data is valid wasm and compatibility with the CosmWasm API (imports and exports)
pub fn check_wasm(wasm_code: &[u8]) -> Result<()> {
    let module = match deserialize_buffer(&wasm_code) {
        Ok(deserialized) => deserialized,
        Err(err) => {
            return make_validation_err(format!(
                "Wasm bytecode could not be deserialized. Deserialization error: \"{}\"",
                err
            ));
        }
    };
    check_wasm_exports(&module)?;
    check_wasm_imports(&module)?;
    Ok(())
}

fn check_wasm_exports(module: &Module) -> Result<()> {
    let available_exports: Vec<String> = module.export_section().map_or(vec![], |export_section| {
        export_section
            .entries()
            .iter()
            .map(|entry| entry.field().to_string())
            .collect()
    });

    for required_export in REQUIRED_EXPORTS {
        if !available_exports.iter().any(|x| x == required_export) {
            return make_validation_err(format!(
                "Wasm contract doesn't have required export: \"{}\". Exports required by VM: {:?}. Contract version too old for this VM?",
                required_export, REQUIRED_EXPORTS
            ));
        }
    }
    Ok(())
}

/// Checks if the import requirements of the contract are satisfied.
/// When this is not the case, we either have an incompatibility between contract and VM
/// or a error in the contract.
fn check_wasm_imports(module: &Module) -> Result<()> {
    let required_imports: Vec<String> = module.import_section().map_or(vec![], |import_section| {
        import_section
            .entries()
            .iter()
            .map(|entry| format!("{}.{}", entry.module(), entry.field()))
            .collect()
    });

    for required_import in required_imports {
        if !SUPPORTED_IMPORTS.contains(&required_import.as_str()) {
            return make_validation_err(format!(
                "Wasm contract requires unsupported import: \"{}\". Imports supported by VM: {:?}. Contract version too new for this VM?",
                required_import, SUPPORTED_IMPORTS
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::errors::Error;
    use wabt::wat2wasm;

    static CONTRACT_0_6: &[u8] = include_bytes!("../testdata/contract_0.6.wasm");
    static CONTRACT_0_7: &[u8] = include_bytes!("../testdata/contract_0.7.wasm");
    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");
    static CORRUPTED: &[u8] = include_bytes!("../testdata/corrupted.wasm");

    #[test]
    fn test_check_wasm() {
        // this is our reference check, must pass
        check_wasm(CONTRACT).unwrap();
    }

    #[test]
    fn test_check_wasm_old_contract() {
        match check_wasm(CONTRACT_0_7) {
            Err(Error::ValidationErr { msg, .. }) => assert!(msg.starts_with(
                "Wasm contract doesn't have required export: \"cosmwasm_vm_version_1\""
            )),
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("This must not succeeed"),
        };

        match check_wasm(CONTRACT_0_6) {
            Err(Error::ValidationErr { msg, .. }) => assert!(msg.starts_with(
                "Wasm contract doesn't have required export: \"cosmwasm_vm_version_1\""
            )),
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("This must not succeeed"),
        };
    }

    #[test]
    fn test_check_wasm_corrupted_data() {
        match check_wasm(CORRUPTED) {
            Err(Error::ValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm bytecode could not be deserialized."))
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("This must not succeeed"),
        }
    }

    #[test]
    fn test_check_wasm_exports() {
        // this is invalid, as it doesn't contain all required exports
        static WAT_MISSING_EXPORTS: &'static str = r#"
            (module
              (type $t0 (func (param i32) (result i32)))
              (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
                get_local $p0
                i32.const 1
                i32.add))
        "#;
        let wasm_missing_exports = wat2wasm(WAT_MISSING_EXPORTS).unwrap();

        let module = deserialize_buffer(&wasm_missing_exports).unwrap();
        match check_wasm_exports(&module) {
            Err(Error::ValidationErr { msg, .. }) => {
                assert!(msg.starts_with(
                    "Wasm contract doesn't have required export: \"cosmwasm_vm_version_1\""
                ));
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn test_check_wasm_exports_of_old_contract() {
        let module = deserialize_buffer(CONTRACT_0_7).unwrap();
        match check_wasm_exports(&module) {
            Err(Error::ValidationErr { msg, .. }) => {
                assert!(msg.starts_with(
                    "Wasm contract doesn't have required export: \"cosmwasm_vm_version_1\""
                ));
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn check_wasm_imports_ok() {
        let wasm = wat2wasm(
            r#"(module
            (import "env" "db_read" (func (param i32 i32) (result i32)))
            (import "env" "db_write" (func (param i32 i32) (result i32)))
            (import "env" "db_remove" (func (param i32) (result i32)))
            (import "env" "canonicalize_address" (func (param i32 i32) (result i32)))
            (import "env" "humanize_address" (func (param i32 i32) (result i32)))
        )"#,
        )
        .unwrap();
        check_wasm_imports(&deserialize_buffer(&wasm).unwrap()).unwrap();
    }

    #[test]
    fn test_check_wasm_imports_of_old_contract() {
        let module = deserialize_buffer(CONTRACT_0_7).unwrap();
        match check_wasm_imports(&module) {
            Err(Error::ValidationErr { msg, .. }) => {
                assert!(
                    msg.starts_with("Wasm contract requires unsupported import: \"env.read_db\"")
                );
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }
}
