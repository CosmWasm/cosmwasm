pub mod contract;
pub mod msg;
pub mod state;

#[cfg(target_arch = "wasm32")]
cosmwasm_std::create_entry_points!(contract);
