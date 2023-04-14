use cosmwasm_std::{Api, StdError, StdResult};
use rlp::RlpStream;
use sha3::{Digest, Keccak256};

#[allow(clippy::too_many_arguments)]
pub fn verify_transaction(
    api: &dyn Api,
    from: [u8; 20],
    to: [u8; 20],
    nonce: u64,
    gas: u128,
    gas_price: u128,
    value: u128,
    data: &[u8],
    chain_id: u64,
    r: &[u8],
    s: &[u8],
    v: u64,
) -> StdResult<bool> {
    let sign_bytes =
        serialize_unsigned_transaction(to, nonce, gas, gas_price, value, data, chain_id);
    let hash = Keccak256::digest(sign_bytes);
    let mut rs: Vec<u8> = Vec::with_capacity(64);
    rs.resize(32 - r.len(), 0); // Left pad r to 32 bytes
    rs.extend_from_slice(r);
    rs.resize(32 + (32 - s.len()), 0); // Left pad s to 32 bytes
    rs.extend_from_slice(s);

    let recovery = get_recovery_param_with_chain_id(v, chain_id)?;
    let calculated_pubkey = api.secp256k1_recover_pubkey(&hash, &rs, recovery)?;
    let calculated_address = ethereum_address_raw(&calculated_pubkey)?;
    if from != calculated_address {
        return Ok(false);
    }
    let valid = api.secp256k1_verify(&hash, &rs, &calculated_pubkey)?;
    Ok(valid)
}

fn serialize_unsigned_transaction(
    to: [u8; 20],
    nonce: u64,
    gas_limit: u128,
    gas_price: u128,
    value: u128,
    data: &[u8],
    chain_id: u64,
) -> Vec<u8> {
    // See https://ethereum.stackexchange.com/a/2097/54581 and
    // https://github.com/tomusdrw/jsonrpc-proxy/blob/7855dec/ethereum-proxy/plugins/accounts/transaction/src/lib.rs#L132-L144.
    let mut stream = RlpStream::new();
    stream.begin_list(9);
    stream.append(&nonce);
    stream.append(&gas_price);
    stream.append(&gas_limit);
    stream.append(&to.as_ref());
    stream.append(&value);
    stream.append(&data);
    stream.append(&chain_id);
    stream.append(&Vec::<u8>::new()); // empty r
    stream.append(&Vec::<u8>::new()); // empty s
    stream.out().to_vec()
}

/// Get the recovery param from the value `v` when no chain ID for replay protection is used.
///
/// This is needed for chain-agnostig aignatures like signed text.
///
/// See [EIP-155] for how `v` is composed.
///
/// [EIP-155]: https://github.com/ethereum/EIPs/blob/master/EIPS/eip-155.md
pub fn get_recovery_param(v: u8) -> StdResult<u8> {
    match v {
        27 => Ok(0),
        28 => Ok(1),
        _ => Err(StdError::generic_err("Values of v other than 27 and 28 not supported. Replay protection (EIP-155) cannot be used here."))
    }
}

/// Get the recovery param from the value `v` when a chain ID for replay protection is used.
///
/// This is needed for chain-agnostig aignatures like signed text.
///
/// See [EIP-155] for how `v` is composed.
///
/// [EIP-155]: https://github.com/ethereum/EIPs/blob/master/EIPS/eip-155.md
pub fn get_recovery_param_with_chain_id(v: u64, chain_id: u64) -> StdResult<u8> {
    let recovery = v - chain_id * 2 - 35;
    match recovery {
        0 | 1 => Ok(recovery as u8),
        _ => Err(StdError::generic_err(format!(
            "Calculated recovery parameter must be 0 or 1 but is {}.",
            recovery
        ))),
    }
}

/// Returns a raw 20 byte Ethereum address
pub fn ethereum_address_raw(pubkey: &[u8]) -> StdResult<[u8; 20]> {
    let (tag, data) = match pubkey.split_first() {
        Some(pair) => pair,
        None => return Err(StdError::generic_err("Public key must not be empty")),
    };
    if *tag != 0x04 {
        return Err(StdError::generic_err("Public key must start with 0x04"));
    }
    if data.len() != 64 {
        return Err(StdError::generic_err("Public key must be 65 bytes long"));
    }

    let hash = Keccak256::digest(data);
    Ok(hash[hash.len() - 20..].try_into().unwrap())
}

