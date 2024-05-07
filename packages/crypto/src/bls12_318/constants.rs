pub const BLS12_381_G1_POINT_LEN: usize = 48;
pub const BLS12_381_G2_POINT_LEN: usize = 96;

pub const BLS12_381_G1_GENERATOR_COMPRESSED: [u8; BLS12_381_G1_POINT_LEN] = [
    151, 241, 211, 167, 49, 151, 215, 148, 38, 149, 99, 140, 79, 169, 172, 15, 195, 104, 140, 79,
    151, 116, 185, 5, 161, 78, 58, 63, 23, 27, 172, 88, 108, 85, 232, 63, 249, 122, 26, 239, 251,
    58, 240, 10, 219, 34, 198, 187,
];
pub const BLS12_381_G2_GENERATOR_COMPRESSED: [u8; BLS12_381_G2_POINT_LEN] = [
    147, 224, 43, 96, 82, 113, 159, 96, 125, 172, 211, 160, 136, 39, 79, 101, 89, 107, 208, 208,
    153, 32, 182, 26, 181, 218, 97, 187, 220, 127, 80, 73, 51, 76, 241, 18, 19, 148, 93, 87, 229,
    172, 125, 5, 93, 4, 43, 126, 2, 74, 162, 178, 240, 143, 10, 145, 38, 8, 5, 39, 45, 197, 16, 81,
    198, 228, 122, 212, 250, 64, 59, 2, 180, 81, 11, 100, 122, 227, 209, 119, 11, 172, 3, 38, 168,
    5, 187, 239, 212, 128, 86, 200, 193, 33, 189, 184,
];

#[cfg(test)]
mod test {
    use ark_bls12_381::{G1Affine, G2Affine};
    use ark_ec::AffineRepr;
    use ark_serialize::CanonicalSerialize;
    use hex_literal::hex;

    use super::{BLS12_381_G1_GENERATOR_COMPRESSED, BLS12_381_G2_GENERATOR_COMPRESSED};

    use crate::{BLS12_381_G1_POINT_LEN, BLS12_381_G2_POINT_LEN};

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
        // Source: <https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-pairing-friendly-curves-02#section-4.3.2>
        //
        // See the `x` coordinate
        let mut generator = hex!("17f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb");
        generator[0] |= 0b1000_0000;
        assert_eq!(generator, bls12_381_g1_generator());
        assert_eq!(bls12_381_g1_generator(), BLS12_381_G1_GENERATOR_COMPRESSED);
    }

    #[test]
    fn g2_generator_correct() {
        // Source: <https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-pairing-friendly-curves-02#section-4.3.2>
        //
        // $$
        // G2_{raw} = x'_1 || x'_0
        // $$
        let mut generator = hex!("13e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8");
        generator[0] |= 0b1000_0000;
        assert_eq!(generator, bls12_381_g2_generator());
        assert_eq!(bls12_381_g2_generator(), BLS12_381_G2_GENERATOR_COMPRESSED);
    }
}
