pub mod contract;

/** Below we expose wasm exports **/
#[cfg(target_arch = "wasm32")]
pub use cosmwasm_std::{allocate, deallocate};

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::contract;
    use cosmwasm_std::{do_handle, do_init, do_query, ExternalApi, ExternalStorage};
    use std::ffi::c_void;

    #[no_mangle]
    pub extern "C" fn init(env_ptr: *mut c_void, msg_ptr: *mut c_void) -> *mut c_void {
        do_init(
            &contract::init::<ExternalStorage, ExternalApi>,
            env_ptr,
            msg_ptr,
        )
    }

    #[no_mangle]
    pub extern "C" fn handle(env_ptr: *mut c_void, msg_ptr: *mut c_void) -> *mut c_void {
        do_handle(
            &contract::handle::<ExternalStorage, ExternalApi>,
            env_ptr,
            msg_ptr,
        )
    }

    #[no_mangle]
    pub extern "C" fn query(msg_ptr: *mut c_void) -> *mut c_void {
        do_query(&contract::query::<ExternalStorage, ExternalApi>, msg_ptr)
    }
}
