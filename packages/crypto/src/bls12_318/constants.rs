use bls12_381::{G1Affine, G2Affine};

pub fn bls12_381_g1_generator() -> [u8; 48] {
    G1Affine::generator().to_compressed()
}

pub fn bls12_381_g2_generator() -> [u8; 96] {
    G2Affine::generator().to_compressed()
}
