use crate::errors::StdError;
use std::convert::TryFrom;

/// A pair of values, returned from our iterators
pub type Pair<A = Vec<u8>, B = Vec<u8>> = (A, B);

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

#[cfg(test)]
mod test {
    use super::Pair;

    #[test]
    // Generic K works
    fn pair_works() {
        let _default: Pair = (vec![1, 2, 3], vec![5]);
        let _value: Pair<u64> = (1234567890, vec![4, 3]);
        let _with_key: Pair<String, u64> = ("hello".to_owned(), 12345678);
    }
}
