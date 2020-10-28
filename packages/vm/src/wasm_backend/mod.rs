mod gas;
mod singlepass;

pub use gas::{decrease_gas_left, get_gas_left, set_gas_left, InsufficientGasLeft};
pub use singlepass::compile;
