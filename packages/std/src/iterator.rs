use crate::errors::StdError;
use std::convert::TryFrom;

/// A Key-Value pair, returned from our iterators
pub type Pair<V = Vec<u8>> = (Vec<u8>, V);

#[derive(Copy, Clone)]
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
