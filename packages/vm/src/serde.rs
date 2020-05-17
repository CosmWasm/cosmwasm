//! This file simply re-exports some methods from serde_json
//! The reason is two fold:
//! 1. To easily ensure that all calling libraries use the same version (minimize code size)
//! 2. To allow us to switch out to eg. serde-json-core more easily
use serde::{Deserialize, Serialize};
use std::any::type_name;

use crate::errors::{make_parse_err, make_serialize_err, VmResult};

pub fn from_slice<'a, T>(value: &'a [u8]) -> VmResult<T>
where
    T: Deserialize<'a>,
{
    serde_json::from_slice(value).map_err(|e| make_parse_err(type_name::<T>(), e))
}

pub fn to_vec<T>(data: &T) -> VmResult<Vec<u8>>
where
    T: Serialize + ?Sized,
{
    serde_json::to_vec(data).map_err(|e| make_serialize_err(type_name::<T>(), e))
}
