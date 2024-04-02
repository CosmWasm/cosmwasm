use super::points::{Gt, G1, G2};
use bls12_381::G2Prepared;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

/// Invoke the pairing function over the pair
pub fn bls12_381_pairing(p: &G1, q: &G2) -> Gt {
    Gt(bls12_381::pairing(&p.0, &q.0))
}

/// Compute the sums of the miller loop invocations over a series of point pairs
/// and execute the final exponentiation.
pub fn bls12_381_multi_miller_loop(points: &[(&G1, &G2)]) -> Gt {
    let mut prepared_g2 = Vec::with_capacity(points.len());
    points
        .par_iter()
        .map(|(_g1, g2)| G2Prepared::from(g2.0))
        .collect_into_vec(&mut prepared_g2);

    let mut terms = Vec::with_capacity(points.len());
    let term_iter = points.iter().map(|(g1, _g2)| &g1.0).zip(prepared_g2.iter());
    terms.extend(term_iter);

    Gt(bls12_381::multi_miller_loop(&terms).final_exponentiation())
}

/// Check whether the following condition holds true:
///
/// $$
/// e(p, q) = e(r, s)
/// $$
pub fn bls12_381_pairing_equality(p: &G1, q: &G2, r: &G1, s: &G2) -> bool {
    let p_neg = -p;
    let terms = [(&p_neg, q), (r, s)];
    bls12_381_multi_miller_loop(&terms).is_identity()
}

#[cfg(test)]
mod test {
    use bls12_381::hash_to_curve::ExpandMsgXmd;
    use digest::generic_array::GenericArray;
    use hex_literal::hex;
    use sha2_v9::{Digest, Sha256};

    use crate::{
        bls12_318::points::{g1_from_fixed, g2_from_hash, g2_from_variable, G1},
        bls12_381_pairing_equality,
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
        let g2_msg = g2_from_hash::<ExpandMsgXmd<Sha256>>(msg.as_slice(), DOMAIN_HASH_TO_G2);

        assert!(bls12_381_pairing_equality(&g1, &sigma, &key, &g2_msg));

        // Wrong round -> Therefore wrong hashed G2 point
        #[allow(clippy::unusual_byte_groupings)]
        let msg = build_message(0xDEAD_2_BAD, &previous_signature);
        let g2_msg = g2_from_hash::<ExpandMsgXmd<Sha256>>(msg.as_slice(), DOMAIN_HASH_TO_G2);

        assert!(!bls12_381_pairing_equality(&g1, &sigma, &key, &g2_msg));
    }
}
