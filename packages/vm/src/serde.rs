//! This file simply re-exports some methods from serde_json
//! The reason is two fold:
//! 1. To easily ensure that all calling libraries use the same version (minimize code size)
//! 2. To allow us to switch out to eg. serde-json-core more easily
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::any::type_name;

use crate::errors::{ParseErr, SerializeErr, VmResult};

pub fn from_slice<'a, T>(value: &'a [u8]) -> VmResult<T>
where
    T: Deserialize<'a>,
{
    serde_json::from_slice(value).context(ParseErr {
        kind: type_name::<T>(),
    })
}

pub fn to_vec<T>(data: &T) -> VmResult<Vec<u8>>
where
    T: Serialize + ?Sized,
{
    serde_json::to_vec(data).context(SerializeErr {
        kind: type_name::<T>(),
    })
}
