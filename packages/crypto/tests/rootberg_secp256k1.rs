use cosmwasm_crypto::{secp256k1_recover_pubkey, secp256k1_verify};
use serde::Deserialize;

// See ./testdata/rootberg/README.md for how to get/update those files
const SECP256K1_SHA256: &str = "./testdata/rootberg/ecdsa_secp256k1_sha_256_raw.json";
const SECP256K1_KECCAK256: &str = "./testdata/rootberg/ecdsa_secp256k1_keccak256_raw.json";

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub num_tests: usize,
    pub tests: Vec<Test>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Test {
    pub tc_id: i64,
    #[serde(deserialize_with = "hex::deserialize")]
    pub public_key_uncompressed: Vec<u8>,
    #[serde(deserialize_with = "hex::deserialize")]
    pub msg: Vec<u8>,
    pub sig: Sig,
    pub comment: String,
    pub valid: bool,
    pub flags: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sig {
    #[serde(deserialize_with = "hex::deserialize")]
    pub r: Vec<u8>,
    #[serde(deserialize_with = "hex::deserialize")]
    pub s: Vec<u8>,
    pub id: u8,
}

fn read_file(path: &str) -> File {
    use std::fs::File;
    use std::io::BufReader;

    // Open the file in read-only mode with buffer.
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    serde_json::from_reader(reader).unwrap()
}

mod hashers {
    use digest::Digest;
    use sha2::Sha256;
    use sha3::Keccak256;

    pub fn sha256(data: &[u8]) -> [u8; 32] {
        Sha256::digest(data).into()
    }

    pub fn keccak_256(data: &[u8]) -> [u8; 32] {
        Keccak256::digest(data).into()
    }
}

#[test]
fn rootberg_ecdsa_secp256k1_sha256() {
    let File { num_tests, tests } = read_file(SECP256K1_SHA256);
    assert_eq!(num_tests, tests.len(), "Invalid number of tests");
    assert!(num_tests >= 423, "Got unexpected number of tests");

    for test in tests {
        assert_eq!(test.public_key_uncompressed.len(), 65);

        // eprintln!("Test case ID: {}", test.tc_id);
        let message_hash = hashers::sha256(&test.msg);

        let signature = combine_signature(&test.sig);
        match secp256k1_verify(&message_hash, &signature, &test.public_key_uncompressed) {
            Ok(valid) => assert_eq!(test.valid, valid),
            Err(e) => {
                assert!(!test.valid, "expected valid signature, got {:?}", e);
            }
        }

        if test.valid {
            let recovered =
                secp256k1_recover_pubkey(&message_hash, &signature, test.sig.id).unwrap();
            assert_eq!(recovered, test.public_key_uncompressed);
        }
    }
}

#[test]
fn rootberg_ecdsa_secp256k1_keccak256() {
    let File { num_tests, tests } = read_file(SECP256K1_KECCAK256);
    assert_eq!(num_tests, tests.len(), "Invalid number of tests");
    assert!(num_tests >= 263, "Got unexpected number of tests");

    for test in tests {
        assert_eq!(test.public_key_uncompressed.len(), 65);

        // eprintln!("Test case ID: {}", test.tc_id);
        let message_hash = hashers::keccak_256(&test.msg);

        let signature = combine_signature(&test.sig);
        match secp256k1_verify(&message_hash, &signature, &test.public_key_uncompressed) {
            Ok(valid) => assert_eq!(test.valid, valid),
            Err(e) => {
                assert!(!test.valid, "expected valid signature, got {:?}", e);
            }
        }

        if test.valid {
            let recovered =
                secp256k1_recover_pubkey(&message_hash, &signature, test.sig.id).unwrap();
            assert_eq!(recovered, test.public_key_uncompressed);
        }
    }
}

fn combine_signature(sig: &Sig) -> Vec<u8> {
    // the test data contains values with leading zeroes, which we need to ignore
    let first_non_zero = sig.r.iter().position(|&v| v != 0).unwrap_or_default();
    let r = &sig.r[first_non_zero..];
    let first_non_zero = sig.s.iter().position(|&v| v != 0).unwrap_or_default();
    let s = &sig.s[first_non_zero..];

    // at least one of the tests has an s that is 33 bytes long
    let r_len = r.len().max(32);
    let s_len = s.len().max(32);

    // the test data also contains values with less than 32 bytes, so we need to pad them with zeroes
    let mut signature = vec![0; r_len + s_len];
    let (r_part, s_part) = signature.split_at_mut(r_len);
    r_part[r_len - r.len()..].copy_from_slice(r);
    s_part[s_len - s.len()..].copy_from_slice(s);

    signature
}
