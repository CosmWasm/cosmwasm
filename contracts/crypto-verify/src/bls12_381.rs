use cosmwasm_std::{Api, HashFunction, StdResult, BLS12_381_G1_GENERATOR, BLS12_381_G2_GENERATOR};

/// Signature verification with public key in G1 (e.g. drand classic mainnet, ETH2 block headers).
///
/// See https://hackmd.io/@benjaminion/bls12-381#Verification.
pub fn verify_g1(
    api: &dyn Api,
    signature: &[u8],
    pubkey: &[u8],
    msg: &[u8],
    dst: &[u8],
) -> StdResult<bool> {
    // The H(m) from the docs
    let msg_hash = api.bls12_381_hash_to_g2(HashFunction::Sha256, msg, dst)?;
    api.bls12_381_pairing_equality(&BLS12_381_G1_GENERATOR, signature, pubkey, &msg_hash)
        .map_err(Into::into)
}

/// Signature verification with public key in G2 (e.g. drand Quicknet)
///
/// See https://hackmd.io/@benjaminion/bls12-381#Verification in combination with
/// https://hackmd.io/@benjaminion/bls12-381#Swapping-G1-and-G2.
pub fn verify_g2(
    api: &dyn Api,
    signature: &[u8],
    pubkey: &[u8],
    msg: &[u8],
    dst: &[u8],
) -> StdResult<bool> {
    // The H(m) from the docs
    let msg_hash = api.bls12_381_hash_to_g1(HashFunction::Sha256, msg, dst)?;
    api.bls12_381_pairing_equality(signature, &BLS12_381_G2_GENERATOR, &msg_hash, pubkey)
        .map_err(Into::into)
}
