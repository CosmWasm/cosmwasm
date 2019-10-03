use std::vec::Vec;

pub trait Storage {
    fn get_state(&self) -> Option<Vec<u8>>;
    fn set_state(&mut self, state: Vec<u8>);
}

#[cfg(target_arch = "wasm32")]
pub use wasm::{ExternalStorage};

#[cfg(test)]
pub use mock::{MockStorage};

#[cfg(target_arch = "wasm32")]
mod wasm {
    extern "C" {
        fn c_read() -> *mut c_char;
        fn c_write(string: *mut c_char);
    }

    use super::*;
    use std::os::raw::{c_char};

    pub struct ExternalStorage {}

    impl Storage for ExternalStorage {
        fn get_state(&self) -> Option<Vec<u8>> {
            use std::ffi::{CStr};
            unsafe {
                let ptr = c_read();
                if ptr.is_null() {
                    return None;
                }
                let state = CStr::from_ptr(ptr).to_bytes().to_vec();
                return Some(state);
            }
        }

        fn set_state(&mut self, state: Vec<u8>) {
            use std::ffi::{CString};
            unsafe {
                c_write(CString::new(state).unwrap().into_raw());
            }
        }
    }
}

#[cfg(test)]
mod mock {
    use super::*;

    pub struct MockStorage {
        data: Option<Vec<u8>>
    }

    impl MockStorage {
        pub fn new() -> MockStorage {
            MockStorage{data: None}
        }
    }

    impl Storage for &mut MockStorage {
        fn get_state(&self) -> Option<Vec<u8>> {
            match &self.data {
                Some(v) => Some(v.clone()),
                None => None,
            }
        }

        fn set_state(&mut self, state: Vec<u8>) {
            self.data = Some(state);
        }
    }
}
