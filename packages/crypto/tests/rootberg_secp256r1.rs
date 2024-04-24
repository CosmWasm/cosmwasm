use cosmwasm_crypto::{secp256r1_recover_pubkey, secp256r1_verify};
use rootberg::*;

mod hashers;
mod rootberg;

// See ./testdata/rootberg/README.md for how to get/update those files
const SECP256R1_SHA256: &str = "./testdata/rootberg/ecdsa_secp256r1_sha_256_raw.json";
const SECP256R1_KECCAK256: &str = "./testdata/rootberg/ecdsa_secp256r1_keccak256_raw.json";

#[test]
fn rootberg_ecdsa_secp256r1_sha256() {
    let File { num_tests, tests } = read_file(SECP256R1_SHA256);
    assert_eq!(num_tests, tests.len(), "Invalid number of tests");
    assert!(num_tests >= 407, "Got unexpected number of tests");

    for test in tests {
        assert_eq!(test.public_key_uncompressed.len(), 65);

        // eprintln!("Test case ID: {}", test.tc_id);
        let message_hash = hashers::sha256(&test.msg);

        let signature = combine_signature(&test.sig);
        match secp256r1_verify(&message_hash, &signature, &test.public_key_uncompressed) {
            Ok(valid) => assert_eq!(test.valid, valid),
            Err(e) => {
                assert!(!test.valid, "expected valid signature, got {:?}", e);
            }
        }

        if test.valid {
            let recovered =
                secp256r1_recover_pubkey(&message_hash, &signature, test.sig.id).unwrap();
            assert_eq!(recovered, test.public_key_uncompressed);
        }
    }
}

#[test]
fn rootberg_ecdsa_secp256r1_keccak256() {
    let File { num_tests, tests } = read_file(SECP256R1_KECCAK256);
    assert_eq!(num_tests, tests.len(), "Invalid number of tests");
    assert!(num_tests >= 247, "Got unexpected number of tests");

    for test in tests {
        assert_eq!(test.public_key_uncompressed.len(), 65);

        // eprintln!("Test case ID: {}", test.tc_id);
        let message_hash = hashers::keccak_256(&test.msg);

        let signature = combine_signature(&test.sig);
        match secp256r1_verify(&message_hash, &signature, &test.public_key_uncompressed) {
            Ok(valid) => assert_eq!(test.valid, valid),
            Err(e) => {
                assert!(!test.valid, "expected valid signature, got {:?}", e);
            }
        }

        if test.valid {
            let recovered =
                secp256r1_recover_pubkey(&message_hash, &signature, test.sig.id).unwrap();
            assert_eq!(recovered, test.public_key_uncompressed);
        }
    }
}
