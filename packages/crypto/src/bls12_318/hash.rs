//!
//! Note about the usage of `.unwrap()` here:
//!
//! Since the underlying curve implementation, when implemented sanely, should never request 255 curve elements at the same time,
//! the expansion will always finish without exiting with an error (since that is the only "ABORT" condition).
//!
//! Therefore we can conclude, if the implementation is done as defined in the IETF publication, won't ever error out.
//!
//! IETF doc in question: <https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-hash-to-curve-12#section-5.4.1>
//!
//! In addition to that I (@aumetra) skimmed through the tree of traits making up our hash-to-curve configuration,
//! and I have not found a condition where an error is returned.
//!
//! ark crate versions that I looked at:
//!
//! - ark-bls12-381 v0.4.0
//! - ark-ec v0.4.2
//! - ark-ff v0.4.2
//!

use ark_bls12_381::{g1, g2};
use ark_ec::{
    hashing::{
        curve_maps::wb::WBMap, map_to_curve_hasher::MapToCurveBasedHasher, HashToCurve as _,
    },
    short_weierstrass::Projective,
};
use ark_ff::field_hashers::DefaultFieldHasher;
use ark_serialize::CanonicalSerialize;
use sha2::Sha256;

use crate::{CryptoError, BLS12_381_G1_POINT_LEN, BLS12_381_G2_POINT_LEN};

type HashToCurve<CurveConfig, Hash> =
    MapToCurveBasedHasher<Projective<CurveConfig>, DefaultFieldHasher<Hash>, WBMap<CurveConfig>>;

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum HashFunction {
    Sha256 = 0,
}

#[doc(hidden)]
impl HashFunction {
    pub fn from_u32(idx: u32) -> Result<Self, CryptoError> {
        let hash = match idx {
            0 => Self::Sha256,
            _ => return Err(CryptoError::unknown_hash_function()),
        };

        Ok(hash)
    }
}

pub fn bls12_381_hash_to_g1(
    hash: HashFunction,
    msg: &[u8],
    dst: &[u8],
) -> [u8; BLS12_381_G1_POINT_LEN] {
    let point = match hash {
        HashFunction::Sha256 => HashToCurve::<g1::Config, Sha256>::new(dst)
            .unwrap()
            .hash(msg)
            .unwrap(),
    };

    let mut serialized = [0; BLS12_381_G1_POINT_LEN];
    point.serialize_compressed(&mut serialized[..]).unwrap();
    serialized
}

pub fn bls12_381_hash_to_g2(
    hash: HashFunction,
    msg: &[u8],
    dst: &[u8],
) -> [u8; BLS12_381_G2_POINT_LEN] {
    let point = match hash {
        HashFunction::Sha256 => HashToCurve::<g2::Config, Sha256>::new(dst)
            .unwrap()
            .hash(msg)
            .unwrap(),
    };

    let mut serialized = [0; BLS12_381_G2_POINT_LEN];
    point.serialize_compressed(&mut serialized[..]).unwrap();
    serialized
}

#[cfg(test)]
mod test {
    use hex_literal::hex;

    use crate::{bls12_381_hash_to_g1, bls12_381_hash_to_g2, HashFunction};

    #[test]
    fn hash_to_g1_works() {
        // See: <https://datatracker.ietf.org/doc/rfc9380/>; Section J.9.1

        let msg = b"abc";
        let dst = b"QUUX-V01-CS02-with-BLS12381G1_XMD:SHA-256_SSWU_RO_";

        let hashed_point = bls12_381_hash_to_g1(HashFunction::Sha256, msg, dst);
        let mut serialized_expected_compressed = hex!("03567bc5ef9c690c2ab2ecdf6a96ef1c139cc0b2f284dca0a9a7943388a49a3aee664ba5379a7655d3c68900be2f6903");
        // Set the compression tag
        serialized_expected_compressed[0] |= 0b1000_0000;

        assert_eq!(hashed_point, serialized_expected_compressed);
    }

    #[test]
    fn hash_to_g2_works() {
        let msg = b"abc";
        let dst = b"QUUX-V01-CS02-with-BLS12381G2_XMD:SHA-256_SSWU_RO_";

        let hashed_point = bls12_381_hash_to_g2(HashFunction::Sha256, msg, dst);
        let mut serialized_expected_compressed = hex!("139cddbccdc5e91b9623efd38c49f81a6f83f175e80b06fc374de9eb4b41dfe4ca3a230ed250fbe3a2acf73a41177fd802c2d18e033b960562aae3cab37a27ce00d80ccd5ba4b7fe0e7a210245129dbec7780ccc7954725f4168aff2787776e6");
        // Set the compression tag
        serialized_expected_compressed[0] |= 0b1000_0000;

        assert_eq!(hashed_point, serialized_expected_compressed);
    }
}
