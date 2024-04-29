mod aggregate;
mod constants;
mod hash;
mod pairing;
mod points;

pub use aggregate::{bls12_381_aggregate_g1, bls12_381_aggregate_g2};
pub use constants::{bls12_381_g1_generator, bls12_381_g2_generator};
pub use hash::{bls12_381_hash_to_g1, bls12_381_hash_to_g2, HashFunction};
pub use pairing::bls12_381_aggregate_pairing_equality;
pub use points::{bls12_381_g1_is_identity, bls12_381_g2_is_identity};

pub const BLS12_381_G1_POINT_LEN: usize = 48;
pub const BLS12_381_G2_POINT_LEN: usize = 96;
