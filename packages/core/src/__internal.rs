//!
//! # ⚠ DO NOT DEPEND ON THIS AS AN OUTSIDE CONSUMER
//!
//! Module for re-exporting implementation details from `core` to `std`
//!

pub mod errors {
    pub use crate::errors::*;
}

pub mod backtrace {
    pub use crate::errors::backtrace::*;
}
