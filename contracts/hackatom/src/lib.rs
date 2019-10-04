pub mod contract;
pub mod imports;
pub mod memory;
pub mod mock;
pub mod types;

/** Below we expose wasm exports **/
#[cfg(target_arch = "wasm32")]
mod exports;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;
    pub use exports::{allocate, deallocate};
    use std::ffi::c_void;

    #[no_mangle]
    pub extern "C" fn init_wrapper(
        params_ptr: *mut c_void,
        msg_ptr: *mut c_void,
    ) -> *mut c_void {
        exports::init(
            &contract::init::<imports::ExternalStorage>,
            params_ptr,
            msg_ptr,
        )
    }

    #[no_mangle]
    pub extern "C" fn send_wrapper(
        params_ptr: *mut c_void,
        msg_ptr: *mut c_void,
    ) -> *mut c_void {
        exports::send(
            &contract::send::<imports::ExternalStorage>,
            params_ptr,
            msg_ptr,
        )
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::{allocate, deallocate, init_wrapper, send_wrapper};
