// This file simply re-exports some methods from serde_json
// The reason is two fold:
// 1. To easily ensure that all calling libraries use the same version (minimize code size)
// 2. To allow us to switch out to eg. serde-json-core more easily
use prost::Message;
use std::any::type_name;

use crate::binary::Binary;
use crate::errors::{StdError, StdResult};

pub fn from_slice<T: Message + Default>(value: &[u8]) -> StdResult<T> {
    // TODO: make a unique error variant?
    T::decode(value).map_err(|e| StdError::parse_err(type_name::<T>(), e))
}

pub fn from_binary<T: Message + Default>(value: &Binary) -> StdResult<T> {
    from_slice(value.as_slice())
}

pub fn to_vec<T>(data: &T) -> StdResult<Vec<u8>>
where
    T: Message,
{
    Ok(data.encode_to_vec())
}

pub fn to_binary<T>(data: &T) -> StdResult<Binary>
where
    T: Message,
{
    to_vec(data).map(Binary)
}
