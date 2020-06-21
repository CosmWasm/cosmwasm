pub mod contract;
pub mod msg;

#[cfg(target_arch = "wasm32")]
cosmwasm_std::entry_points!(contract);
