#[macro_export]
macro_rules! set_wasm_version {
    ($s1: expr, $s2: expr) => {
        /// Private module with no stability guarantees
        mod __cw5 {
            use super::*;

            const FULL: &str = const_str::concat!("/", $s1, "/", $s2);

            #[link_section = "cw5"]
            #[allow(unused)]
            static AS_BYTES: [u8; FULL.len()] = const_str::to_byte_array!(FULL);
        }
    };
}
