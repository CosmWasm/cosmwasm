use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

/// An empty struct that serves as a placeholder in different places,
/// such as contracts that don't set a custom message.
///
/// It is designed to be expressable in correct JSON and JSON Schema but
/// contains no meaningful data. Previously we used enums without cases,
/// but those cannot represented as valid JSON Schema (https://github.com/CosmWasm/cosmwasm/issues/451)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Empty {}

/// Some uses require Display on contract input, error. Allow use of Empty there.
impl fmt::Display for Empty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("{}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::serde::{from_slice, to_vec};

    #[test]
    fn empty_can_be_instantiated_serialized_and_deserialized() {
        let instance = Empty {};
        let serialized = to_vec(&instance).unwrap();
        assert_eq!(serialized, b"{}");

        let deserialized: Empty = from_slice(b"{}").unwrap();
        assert_eq!(deserialized, instance);

        let deserialized: Empty = from_slice(b"{\"stray\":\"data\"}").unwrap();
        assert_eq!(deserialized, instance);
    }
}
