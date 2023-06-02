#[cfg(not(feature = "no-std"))]
pub use std::vec::Vec;

#[cfg(feature = "no-std")]
pub use alloc::vec;

#[cfg(feature = "no-std")]
pub use core::mem;
#[cfg(not(feature = "no-std"))]
use std::mem;

pub mod ops {
    #[cfg(feature = "no-std")]
    pub use core::ops::Deref;
    #[cfg(not(feature = "no-std"))]
    use std::ops::Deref;
}

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
pub use std::fmt;

#[cfg(feature = "no-std")]
pub use alloc::fmt;

pub mod borrow {
    #[cfg(not(feature = "no-std"))]
    pub use std::borrow::Cow;

    #[cfg(feature = "no-std")]
    pub use alloc::borrow::Cow;
}


pub mod any {
    #[cfg(not(feature = "no-std"))]
    pub use std::any::type_name;

    #[cfg(feature = "no-std")]
    pub use core::any::type_name;
}

// just because it is default prelude
pub mod prelude {
    pub use crate::cw_std::{string::String, vec::Vec};
}
