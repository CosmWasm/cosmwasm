//!
//! # ⚠ DO NOT DEPEND ON THIS AS AN OUTSIDE CONSUMER
//!
//! **THIS MODULE IS SEMVER EXCEMPT AND ONLY MEANT TO SHARE TYPES BETWEEN CORE AND STD**
//!
//! Module for re-exporting implementation details from `core` to `std`
//!

pub use crate::__internal__forward_ref_partial_eq as forward_ref_partial_eq;
