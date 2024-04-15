use bls12_381::{
    hash_to_curve::{ExpandMsgXmd, HashToCurve},
    G1Affine, G1Projective, G2Affine, G2Projective,
};
use sha2_v9::Sha256;

use crate::CryptoError;

#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum HashFunction {
    Sha256,
}

impl HashFunction {
    pub fn from_u32(idx: u32) -> Result<Self, CryptoError> {
        let hash = match idx {
            0 => Self::Sha256,
            _ => return Err(CryptoError::unknown_hash_function()),
        };

        Ok(hash)
    }

    pub fn to_u32(self) -> u32 {
        match self {
            Self::Sha256 => 0,
        }
    }
}

pub fn bls12_381_hash_to_g1(hash: HashFunction, msg: &[u8], dst: &[u8]) -> [u8; 48] {
    let g1 = match hash {
        HashFunction::Sha256 => {
            <G1Projective as HashToCurve<ExpandMsgXmd<Sha256>>>::hash_to_curve(msg, dst)
        }
    };

    G1Affine::from(g1).to_compressed()
}

pub fn bls12_381_hash_to_g2(hash: HashFunction, msg: &[u8], dst: &[u8]) -> [u8; 96] {
    let g2 = match hash {
        HashFunction::Sha256 => {
            <G2Projective as HashToCurve<ExpandMsgXmd<Sha256>>>::hash_to_curve(msg, dst)
        }
    };

    G2Affine::from(g2).to_compressed()
}
