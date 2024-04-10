#![allow(unused)]

use alloc::vec::Vec;
use core::ops::Add;
use core::{fmt, ops::Neg};

use bls12_381::hash_to_curve::ExpandMsgXmd;
use bls12_381::{
    hash_to_curve::{ExpandMessage, HashToCurve},
    G1Affine, G1Projective, G2Affine, G2Projective,
};
use pairing::group::Group;
use sha2_v9::Sha256;

/// Point on G1
#[derive(Debug, PartialEq, Clone)]
pub struct G1(pub(crate) G1Affine);

impl G1 {
    /// Creates the generaor in G1
    #[inline]
    pub fn generator() -> Self {
        Self(G1Affine::generator())
    }

    /// Creates the identity element in G1 (point at infinity)
    #[inline]
    pub fn identity() -> Self {
        Self(G1Affine::identity())
    }

    /// Check if the point is the identity element
    #[inline]
    pub fn is_identity(&self) -> bool {
        self.0.is_identity().into()
    }

    #[inline]
    pub fn from_uncompressed(data: &[u8; 96]) -> Option<Self> {
        G1Affine::from_uncompressed(data).map(Self).into()
    }

    #[inline]
    pub fn to_uncompressed(&self) -> [u8; 96] {
        self.0.to_uncompressed()
    }

    #[inline]
    pub fn to_compressed(&self) -> [u8; 48] {
        self.0.to_compressed()
    }
}

impl Add<G1> for G1 {
    type Output = G1;

    fn add(self, rhs: Self) -> Self {
        let sum = self.0 + G1Projective::from(rhs.0);
        Self(sum.into())
    }
}

impl Add<&G1> for G1 {
    type Output = G1;

    fn add(self, rhs: &G1) -> G1 {
        let sum = self.0 + G1Projective::from(rhs.0);
        G1(sum.into())
    }
}

impl Add<&G1> for &G1 {
    type Output = G1;

    fn add(self, rhs: &G1) -> G1 {
        let sum = self.0 + G1Projective::from(rhs.0);
        G1(sum.into())
    }
}

impl Neg for G1 {
    type Output = G1;

    fn neg(self) -> Self::Output {
        G1(-self.0)
    }
}

impl Neg for &G1 {
    type Output = G1;

    fn neg(self) -> Self::Output {
        G1(-self.0)
    }
}

impl core::iter::Sum<G1> for G1 {
    fn sum<I: Iterator<Item = G1>>(iter: I) -> Self {
        let zero = G1Projective::identity();
        let sum = iter.fold(zero, |acc, next| acc + G1Projective::from(next.0));
        G1(sum.into())
    }
}

impl<'a> core::iter::Sum<&'a G1> for G1 {
    fn sum<I: Iterator<Item = &'a G1>>(iter: I) -> Self {
        let zero = G1Projective::identity();
        let sum = iter.fold(zero, |acc, next| acc + G1Projective::from(next.0));
        G1(sum.into())
    }
}

/// Point on G2
#[derive(Debug, PartialEq, Clone)]
pub struct G2(pub(crate) G2Affine);

impl G2 {
    /// Creates the generaor in G2
    #[inline]
    pub fn generator() -> Self {
        Self(G2Affine::generator())
    }

    /// Creates the identity element in G2 (point at infinity)
    #[inline]
    pub fn identity() -> Self {
        Self(G2Affine::identity())
    }

    /// Check if the point is the identity element
    #[inline]
    pub fn is_identity(&self) -> bool {
        self.0.is_identity().into()
    }

    #[inline]
    pub fn from_uncompressed(data: &[u8; 192]) -> Option<Self> {
        G2Affine::from_uncompressed(data).map(Self).into()
    }

    #[inline]
    pub fn to_uncompressed(&self) -> [u8; 192] {
        self.0.to_uncompressed()
    }

    #[inline]
    pub fn to_compressed(&self) -> [u8; 96] {
        self.0.to_compressed()
    }
}

impl Add<&G2> for &G2 {
    type Output = G2;

    fn add(self, rhs: &G2) -> Self::Output {
        [self, rhs].into_iter().sum()
    }
}

impl core::iter::Sum<G2> for G2 {
    fn sum<I: Iterator<Item = G2>>(iter: I) -> Self {
        let zero = G2Projective::identity();
        let sum = iter.fold(zero, |acc, next| acc + G2Projective::from(next.0));
        G2(sum.into())
    }
}

