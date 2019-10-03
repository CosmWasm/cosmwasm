use std::vec::Vec;

pub trait Storage {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
    fn set(&mut self, key: &[u8], value: &[u8]);
}

#[cfg(target_arch = "wasm32")]
pub use wasm::ExternalStorage;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;
    use std::ffi::{CStr, CString};
    use std::os::raw::c_char;

    extern "C" {
        // both take an opaque database ref that can be used by the environment to determine which
        // substore to allow read/writes from
        fn c_read(dbref: i32, key: *const c_char) -> *mut c_char;
        fn c_write(dbref: i32, key: *const c_char, value: *mut c_char);
    }

    pub struct ExternalStorage {
        dbref: i32,
    }

    impl ExternalStorage {
        pub fn new(dbref: i32) -> ExternalStorage {
            ExternalStorage { dbref }
        }
    }

    impl Storage for ExternalStorage {
        fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
            unsafe {
                let key = CString::new(key).unwrap().into_raw();
                let ptr = c_read(self.dbref, key);
                if ptr.is_null() {
                    return None;
                }
                let state = CStr::from_ptr(ptr).to_bytes().to_vec();
                return Some(state);
            }
        }

        fn set(&mut self, key: &[u8], value: &[u8]) {
            unsafe {
                let key = CString::new(key).unwrap().into_raw();
                let value = CString::new(value).unwrap().into_raw();
                c_write(self.dbref, key, value);
            }
        }
    }
}

