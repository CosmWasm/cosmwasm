use cosmwasm_std::{Api, HashFunction, StdResult};

pub fn verify(
    api: &dyn Api,
    p: &[u8],
    q: &[u8],
    r: &[u8],
    msg: &[u8],
    dst: &[u8],
) -> StdResult<bool> {
    let s = api.bls12_381_hash_to_g2(HashFunction::Sha256, msg, dst)?;
    api.bls12_381_aggregate_pairing_equality(p, q, r, &s)
        .map_err(Into::into)
}
