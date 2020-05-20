use parity_wasm::elements::{Internal, Module};
use std::collections::HashSet;
use std::iter::FromIterator;
use wasmer_runtime_core::{export::Export, instance::Instance};

static REQUIRES_PREFIX: &str = "requires_";

/// Takes a comma-separated string, splits it by commas, and returns a set of features.
/// This can be used e.g. to initialize the cache.
pub fn features_from_csv(csv: &str) -> HashSet<String> {
    HashSet::from_iter(csv.split(',').map(|x| x.trim().to_string()))
}

pub fn required_features_from_wasmer_instance(wasmer_instance: &Instance) -> HashSet<String> {
    HashSet::from_iter(wasmer_instance.exports().filter_map(|(mut name, export)| {
        if let Export::Function { .. } = export {
            if name.starts_with(REQUIRES_PREFIX) && name.len() > REQUIRES_PREFIX.len() {
                let required_feature = name.split_off(REQUIRES_PREFIX.len());
                return Some(required_feature);
            }
        }
        None
    }))
}

/// Implementation for check_wasm, based on static analysis of the bytecode.
/// This is used for code upload, to perform check before compiling the Wasm.
pub fn required_features_from_module(module: &Module) -> HashSet<String> {
    match module.export_section() {
        None => HashSet::new(),
        Some(export_section) => {
            HashSet::from_iter(export_section.entries().iter().filter_map(|entry| {
                if let Internal::Function(_) = entry.internal() {
                    let name = entry.field();
                    if name.starts_with(REQUIRES_PREFIX) && name.len() > REQUIRES_PREFIX.len() {
                        let (_, required_feature) = name.split_at(REQUIRES_PREFIX.len());
                        return Some(required_feature.to_string());
                    }
                }
                None
            }))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use parity_wasm::elements::deserialize_buffer;
    use wabt::wat2wasm;

    #[test]
    fn features_from_csv_works() {
        let set = features_from_csv("foo, bar,baz ");
        assert_eq!(set.len(), 3);
        assert!(set.contains("foo"));
        assert!(set.contains("bar"));
        assert!(set.contains("baz"));
    }

    #[test]
    fn required_features_from_module_works() {
        let wasm = wat2wasm(
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
        let module = deserialize_buffer(&wasm).unwrap();

        let required_features = required_features_from_module(&module);
        assert_eq!(required_features.len(), 3);
        assert!(required_features.contains("nutrients"));
        assert!(required_features.contains("sun"));
        assert!(required_features.contains("water"));
    }

    #[test]
    fn required_features_from_module_works_without_exports_section() {
        let wasm = wat2wasm(r#"(module)"#).unwrap();
        let module = deserialize_buffer(&wasm).unwrap();
        let required_features = required_features_from_module(&module);
        assert_eq!(required_features.len(), 0);
    }
}
