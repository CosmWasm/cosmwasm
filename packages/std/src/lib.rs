// Exposed on all platforms

pub mod encoding;
pub mod errors;
pub mod mock;
pub mod serde;
pub mod storage;
pub mod traits;
pub mod types;

// Exposed in wasm build only

#[cfg(target_arch = "wasm32")]
pub mod exports;
#[cfg(target_arch = "wasm32")]
pub mod imports;
#[cfg(target_arch = "wasm32")]
pub mod memory; // used by exports and imports only

// Not exposed

mod demo;
