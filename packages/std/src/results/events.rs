use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A full Cosmos SDK event as documented in
/// https://docs.cosmos.network/v0.42/core/events.html.
///
/// This version uses string attributes (similar to
/// https://github.com/cosmos/cosmos-sdk/blob/v0.42.5/proto/cosmos/base/abci/v1beta1/abci.proto#L56-L70),
/// which then get magically converted to bytes for Tendermint somewhere between
/// the Rust-Go interface, JSON deserialization and the `NewEvent` call in Cosmos SDK.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Event {
    /// The event type. This is renamed to "ty" because "type" is reserved in Rust. This sucks, we know.
    #[serde(rename = "type")]
    pub ty: String,
    pub attributes: Vec<Attribute>,
}

impl Event {
    /// Create a new event with the given type and an empty list of attributes.
    pub fn new(ty: impl Into<String>) -> Self {
        Event {
            ty: ty.into(),
            attributes: Vec::with_capacity(10),
        }
    }

    /// Add an attribute to the event.
    pub fn attr(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push(Attribute {
            key: key.into(),
            value: value.into(),
        });
        self
    }
}

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
                "attribute key `{}` is invalid - keys starting with an underscore are reserved",
                key
            );
        }

        Self {
            key,
            value: value.into(),
        }
    }
}

impl<K: Into<String>, V: Into<String>> From<(K, V)> for Attribute {
    fn from((k, v): (K, V)) -> Self {
        Attribute::new(k, v)
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
    fn event_construction() {
        let event_direct = Event {
            ty: "test".to_string(),
            attributes: vec![attr("foo", "bar"), attr("bar", "baz")],
        };
        let event_builder = Event::new("test").attr("foo", "bar").attr("bar", "baz");

        assert_eq!(event_direct, event_builder);
    }

    #[test]
    #[should_panic]
    fn attribute_new_reserved_key_panicks() {
        Attribute::new("_invalid", "value");
    }

    #[test]
    #[should_panic]
    fn attribute_new_reserved_key_panicks2() {
        Attribute::new("_", "value");
    }

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
