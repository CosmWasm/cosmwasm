//! CosmWasm is a smart contract platform for the Cosmos ecosystem.
//! This crate contains components of cosmwasm-std that can be used in no_std environments.
//!
//! For more information, see: <https://docs.cosmwasm.com>

#![no_std]

mod crypto;

#[doc(hidden)]
pub use self::crypto::{
    BLS12_381_G1_GENERATOR, BLS12_381_G1_POINT_LEN, BLS12_381_G2_GENERATOR, BLS12_381_G2_POINT_LEN,
};
