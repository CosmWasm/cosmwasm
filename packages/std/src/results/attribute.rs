use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// An key value pair that is used in the context of event attributes in logs
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct Attribute {
    pub key: String,
    pub value: String,
}

/// Creates a new Attribute.
pub fn attr<K: ToString, V: ToString>(key: K, value: V) -> Attribute {
    Attribute {
        key: key.to_string(),
        value: value.to_string(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::addresses::HumanAddr;
    use crate::Uint128;

    #[test]
    fn attr_works_for_different_types() {
        let expeceted = Attribute {
            key: "foo".to_string(),
            value: "42".to_string(),
        };

        assert_eq!(attr("foo", "42"), expeceted);
        assert_eq!(attr("foo".to_string(), "42"), expeceted);
        assert_eq!(attr("foo", "42".to_string()), expeceted);
        assert_eq!(attr("foo", HumanAddr::from("42")), expeceted);
        assert_eq!(attr("foo", Uint128(42)), expeceted);
        assert_eq!(attr("foo", 42), expeceted);
    }
}
