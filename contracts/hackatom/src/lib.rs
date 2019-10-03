pub mod contract;
pub mod imports;
pub mod types;

/** Below we expose wasm exports **/

#[cfg(target_arch = "wasm32")]
mod exports;

#[cfg(target_arch = "wasm32")]
pub use crate::exports::{allocate, deallocate};
#[cfg(target_arch = "wasm32")]
use std::os::raw::{c_char};

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn init_wrapper(params_ptr: *mut c_char, msg_ptr: *mut c_char) -> *mut c_char {
    exports::init(&contract::init::<imports::ExternalStorage>, params_ptr, msg_ptr)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn send_wrapper(params_ptr: *mut c_char, msg_ptr: *mut c_char) -> *mut c_char {
    exports::send(&contract::send::<imports::ExternalStorage>, params_ptr, msg_ptr)
}
