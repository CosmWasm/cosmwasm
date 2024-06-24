use crate::StdError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A record of a key-value storage that is created through an iterator API.
/// The first element (key) is always raw binary data. The second element
/// (value) is binary by default but can be changed to a custom type. This
/// allows contracts to reuse the type when deserializing database records.
pub type Record<V = Vec<u8>> = (Vec<u8>, V);

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
// We assign these to integers to provide a stable API for passing over FFI (to wasm and Go)
pub enum Order {
    Ascending = 1,
    Descending = 2,
}

impl TryFrom<i32> for Order {
    type Error = StdError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Order::Ascending),
            2 => Ok(Order::Descending),
            _ => Err(StdError::generic_err("Order must be 1 or 2")),
        }
    }
}

impl From<Order> for i32 {
    fn from(original: Order) -> i32 {
        original as _
    }
}

#[cfg(test)]
mod tests {
    use crate::{from_json, to_json_vec};

    use super::*;

    #[test]
    fn order_serde() {
        let ascending_bytes = br#""ascending""#;
        let descending_bytes = br#""descending""#;

        assert_eq!(to_json_vec(&Order::Ascending).unwrap(), ascending_bytes);
        assert_eq!(to_json_vec(&Order::Descending).unwrap(), descending_bytes);

        assert_eq!(
            from_json::<Order>(ascending_bytes).unwrap(),
            Order::Ascending
        );

        assert_eq!(
            from_json::<Order>(descending_bytes).unwrap(),
            Order::Descending
        );
    }
}
