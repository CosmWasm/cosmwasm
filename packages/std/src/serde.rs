// This file simply re-exports some methods from serde_json
// The reason is two fold:
// 1. To easily ensure that all calling libraries use the same version (minimize code size)
// 2. To allow us to switch out to eg. serde-json-core more easily
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::any::type_name;

use crate::encoding::Binary;
use crate::errors::{ParseErr, SerializeErr, StdResult};

pub fn from_slice<'a, T>(value: &'a [u8]) -> StdResult<T>
where
    T: Deserialize<'a>,
{
    serde_json_wasm::from_slice(value).context(ParseErr {
        kind: type_name::<T>(),
    })
}

pub fn from_binary<'a, T>(value: &'a Binary) -> StdResult<T>
where
    T: Deserialize<'a>,
{
    from_slice(value.as_slice())
}

pub fn to_vec<T>(data: &T) -> StdResult<Vec<u8>>
where
    T: Serialize + ?Sized,
{
    serde_json_wasm::to_vec(data).context(SerializeErr {
        kind: type_name::<T>(),
    })
}

pub fn to_binary<T>(data: &T) -> StdResult<Binary>
where
    T: Serialize + ?Sized,
{
    to_vec(data).map(Binary)
}
