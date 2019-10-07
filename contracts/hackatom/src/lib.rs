pub mod contract;

/** Below we expose wasm exports **/
#[cfg(target_arch = "wasm32")]
pub use cosmwasm::exports::{allocate, deallocate};

#[cfg(target_arch = "wasm32")]
pub use wasm::{handle, init};

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::contract;
    use cosmwasm::{exports, imports};
    use std::ffi::c_void;

    #[no_mangle]
    pub extern "C" fn init(params_ptr: *mut c_void, msg_ptr: *mut c_void) -> *mut c_void {
        exports::do_init(
            &contract::init::<imports::ExternalStorage>,
            params_ptr,
            msg_ptr,
        )
    }

    #[no_mangle]
    pub extern "C" fn handle(params_ptr: *mut c_void, msg_ptr: *mut c_void) -> *mut c_void {
        exports::do_handle(
            &contract::handle::<imports::ExternalStorage>,
            params_ptr,
            msg_ptr,
        )
    }
}
