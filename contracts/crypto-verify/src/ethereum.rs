use std::borrow::Cow;

use ethereum_transaction::{SignedTransaction, Transaction};

pub fn serialize_unsigned_transaction(
    from: [u8; 20],
    to: [u8; 20],
    nonce: u128,
    gas: u128,
    gas_price: u128,
    value: u128,
    data: Vec<u8>,
    chain_id: u64,
) -> Vec<u8> {
    let unsigned = Transaction {
        from: from.into(),
        to: Some(to.into()),
        nonce: nonce.into(),
        gas: gas.into(),
        gas_price: gas_price.into(),
        value: value.into(),
        data: data.into(),
    };

    SignedTransaction {
        transaction: Cow::Owned(unsigned),
        v: chain_id,
        r: 0.into(),
        s: 0.into(),
    }
    .to_rlp()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn serialize_unsigned_transaction_works() {
        // Test data from https://github.com/iov-one/iov-core/blob/v2.5.0/packages/iov-ethereum/src/serialization.spec.ts#L78-L93
        let nonce = 26;
        let chain_id = 5777;
        let from = hex!("9d8a62f656a8d1615c1294fd71e9cfb3e4855a4f");
        let to = hex!("43aa18faae961c23715735682dc75662d90f4dde");
        let gas_limit = 21000;
        let gas_price = 20000000000;
        let value = 20000000000000000000;
        let data = Vec::default();
        let bytes_to_sign = serialize_unsigned_transaction(
            from, to, nonce, gas_limit, gas_price, value, data, chain_id,
        );
        assert_eq!(hex::encode(bytes_to_sign), "ef1a8504a817c8008252089443aa18faae961c23715735682dc75662d90f4dde8901158e460913d00000808216918080");
    }
}
