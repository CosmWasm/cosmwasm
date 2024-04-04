use super::points::{g1_from_fixed, g2_from_fixed, InvalidPoint};
use bls12_381::G2Prepared;
use pairing::group::Group;

/// Check whether the following condition holds true:
///
/// $$
/// e(p, q) = e(r, s)
/// $$
pub fn bls12_381_pairing_equality(
    p: &[u8; 48],
    q: &[u8; 96],
    r: &[u8; 48],
    s: &[u8; 96],
) -> Result<bool, InvalidPoint> {
    let (p, q, r, s) = (
        g1_from_fixed(p)?,
        g2_from_fixed(q)?,
        g1_from_fixed(r)?,
        g2_from_fixed(s)?,
    );

    let p_neg = -p;
    let terms = [
        (&p_neg.0, &G2Prepared::from(q.0)),
        (&r.0, &G2Prepared::from(s.0)),
    ];

    Ok(bls12_381::multi_miller_loop(&terms)
        .final_exponentiation()
        .is_identity()
        .into())
}

#[cfg(test)]
mod test {
    use digest::generic_array::GenericArray;
    use hex_literal::hex;
    use sha2_v9::{Digest, Sha256};

    use crate::{
        bls12_318::points::{g1_from_fixed, g2_from_fixed, g2_from_variable, G1},
        bls12_381_hash_to_g2, bls12_381_pairing_equality, HashFunction,
    };

    // Let's directly go for something really cool and advanced:
    // dRand compatibility of this API

    // See https://github.com/drand/kyber-bls12381/issues/22 and
    // https://github.com/drand/drand/pull/1249
    const DOMAIN_HASH_TO_G2: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";

    /// Public key League of Entropy Mainnet (curl -sS https://drand.cloudflare.com/info)
    const PK_LEO_MAINNET: [u8; 48] = hex!("868f005eb8e6e4ca0a47c8a77ceaa5309a47978a7c71bc5cce96366b5d7a569937c529eeda66c7293784a9402801af31");

    fn build_message(
        round: u64,
        previous_signature: &[u8],
    ) -> GenericArray<u8, <Sha256 as Digest>::OutputSize> {
        Sha256::new()
            .chain(previous_signature)
            .chain(round.to_be_bytes())
            .finalize()
    }

    #[test]
    fn pairing_equality_works() {
        let previous_signature = hex::decode("a609e19a03c2fcc559e8dae14900aaefe517cb55c840f6e69bc8e4f66c8d18e8a609685d9917efbfb0c37f058c2de88f13d297c7e19e0ab24813079efe57a182554ff054c7638153f9b26a60e7111f71a0ff63d9571704905d3ca6df0b031747").unwrap();
        let signature = hex::decode("82f5d3d2de4db19d40a6980e8aa37842a0e55d1df06bd68bddc8d60002e8e959eb9cfa368b3c1b77d18f02a54fe047b80f0989315f83b12a74fd8679c4f12aae86eaf6ab5690b34f1fddd50ee3cc6f6cdf59e95526d5a5d82aaa84fa6f181e42").unwrap();
        let round: u64 = 72785;

        let key = g1_from_fixed(&PK_LEO_MAINNET).unwrap();
        let sigma = g2_from_variable(&signature).unwrap();
        let g1 = G1::generator();
        let msg = build_message(round, &previous_signature);
        let g2_msg = bls12_381_hash_to_g2(HashFunction::Sha256, msg.as_slice(), DOMAIN_HASH_TO_G2);

        assert!(bls12_381_pairing_equality(
            &g1.to_compressed(),
            &sigma.to_compressed(),
            &PK_LEO_MAINNET,
            &g2_msg
        )
        .unwrap());

        // Do this in a separate scope to not shadow with wrong values
        {
            // Wrong round -> Therefore wrong hashed G2 point
            #[allow(clippy::unusual_byte_groupings)]
            let msg = build_message(0xDEAD_2_BAD, &previous_signature);
            let g2_msg =
                bls12_381_hash_to_g2(HashFunction::Sha256, msg.as_slice(), DOMAIN_HASH_TO_G2);

            assert!(!bls12_381_pairing_equality(
                &g1.to_compressed(),
                &sigma.to_compressed(),
                &PK_LEO_MAINNET,
                &g2_msg
            )
            .unwrap());
        }

        // curl -sS https://drand.cloudflare.com/public/1
        let previous_signature =
            hex::decode("176f93498eac9ca337150b46d21dd58673ea4e3581185f869672e59fa4cb390a")
                .unwrap();
        let signature = hex::decode("8d61d9100567de44682506aea1a7a6fa6e5491cd27a0a0ed349ef6910ac5ac20ff7bc3e09d7c046566c9f7f3c6f3b10104990e7cb424998203d8f7de586fb7fa5f60045417a432684f85093b06ca91c769f0e7ca19268375e659c2a2352b4655").unwrap();
        let round: u64 = 1;

        // Aggregate things down
        let aggregated_key = &key + &key;
        let aggregated_sigma = &sigma + &g2_from_variable(&signature).unwrap();
        let aggregated_g1 = &g1 + &g1;
        let aggregated_msg = &g2_from_fixed(&g2_msg).unwrap()
            + &g2_from_fixed(&bls12_381_hash_to_g2(
                HashFunction::Sha256,
                build_message(round, &previous_signature).as_slice(),
                DOMAIN_HASH_TO_G2,
            ))
            .unwrap();

        assert!(bls12_381_pairing_equality(
            &aggregated_g1.to_compressed(),
            &aggregated_sigma.to_compressed(),
            &aggregated_key.to_compressed(),
            &aggregated_msg.to_compressed()
        )
        .unwrap());
    }
}
