//! if somebody will need alternative implementation, for example for collection or fmt, one may patch for himself this one

#[cfg(not(feature = "no-std"))]
pub use std::vec;

#[cfg(feature = "no-std")]
pub use alloc::vec;

#[cfg(feature = "no-std")]
pub use core::mem;
#[cfg(not(feature = "no-std"))]
pub use std::mem;


#[cfg(feature = "no-std")]
pub use core::result;
#[cfg(not(feature = "no-std"))]
pub use std::result;


#[cfg(feature = "no-std")]
pub use core::cmp;
pub use std::cmp;

#[cfg(not(feature = "no-std"))]
pub use std::iter;

#[cfg(feature = "no-std")]
pub use core::iter;

#[cfg(not(feature = "no-std"))]
pub use std::ops;

#[cfg(feature = "no-std")]
pub use core::ops;

pub mod string {
    #[cfg(not(feature = "no-std"))]
    pub use std::string::String;

    #[cfg(feature = "no-std")]
    pub use alloc::string::String;
}

pub mod marker {
    #[cfg(feature = "no-std")]
    pub use core::marker::PhantomData;
    #[cfg(not(feature = "no-std"))]
    pub use std::marker::PhantomData;
}

pub mod array {
    #[cfg(not(feature = "no-std"))]
    pub use std::array::TryFromSliceError;

    #[cfg(feature = "no-std")]
    pub use core::array::TryFromSliceError;
}

pub mod convert {
    #[cfg(not(feature = "no-std"))]
    pub use std::convert::TryInto;

    #[cfg(feature = "no-std")]
    pub use core::convert::TryInto;
}




#[cfg(not(feature = "no-std"))]
pub use std::collections;

#[cfg(feature = "no-std")]
pub use alloc::collections;


#[cfg(not(feature = "no-std"))]
pub use std::fmt;

#[cfg(feature = "no-std")]
pub use alloc::fmt;

pub mod borrow {
    #[cfg(not(feature = "no-std"))]
    pub use std::borrow::Cow;

    #[cfg(feature = "no-std")]
    pub use alloc::borrow::Cow;
}


pub mod str {
    #[cfg(not(feature = "no-std"))]
    pub use std::str::FromStr;

    #[cfg(feature = "no-std")]
    pub use alloc::str::FromStr;
}

pub mod any {
    #[cfg(not(feature = "no-std"))]
    pub use std::any::type_name;

    #[cfg(feature = "no-std")]
    pub use core::any::type_name;
}

// just because it is default prelude
pub mod prelude {
    pub use crate::cw_std::{string::String, vec::Vec, result::Result::{Err, Ok}};
}
