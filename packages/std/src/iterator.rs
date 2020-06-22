use crate::errors::StdError;
use std::convert::TryFrom;

/// KV is a Key-Value pair, returned from our iterators
pub type KV<T = Vec<u8>> = (Vec<u8>, T);

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

impl Into<i32> for Order {
    fn into(self) -> i32 {
        self as i32
    }
}
