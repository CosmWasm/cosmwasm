use dyn_partial_eq::DynPartialEq;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// An empty struct that serves as a placeholder in different places,
/// such as contracts that don't set a custom message.
///
/// It is designed to be expressible in correct JSON and JSON Schema but
/// contains no meaningful data. Previously we used enums without cases,
/// but those cannot represented as valid JSON Schema (https://github.com/CosmWasm/cosmwasm/issues/451)
#[derive(
    Serialize, Deserialize, Clone, Debug, DynPartialEq, PartialEq, Eq, JsonSchema, Default,
)]
pub struct Empty {}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{from_json, to_json_vec};

    #[test]
    fn empty_can_be_instantiated() {
        let instance = Empty::default();
        assert_eq!(instance, Empty {});
    }

    #[test]
    fn empty_can_be_instantiated_serialized_and_deserialized() {
        let instance = Empty {};
        let serialized = to_json_vec(&instance).unwrap();
        assert_eq!(serialized, b"{}");

        let deserialized: Empty = from_json(b"{}").unwrap();
        assert_eq!(deserialized, instance);

        let deserialized: Empty = from_json(b"{\"stray\":\"data\"}").unwrap();
        assert_eq!(deserialized, instance);
    }
}
