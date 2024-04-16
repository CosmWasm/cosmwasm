use bls12_381::{G1Affine, G2Affine};

use crate::{BLS12_381_G1_POINT_LEN, BLS12_381_G2_POINT_LEN};

pub fn bls12_381_g1_generator() -> [u8; BLS12_381_G1_POINT_LEN] {
    G1Affine::generator().to_compressed()
}

pub fn bls12_381_g2_generator() -> [u8; BLS12_381_G2_POINT_LEN] {
    G2Affine::generator().to_compressed()
}
