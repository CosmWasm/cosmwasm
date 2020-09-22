pub mod contract;
mod errors;
pub mod msg;
pub mod state;

#[cfg(target_arch = "wasm32")]
cosmwasm_std::create_entry_points!(contract);
