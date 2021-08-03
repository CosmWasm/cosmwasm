use std::collections::HashSet;
use wasmer::Instance as WasmerInstance;

use crate::static_analysis::ExportInfo;

const REQUIRES_PREFIX: &str = "requires_";

/// Takes a comma-separated string, splits it by commas, removes empty elements and returns a set of features.
/// This can be used e.g. to initialize the cache.
pub fn features_from_csv(csv: &str) -> HashSet<String> {
    csv.split(',')
        .map(|x| x.trim().to_string())
        .filter(|f| !f.is_empty())
        .collect()
}

pub fn required_features_from_wasmer_instance(wasmer_instance: &WasmerInstance) -> HashSet<String> {
    let module = wasmer_instance.module();
    required_features_from_module(module)
}

/// Implementation for check_wasm, based on static analysis of the bytecode.
/// This is used for code upload, to perform check before compiling the Wasm.
pub fn required_features_from_module(module: &impl ExportInfo) -> HashSet<String> {
    module
        .exported_function_names(Some(REQUIRES_PREFIX))
        .into_iter()
        .filter_map(|name| {
            if name.len() > REQUIRES_PREFIX.len() {
                let (_, required_feature) = name.split_at(REQUIRES_PREFIX.len());
                Some(required_feature.to_string())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::static_analysis::deserialize_wasm;

    #[test]
    fn features_from_csv_works() {
        let set = features_from_csv("foo, bar,baz ");
        assert_eq!(set.len(), 3);
        assert!(set.contains("foo"));
        assert!(set.contains("bar"));
        assert!(set.contains("baz"));
    }

    #[test]
    fn features_from_csv_skips_empty() {
        let set = features_from_csv("");
        assert_eq!(set.len(), 0);
        let set = features_from_csv("a,,b");
        assert_eq!(set.len(), 2);
        assert!(set.contains("a"));
        assert!(set.contains("b"));
        let set = features_from_csv("a,b,");
        assert_eq!(set.len(), 2);
        assert!(set.contains("a"));
        assert!(set.contains("b"));
    }

    #[test]
    fn required_features_from_module_works() {
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

        let required_features = required_features_from_module(&module);
        assert_eq!(required_features.len(), 3);
        assert!(required_features.contains("nutrients"));
        assert!(required_features.contains("sun"));
        assert!(required_features.contains("water"));
    }

    #[test]
    fn required_features_from_module_works_without_exports_section() {
        let wasm = wat::parse_str(r#"(module)"#).unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        let required_features = required_features_from_module(&module);
        assert_eq!(required_features.len(), 0);
    }
}
