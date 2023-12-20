pub use alloc::boxed::Box;
pub use alloc::format;
pub use alloc::string::{String, ToString};
pub use alloc::vec;
pub use alloc::vec::Vec;
pub use core::option::Option::{self, None, Some};

#[cfg(not(feature = "std"))]
core::compile_error!("Please enable `cosmwasm-std`'s `std` feature, as we might move existing functionality to that feature in the future.");
