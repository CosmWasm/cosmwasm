pub mod contract;

#[cfg(target_arch = "wasm32")]
cosmwasm_std::entry_points!(contract);
