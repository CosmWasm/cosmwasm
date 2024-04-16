use crate::{
    errors::AggregationPairingEquality, CryptoError, BLS12_381_G1_POINT_LEN, BLS12_381_G2_POINT_LEN,
};

use super::points::{g1_from_variable, g2_from_variable};
use ark_bls12_381::Bls12_381;
use ark_ec::{
    bls12::{G1Prepared, G2Prepared},
    pairing::Pairing,
};
use num_traits::Zero;
use rayon::iter::{ParallelBridge, ParallelIterator};

pub fn bls12_381_aggregate_pairing_equality(
    ps: &[u8],
    qs: &[u8],
    r: &[u8],
    s: &[u8],
) -> Result<bool, CryptoError> {
    if ps.is_empty() {
        return Err(AggregationPairingEquality::EmptyG1.into());
    } else if qs.is_empty() {
        return Err(AggregationPairingEquality::EmptyG2.into());
    } else if ps.len() % BLS12_381_G1_POINT_LEN != 0 {
        return Err(AggregationPairingEquality::NotMultipleG1 {
            remainder: ps.len() % BLS12_381_G1_POINT_LEN,
        }
        .into());
    } else if qs.len() % BLS12_381_G2_POINT_LEN != 0 {
        return Err(AggregationPairingEquality::NotMultipleG2 {
            remainder: qs.len() % BLS12_381_G2_POINT_LEN,
        }
        .into());
    } else if ps.len() % BLS12_381_G1_POINT_LEN != qs.len() % BLS12_381_G2_POINT_LEN {
        return Err(AggregationPairingEquality::UnequalPointAmount {
            left: ps.len() % BLS12_381_G1_POINT_LEN,
            right: qs.len() % BLS12_381_G2_POINT_LEN,
        }
        .into());
    }

    let pq_pairs: Vec<_> = ps
        .chunks_exact(BLS12_381_G1_POINT_LEN)
        .zip(qs.chunks_exact(BLS12_381_G2_POINT_LEN))
        // From here on parallelism is fine since the miller loop runs over
        // a sum of the pairings and is therefore a commutative operation
        .par_bridge()
        .map(|(p, q)| {
            let g1 = g1_from_variable(p)?;
            let g2 = g2_from_variable(q)?;

            Ok((G1Prepared::from(g1.0), G2Prepared::from(g2.0)))
        })
        .collect::<Result<_, CryptoError>>()?;

    let r = g1_from_variable(r)?;
    let s = g2_from_variable(s)?;

    let r_neg = G1Prepared::from(-r.0);
    let s_prepared = G2Prepared::from(s.0);

    let (ps, qs): (Vec<_>, Vec<_>) = pq_pairs.into_iter().chain([(r_neg, s_prepared)]).unzip();

    Ok(Bls12_381::multi_pairing(ps, qs).is_zero())
}

/// Check whether the following condition holds true:
///
/// $$
/// e(p, q) = e(r, s)
/// $$
pub fn bls12_381_pairing_equality(
    p: &[u8],
    q: &[u8],
    r: &[u8],
    s: &[u8],
) -> Result<bool, CryptoError> {
    let (p, q, r, s) = (
        g1_from_variable(p)?,
        g2_from_variable(q)?,
        g1_from_variable(r)?,
        g2_from_variable(s)?,
    );

    let p_neg = -p;

    Ok(Bls12_381::multi_pairing(
        [G1Prepared::from(p_neg.0), G1Prepared::from(r.0)],
        [G2Prepared::from(q.0), G2Prepared::from(s.0)],
    )
    .is_zero())
}

#[cfg(test)]
mod test {
    use hex_literal::hex;
    use sha2::{Digest, Sha256};

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

    fn build_message(round: u64, previous_signature: &[u8]) -> digest::Output<Sha256> {
        Sha256::new()
            .chain_update(previous_signature)
            .chain_update(round.to_be_bytes())
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