impl<'a> core::iter::Sum<&'a G2> for G2 {
    fn sum<I: Iterator<Item = &'a G2>>(iter: I) -> Self {
        let zero = G2Projective::identity();
        let sum = iter.fold(zero, |acc, next| acc + G2Projective::from(next.0));
        G2(sum.into())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InvalidPoint {
    InvalidLength { expected: usize, actual: usize },
    DecodingError {},
}

impl fmt::Display for InvalidPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidPoint::InvalidLength { expected, actual } => {
                write!(f, "Invalid input length for point (must be in compressed format): Expected {}, actual: {}", expected, actual)
            }
            InvalidPoint::DecodingError {} => {
                write!(f, "Invalid point")
            }
        }
    }
}

pub fn g1_from_variable(data: &[u8]) -> Result<G1, InvalidPoint> {
    if data.len() != 48 {
        return Err(InvalidPoint::InvalidLength {
            expected: 48,
            actual: data.len(),
        });
    }

    let mut buf = [0u8; 48];
    buf[..].clone_from_slice(data);
    g1_from_fixed(&buf)
}

pub fn g1s_from_variable(data_list: &[&[u8]]) -> Vec<Result<G1, InvalidPoint>> {
    use rayon::prelude::*;
    let mut out = Vec::with_capacity(data_list.len());
    data_list
        .par_iter()
        .map(|&data| g1_from_variable(data))
        .collect_into_vec(&mut out);
    out
}

pub fn g2_from_variable(data: &[u8]) -> Result<G2, InvalidPoint> {
    if data.len() != 96 {
        return Err(InvalidPoint::InvalidLength {
            expected: 96,
            actual: data.len(),
        });
    }

    let mut buf = [0u8; 96];
    buf[..].clone_from_slice(data);
    g2_from_fixed(&buf)
}

pub fn g1_from_fixed(data: &[u8; 48]) -> Result<G1, InvalidPoint> {
    Option::from(G1Affine::from_compressed(data))
        .map(G1)
        .ok_or(InvalidPoint::DecodingError {})
}

/// Like [`g1_from_fixed`] without guaranteeing that the encoding represents a valid element.
/// Only use this when you know for sure the encoding is correct.
pub fn g1_from_fixed_unchecked(data: [u8; 48]) -> Result<G1, InvalidPoint> {
    Option::from(G1Affine::from_compressed_unchecked(&data))
        .map(G1)
        .ok_or(InvalidPoint::DecodingError {})
}

pub fn g2_from_fixed(data: &[u8; 96]) -> Result<G2, InvalidPoint> {
    Option::from(G2Affine::from_compressed(data))
        .map(G2)
        .ok_or(InvalidPoint::DecodingError {})
}

/// Like [`g2_from_fixed`] without guaranteeing that the encoding represents a valid element.
/// Only use this when you know for sure the encoding is correct.
pub fn g2_from_fixed_unchecked(data: [u8; 96]) -> Result<G2, InvalidPoint> {
    Option::from(G2Affine::from_compressed_unchecked(&data))
        .map(G2)
        .ok_or(InvalidPoint::DecodingError {})
}

pub fn bls12_381_g1_is_identity(g1: &[u8; 48]) -> Result<bool, InvalidPoint> {
    g1_from_fixed(g1).map(|point| point.is_identity())
}

