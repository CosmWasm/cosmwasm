use std::collections::HashSet;
use std::iter::FromIterator;
use wasmer_runtime_core::{export::Export, instance::Instance};

static REQUIRES_PREFIX: &str = "requires_";

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
