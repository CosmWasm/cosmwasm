#[cfg(test)]
mod test {
    use ark_bls12_381::{G1Affine, G2Affine};
    use ark_ec::AffineRepr;
    use ark_serialize::CanonicalSerialize;
    use hex_literal::hex;

    use crate::{
        bls12_318::{BLS12_381_G1_GENERATOR_COMPRESSED, BLS12_381_G2_GENERATOR_COMPRESSED},
        BLS12_381_G1_POINT_LEN, BLS12_381_G2_POINT_LEN,
    };

    fn bls12_381_g1_generator() -> [u8; BLS12_381_G1_POINT_LEN] {
        let mut point = [0_u8; BLS12_381_G1_POINT_LEN];
        G1Affine::generator()
            .serialize_compressed(&mut point[..])
            .unwrap();

        point
    }

    fn bls12_381_g2_generator() -> [u8; BLS12_381_G2_POINT_LEN] {
        let mut point = [0_u8; BLS12_381_G2_POINT_LEN];
        G2Affine::generator()
            .serialize_compressed(&mut point[..])
            .unwrap();

        point
    }

    #[test]
    fn g1_generator_correct() {
        let mut generator = hex!("17f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb");
        generator[0] |= 0b1000_0000;
        assert_eq!(generator, bls12_381_g1_generator());
        assert_eq!(bls12_381_g1_generator(), BLS12_381_G1_GENERATOR_COMPRESSED);
    }

    #[test]
    fn g2_generator_correct() {
        let mut generator = hex!("13e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8");
        generator[0] |= 0b1000_0000;
        assert_eq!(generator, bls12_381_g2_generator());
        assert_eq!(bls12_381_g2_generator(), BLS12_381_G2_GENERATOR_COMPRESSED);
    }
}