pub fn bls12_381_g2_is_identity(g2: &[u8; 96]) -> Result<bool, InvalidPoint> {
    g2_from_fixed(g2).map(|point| point.is_identity())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn g1_generator_works() {
        let _gen = G1::generator();
    }

    #[test]
    fn g2_generator_works() {
        let _gen = G2::generator();
    }

    #[test]
    fn g1_from_variable_works() {
        let result = g1_from_variable(&hex::decode("868f005eb8e6e4ca0a47c8a77ceaa5309a47978a7c71bc5cce96366b5d7a569937c529eeda66c7293784a9402801af31").unwrap());
        assert!(result.is_ok());

        let result = g1_from_variable(&hex::decode("868f005eb8e6e4ca0a47c8a77ceaa5309a47978a7c71bc5cce96366b5d7a569937c529eeda66c7293784a9402801af").unwrap());
        match result.unwrap_err() {
            InvalidPoint::InvalidLength { expected, actual } => {
                assert_eq!(expected, 48);
                assert_eq!(actual, 47);
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn g2_from_variable_works() {
        let result = g2_from_variable(&hex::decode("82f5d3d2de4db19d40a6980e8aa37842a0e55d1df06bd68bddc8d60002e8e959eb9cfa368b3c1b77d18f02a54fe047b80f0989315f83b12a74fd8679c4f12aae86eaf6ab5690b34f1fddd50ee3cc6f6cdf59e95526d5a5d82aaa84fa6f181e42").unwrap());
        assert!(result.is_ok());

        let result = g2_from_variable(&hex::decode("82f5d3d2de4db19d40a6980e8aa37842a0e55d1df06bd68bddc8d60002e8e959eb9cfa368b3c1b77d18f02a54fe047b80f0989315f83b12a74fd8679c4f12aae86eaf6ab5690b34f1fddd50ee3cc6f6cdf59e95526d5a5d82aaa84fa6f181e").unwrap());
        match result.unwrap_err() {
            InvalidPoint::InvalidLength { expected, actual } => {
                assert_eq!(expected, 96);
                assert_eq!(actual, 95);
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn g1_from_fixed_works() {
        let result = g1_from_fixed(&hex_literal::hex!("868f005eb8e6e4ca0a47c8a77ceaa5309a47978a7c71bc5cce96366b5d7a569937c529eeda66c7293784a9402801af31"));
        assert!(result.is_ok());

        let result = g1_from_fixed(&hex_literal::hex!("118f005eb8e6e4ca0a47c8a77ceaa5309a47978a7c71bc5cce96366b5d7a569937c529eeda66c7293784a9402801af31"));
        match result.unwrap_err() {
            InvalidPoint::DecodingError {} => {}
            err => panic!("Unexpected error: {:?}", err),
        }

        let result = g1_from_fixed(&hex_literal::hex!("868f005eb8e6e4ca0a47c8a77ceaa5309a47978a7c71bc5cce96366b5d7a569937c529eeda66c7293784a9402801af22"));
        match result.unwrap_err() {
            InvalidPoint::DecodingError {} => {}
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn g1_from_fixed_unchecked_works() {
        let data = hex!("868f005eb8e6e4ca0a47c8a77ceaa5309a47978a7c71bc5cce96366b5d7a569937c529eeda66c7293784a9402801af31");
        let a = g1_from_fixed_unchecked(data).unwrap();
        let b = g1_from_fixed(&data).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn g2_from_fixed_works() {
        let result = g2_from_fixed(&hex_literal::hex!("82f5d3d2de4db19d40a6980e8aa37842a0e55d1df06bd68bddc8d60002e8e959eb9cfa368b3c1b77d18f02a54fe047b80f0989315f83b12a74fd8679c4f12aae86eaf6ab5690b34f1fddd50ee3cc6f6cdf59e95526d5a5d82aaa84fa6f181e42"));
        assert!(result.is_ok());

        let result = g2_from_fixed(&hex_literal::hex!("11f5d3d2de4db19d40a6980e8aa37842a0e55d1df06bd68bddc8d60002e8e959eb9cfa368b3c1b77d18f02a54fe047b80f0989315f83b12a74fd8679c4f12aae86eaf6ab5690b34f1fddd50ee3cc6f6cdf59e95526d5a5d82aaa84fa6f181e42"));
        match result.unwrap_err() {
            InvalidPoint::DecodingError {} => {}
            err => panic!("Unexpected error: {:?}", err),
        }

        let result = g2_from_fixed(&hex_literal::hex!("82f5d3d2de4db19d40a6980e8aa37842a0e55d1df06bd68bddc8d60002e8e959eb9cfa368b3c1b77d18f02a54fe047b80f0989315f83b12a74fd8679c4f12aae86eaf6ab5690b34f1fddd50ee3cc6f6cdf59e95526d5a5d82aaa84fa6f181e44"));
        match result.unwrap_err() {
            InvalidPoint::DecodingError {} => {}
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn g2_from_fixed_unchecked_works() {
        let data = hex!("82f5d3d2de4db19d40a6980e8aa37842a0e55d1df06bd68bddc8d60002e8e959eb9cfa368b3c1b77d18f02a54fe047b80f0989315f83b12a74fd8679c4f12aae86eaf6ab5690b34f1fddd50ee3cc6f6cdf59e95526d5a5d82aaa84fa6f181e42");
        let a = g2_from_fixed_unchecked(data).unwrap();
        let b = g2_from_fixed(&data).unwrap();
        assert_eq!(a, b);
    }
}
