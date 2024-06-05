use core::ops::Neg;

use super::points::{g1_from_variable, g2_from_variable};
use crate::{errors::PairingEquality, CryptoError};

use ark_bls12_381::Bls12_381;
use ark_ec::{
    bls12::{G1Prepared, G2Prepared},
    pairing::Pairing,
};
use cosmwasm_core::{BLS12_381_G1_POINT_LEN, BLS12_381_G2_POINT_LEN};
use num_traits::Zero;
use rayon::{
    iter::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSlice,
};

pub fn bls12_381_pairing_equality(
    ps: &[u8],
    qs: &[u8],
    r: &[u8],
    s: &[u8],
) -> Result<bool, CryptoError> {
    if ps.len() % BLS12_381_G1_POINT_LEN != 0 {
        return Err(PairingEquality::NotMultipleG1 {
            remainder: ps.len() % BLS12_381_G1_POINT_LEN,
        }
        .into());
    } else if qs.len() % BLS12_381_G2_POINT_LEN != 0 {
        return Err(PairingEquality::NotMultipleG2 {
            remainder: qs.len() % BLS12_381_G2_POINT_LEN,
        }
        .into());
    } else if (ps.len() / BLS12_381_G1_POINT_LEN) != (qs.len() / BLS12_381_G2_POINT_LEN) {
        return Err(PairingEquality::UnequalPointAmount {
            left: ps.len() / BLS12_381_G1_POINT_LEN,
            right: qs.len() / BLS12_381_G2_POINT_LEN,
        }
        .into());
    }

    let p_iter = ps
        .par_chunks_exact(BLS12_381_G1_POINT_LEN)
        .map(g1_from_variable)
        .chain([g1_from_variable(r).map(Neg::neg)])
        .map(|g1_res| g1_res.map(|g1| G1Prepared::from(g1.0)));

    let q_iter = qs
        .par_chunks_exact(BLS12_381_G2_POINT_LEN)
        .map(g2_from_variable)
        .chain([g2_from_variable(s)])
        .map(|g2_res| g2_res.map(|g2| G2Prepared::from(g2.0)));

    let pq_pairs: Vec<_> = p_iter
        .zip_eq(q_iter)
        .map(|(p_res, q_res)| Ok((p_res?, q_res?)))
        .collect::<Result<_, CryptoError>>()?;

    let (ps, qs): (Vec<_>, Vec<_>) = pq_pairs.into_iter().unzip();

    Ok(Bls12_381::multi_pairing(ps, qs).is_zero())
}

#[cfg(test)]
mod test {
    use hex_literal::hex;
    use sha2::{Digest, Sha256};

    use crate::{
        bls12_318::points::{g1_from_fixed, g2_from_fixed, g2_from_variable, G1},
        bls12_381_hash_to_g2, bls12_381_pairing_equality, CryptoError, HashFunction,
        PairingEqualityError,
    };

    // Let's directly go for something really cool and advanced:
    // dRand compatibility of this API

    // See https://github.com/drand/kyber-bls12381/issues/22 and
    // https://github.com/drand/drand/pull/1249
    const DOMAIN_HASH_TO_G2: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";

    /// Public key League of Entropy Mainnet (curl -sS https://drand.cloudflare.com/info)
    const PK_LEO_MAINNET: [u8; 48] = hex!("868f005eb8e6e4ca0a47c8a77ceaa5309a47978a7c71bc5cce96366b5d7a569937c529eeda66c7293784a9402801af31");

    /// The identity of G1 (the point at infinity).
    ///
    /// See https://docs.rs/bls12_381/latest/bls12_381/notes/serialization/index.html for encoding info.
    const G1_IDENTITY: [u8; 48] = [
        0b11000000, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    /// The identity of G2 (the point at infinity).
    ///
    /// See https://docs.rs/bls12_381/latest/bls12_381/notes/serialization/index.html for encoding info.
    const G2_IDENTITY: [u8; 96] = [
        0b11000000, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

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

    /// This tests 1 == e(a, b) as there is no term on the left-hand side.
    /// This is true for `a` or `b` being the point at infinity. See
    /// https://eips.ethereum.org/EIPS/eip-2537#test-cases
    #[test]
    fn pairing_equality_works_for_empty_lhs() {
        // a and b not point at infinity (Non-degeneracy)
        let a = PK_LEO_MAINNET;
        let b = bls12_381_hash_to_g2(HashFunction::Sha256, b"blub", DOMAIN_HASH_TO_G2);
        let equal = bls12_381_pairing_equality(&[], &[], &a, &b).unwrap();
        assert!(!equal);

        // a point at infinity
        let a = G1_IDENTITY;
        let b = bls12_381_hash_to_g2(HashFunction::Sha256, b"blub", DOMAIN_HASH_TO_G2);
        let equal = bls12_381_pairing_equality(&[], &[], &a, &b).unwrap();
        assert!(equal);

        // b point at infinity
        let a = PK_LEO_MAINNET;
        let b = G2_IDENTITY;
        let equal = bls12_381_pairing_equality(&[], &[], &a, &b).unwrap();
        assert!(equal);

        // a and b point at infinity
        let a = G1_IDENTITY;
        let b = G2_IDENTITY;
        let equal = bls12_381_pairing_equality(&[], &[], &a, &b).unwrap();
        assert!(equal);
    }

    #[test]
    fn pairing_equality_error_cases_work() {
        let result = bls12_381_pairing_equality(&[12], &[0; 96], &[12], &[12]);
        assert!(matches!(
            result,
            Err(CryptoError::PairingEquality {
                source: PairingEqualityError::NotMultipleG1 { remainder: 1 },
                ..
            })
        ));

        let result = bls12_381_pairing_equality(&[0; 48], &[12], &[12], &[12]);
        assert!(matches!(
            result,
            Err(CryptoError::PairingEquality {
                source: PairingEqualityError::NotMultipleG2 { remainder: 1 },
                ..
            })
        ));

        let result = bls12_381_pairing_equality(&[0; 96], &[0; 96], &[12], &[12]);
        assert!(matches!(
            result,
            Err(CryptoError::PairingEquality {
                source: PairingEqualityError::UnequalPointAmount { left: 2, right: 1 },
                ..
            })
        ));
    }
}
