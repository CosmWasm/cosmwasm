/// When compiled to Wasm, this installs a panic handler that aborts the
/// contract in case of a panic.
/// For other targets, this is a noop.
pub fn install_panic_handler() {
    #[cfg(target_arch = "wasm32")]
    {
        use super::imports::handle_panic;
        std::panic::set_hook(Box::new(|info| {
            // E.g. "panicked at 'oh no (a = 3)', src/contract.rs:51:5"
            let full_message = info.to_string();
            handle_panic(&full_message);
        }));
    }
}
