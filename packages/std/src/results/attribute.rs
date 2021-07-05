use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// An key value pair that is used in the context of event attributes in logs
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct Attribute {
    pub key: String,
    pub value: String,
}

/// Creates a new Attribute.
pub fn attr(key: impl Into<String>, value: impl Into<String>) -> Attribute {
    Attribute {
        key: key.into(),
        value: value.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Uint128;

    #[test]
    fn attr_works_for_different_types() {
        let expected = Attribute {
            key: "foo".to_string(),
            value: "42".to_string(),
        };

        assert_eq!(attr("foo", "42"), expected);
        assert_eq!(attr("foo".to_string(), "42"), expected);
        assert_eq!(attr("foo", "42".to_string()), expected);
        assert_eq!(attr("foo", Uint128::new(42)), expected);
        assert_eq!(attr("foo", 42.to_string()), expected);
    }
}
