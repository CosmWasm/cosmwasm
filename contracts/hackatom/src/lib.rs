pub mod contract;
pub mod types;

mod imports;

/** Below we expose wasm exports **/
#[cfg(target_arch = "wasm32")]
mod exports;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;
    use std::os::raw::{c_char};
    pub use exports::{allocate, deallocate};

    #[no_mangle]
    pub extern "C" fn init_wrapper(params_ptr: *mut c_char, msg_ptr: *mut c_char) -> *mut c_char {
        exports::init(&contract::init::<imports::ExternalStorage>, params_ptr, msg_ptr)
    }

    #[no_mangle]
    pub extern "C" fn send_wrapper(params_ptr: *mut c_char, msg_ptr: *mut c_char) -> *mut c_char {
        exports::send(&contract::send::<imports::ExternalStorage>, params_ptr, msg_ptr)
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::{allocate, deallocate, init_wrapper, send_wrapper};
