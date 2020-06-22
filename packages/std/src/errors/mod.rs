mod std_error;
mod std_error_helpers;
mod system_error;

pub use std_error::{StdError, StdResult};
pub use std_error_helpers::{
    generic_err, invalid_base64, invalid_utf8, not_found, parse_err, serialize_err, unauthorized,
    underflow,
};
pub use system_error::{SystemError, SystemResult};
