//! if somebody will need alternative implementation, for example for collection or fmt, one may patch for himself this one

#[cfg(feature = "std")]
pub use std::num;

#[cfg(not(feature = "std"))]
pub use core::num;

#[cfg(feature = "std")]
pub use std::boxed;

#[cfg(not(feature = "std"))]
pub use alloc::boxed;

#[cfg(feature = "std")]
pub use std::format;

#[cfg(not(feature = "std"))]
pub use alloc::format;

#[cfg(feature = "std")]
pub use std::vec;

#[cfg(not(feature = "std"))]
pub use alloc::vec;

#[cfg(not(feature = "std"))]
pub use core::mem;
#[cfg(feature = "std")]
pub use std::mem;

#[cfg(not(feature = "std"))]
pub use core::result;
#[cfg(feature = "std")]
pub use std::result;

#[cfg(not(feature = "std"))]
pub use core::cmp;
#[cfg(feature = "std")]
pub use std::cmp;

#[cfg(not(feature = "std"))]
pub use core::error;
#[cfg(feature = "std")]
pub use std::error;

#[cfg(feature = "std")]
pub use std::iter;

#[cfg(not(feature = "std"))]
pub use core::iter;

#[cfg(feature = "std")]
pub use std::ops;

#[cfg(not(feature = "std"))]
pub use core::ops;

#[cfg(feature = "std")]
pub use std::string;

#[cfg(not(feature = "std"))]
pub use alloc::string;

pub mod array {
    #[cfg(feature = "std")]
    pub use std::array::TryFromSliceError;

    #[cfg(not(feature = "std"))]
    pub use core::array::TryFromSliceError;
}

#[cfg(feature = "std")]
pub use std::convert;

#[cfg(not(feature = "std"))]
pub use core::convert;

#[cfg(feature = "std")]
pub use std::collections;

#[cfg(not(feature = "std"))]
pub use alloc::collections;

#[cfg(feature = "std")]
pub use std::fmt;

#[cfg(not(feature = "std"))]
pub use alloc::fmt;

pub mod borrow {
    #[cfg(feature = "std")]
    pub use std::borrow::Cow;

    #[cfg(not(feature = "std"))]
    pub use alloc::borrow::Cow;
}

#[cfg(feature = "std")]
pub use std::str;

#[cfg(not(feature = "std"))]
pub use alloc::str;

#[cfg(feature = "std")]
pub use std::marker;

#[cfg(not(feature = "std"))]
pub use core::marker;

pub mod any {
    #[cfg(feature = "std")]
    pub use std::any::type_name;

    #[cfg(not(feature = "std"))]
    pub use core::any::type_name;
}

// just because it is default prelude
pub mod prelude {
    pub use super::{
        boxed::Box,
        format,
        marker::{Send, Sync},
        result::Result::{Err, Ok},
        string::{String, ToString},
        vec,
        vec::Vec,
    };
}