pub fn decode_address(input: &str) -> StdResult<[u8; 20]> {
    if input.len() != 42 {
        return Err(StdError::generic_err(
            "Ethereum address must be 42 characters long",
        ));
    }
    if !input.starts_with("0x") {
        return Err(StdError::generic_err("Ethereum address must start wit 0x"));
    }
    let data = hex::decode(&input[2..]).map_err(|_| StdError::generic_err("hex decoding error"))?;
    Ok(data.try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::MockApi;
    use hex_literal::hex;

    #[test]
    fn verify_transaction_works() {
        // curl -sS -X POST --data '{"jsonrpc":"2.0","method":"eth_getTransactionByHash","params":["0x3b87faa3410f33284124a6898fac1001673f0f7c3682d18f55bdff0031cce9ce"],"id":1}' -H "Content-type: application/json" https://rinkeby-light.eth.linkpool.io | jq .result
        // {
        //   "blockHash": "0x05ebd1bd99956537f49cfa1104682b3b3f9ff9249fa41a09931ce93368606c21",
        //   "blockNumber": "0x37ef3e",
        //   "from": "0x0a65766695a712af41b5cfecaad217b1a11cb22a",
        //   "gas": "0x226c8",
        //   "gasPrice": "0x3b9aca00",
        //   "hash": "0x3b87faa3410f33284124a6898fac1001673f0f7c3682d18f55bdff0031cce9ce",
        //   "input": "0x536561726368207478207465737420302e36353930383639313733393634333335",
        //   "nonce": "0xe1",
        //   "to": "0xe137f5264b6b528244e1643a2d570b37660b7f14",
        //   "transactionIndex": "0xb",
        //   "value": "0x53177c",
        //   "v": "0x2b",
        //   "r": "0xb9299dab50b3cddcaecd64b29bfbd5cd30fac1a1adea1b359a13c4e5171492a6",
        //   "s": "0x573059c66d894684488f92e7ce1f91b158ca57b0235485625b576a3b98c480ac"
        // }
        let nonce = 0xe1;
        let chain_id = 4; // Rinkeby, see https://github.com/ethereum/EIPs/blob/master/EIPS/eip-155.md#list-of-chain-ids
        let from = hex!("0a65766695a712af41b5cfecaad217b1a11cb22a");
        let to = hex!("e137f5264b6b528244e1643a2d570b37660b7f14");
        let gas_limit = 0x226c8;
        let gas_price = 0x3b9aca00;
        let value = 0x53177c;
        let data = hex!("536561726368207478207465737420302e36353930383639313733393634333335");
        let r = hex!("b9299dab50b3cddcaecd64b29bfbd5cd30fac1a1adea1b359a13c4e5171492a6");
        let s = hex!("573059c66d894684488f92e7ce1f91b158ca57b0235485625b576a3b98c480ac");
        let v = 0x2b;

        let api = MockApi::default();
        let valid = verify_transaction(
            &api, from, to, nonce, gas_limit, gas_price, value, &data, chain_id, &r, &s, v,
        )
        .unwrap();
        assert!(valid);
    }

    #[test]
    fn serialize_unsigned_transaction_works() {
        // Test data from https://github.com/iov-one/iov-core/blob/v2.5.0/packages/iov-ethereum/src/serialization.spec.ts#L78-L93
        let nonce = 26;
        let chain_id = 5777;
        let _from = hex!("9d8a62f656a8d1615c1294fd71e9cfb3e4855a4f");
        let to = hex!("43aa18faae961c23715735682dc75662d90f4dde");
        let gas_limit = 21000;
        let gas_price = 20000000000;
        let value = 20000000000000000000;
        let data = Vec::default();
        let bytes_to_sign =
            serialize_unsigned_transaction(to, nonce, gas_limit, gas_price, value, &data, chain_id);
        assert_eq!(hex::encode(bytes_to_sign), "ef1a8504a817c8008252089443aa18faae961c23715735682dc75662d90f4dde8901158e460913d00000808216918080");
    }
}
