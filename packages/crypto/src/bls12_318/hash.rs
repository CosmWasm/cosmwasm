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
    Sha256,
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
