pub mod contract;
pub mod msg;
pub mod state;

#[cfg(not(target_arch = "wasm32"))]
pub mod testing;

#[cfg(target_arch = "wasm32")]
cosmwasm_std::entry_points!(contract);
