#![no_std]

mod crypto;

#[doc(hidden)]
pub use self::crypto::{
    BLS12_381_G1_GENERATOR, BLS12_381_G1_POINT_LEN, BLS12_381_G2_GENERATOR, BLS12_381_G2_POINT_LEN,
};
