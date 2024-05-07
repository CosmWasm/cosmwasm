use cosmwasm_std::{Api, HashFunction, StdResult, BLS12_381_G1_GENERATOR, BLS12_381_G2_GENERATOR};

pub fn verify_g1(
    api: &dyn Api,
    signature: &[u8],
    pubkey: &[u8],
    msg: &[u8],
    dst: &[u8],
) -> StdResult<bool> {
    let s = api.bls12_381_hash_to_g2(HashFunction::Sha256, msg, dst)?;
    api.bls12_381_pairing_equality(&BLS12_381_G1_GENERATOR, signature, pubkey, &s)
        .map_err(Into::into)
}

pub fn verify_g2(
    api: &dyn Api,
    signature: &[u8],
    pubkey: &[u8],
    msg: &[u8],
    dst: &[u8],
) -> StdResult<bool> {
    let s = api.bls12_381_hash_to_g1(HashFunction::Sha256, msg, dst)?;
    api.bls12_381_pairing_equality(signature, &BLS12_381_G2_GENERATOR, &s, pubkey)
        .map_err(Into::into)
}
