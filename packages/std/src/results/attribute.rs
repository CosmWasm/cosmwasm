use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// An key value pair that is used in the context of event attributes in logs
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct Attribute {
    pub key: String,
    pub value: String,
}

impl Attribute {
    /// Creates a new Attribute. `attr` is just an alias for this.
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        let key = key.into();

        #[cfg(debug_assertions)]
        if key.starts_with('_') {
            panic!(
                "attribute `{}` is invalid - attributes starting with an underscore are reserved",
                key
            );
        }

        Self {
            key,
            value: value.into(),
        }
    }
}

/// Creates a new Attribute. `Attribute::new` is an alias for this.
#[inline]
pub fn attr(key: impl Into<String>, value: impl Into<String>) -> Attribute {
    Attribute::new(key, value)
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

    #[test]
    #[should_panic]
    fn reserved_attr_key_panicks() {
        Attribute::new("_invalid", "value");
    }

    #[test]
    #[should_panic]
    fn reserved_attr_key_panicks2() {
        Attribute::new("_", "value");
    }
}
