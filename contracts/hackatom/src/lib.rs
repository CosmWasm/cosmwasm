pub mod contract;
pub mod types;
pub mod imports;

/** Below we expose wasm exports **/
#[cfg(target_arch = "wasm32")]
mod exports;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;
    pub use exports::{allocate, deallocate};
    use std::os::raw::c_char;

    #[no_mangle]
    pub extern "C" fn init_wrapper(
        dbref: i32,
        params_ptr: *mut c_char,
        msg_ptr: *mut c_char,
    ) -> *mut c_char {
        exports::init(
            &contract::init::<imports::ExternalStorage>,
            dbref,
            params_ptr,
            msg_ptr,
        )
    }

    #[no_mangle]
    pub extern "C" fn send_wrapper(
        dbref: i32,
        params_ptr: *mut c_char,
        msg_ptr: *mut c_char,
    ) -> *mut c_char {
        exports::send(
            &contract::send::<imports::ExternalStorage>,
            dbref,
            params_ptr,
            msg_ptr,
        )
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::{allocate, deallocate, init_wrapper, send_wrapper};
