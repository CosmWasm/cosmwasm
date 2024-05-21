mod constants;

pub use self::constants::{
    BLS12_381_G1_GENERATOR, BLS12_381_G1_POINT_LEN, BLS12_381_G2_GENERATOR, BLS12_381_G2_POINT_LEN,
};

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        mod aggregate;
        mod hash;
        mod pairing;
        mod points;

        pub use self::aggregate::{bls12_381_aggregate_g1, bls12_381_aggregate_g2};
        pub use self::hash::{bls12_381_hash_to_g1, bls12_381_hash_to_g2, HashFunction};
        pub use self::pairing::bls12_381_pairing_equality;
        pub use self::points::{bls12_381_g1_is_identity, bls12_381_g2_is_identity};
    }
}
