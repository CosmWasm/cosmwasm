#![cfg_attr(target_arch = "wasm32", feature(asm_experimental_arch))]

pub mod contract;
pub(crate) mod floats;
mod instructions;
pub mod msg;
pub mod state;
