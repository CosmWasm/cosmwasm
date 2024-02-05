#![allow(clippy::single_match)] // Only needed for old clippy (e.g. 1.70.0)

use cosmwasm_crypto::{secp256k1_recover_pubkey, secp256k1_verify};
use serde::Deserialize;

// See ./testdata/wycheproof/README.md for how to get/update those files
const SECP256K1_SHA256: &str = "./testdata/wycheproof/ecdsa_secp256k1_sha256_test.json";
const SECP256K1_SHA512: &str = "./testdata/wycheproof/ecdsa_secp256k1_sha512_test.json";
const SECP256K1_SHA3_256: &str = "./testdata/wycheproof/ecdsa_secp256k1_sha3_256_test.json";
const SECP256K1_SHA3_512: &str = "./testdata/wycheproof/ecdsa_secp256k1_sha3_512_test.json";

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct File {
    number_of_tests: usize,
    test_groups: Vec<TestGroup>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TestGroup {
    public_key: Key,
    tests: Vec<TestCase>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Key {
    uncompressed: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TestCase {
    tc_id: u32,
    comment: String,
    msg: String,
    sig: String,
    // "acceptable", "valid" or "invalid"
    result: String,
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
    use sha2::{Digest, Sha256, Sha512};
    use sha3::{Sha3_256, Sha3_512};

    pub fn sha256(data: &[u8]) -> [u8; 32] {
        Sha256::digest(data).into()
    }

    // ecdsa_secp256k1_sha512 requires truncating to 32 bytes
    pub fn sha512(data: &[u8]) -> [u8; 32] {
        let hash = Sha512::digest(data).to_vec();
        hash[..32].try_into().unwrap()
    }

    pub fn sha3_256(data: &[u8]) -> [u8; 32] {
        Sha3_256::digest(data).into()
    }

    // ecdsa_secp256k1_sha3_512 requires truncating to 32 bytes
    pub fn sha3_512(data: &[u8]) -> [u8; 32] {
        let hash = Sha3_512::digest(data).to_vec();
        hash[..32].try_into().unwrap()
    }
}

#[test]
fn ecdsa_secp256k1_sha256() {
    let mut tested: usize = 0;
    let File {
        number_of_tests,
        test_groups,
    } = read_file(SECP256K1_SHA256);
    assert!(number_of_tests >= 463, "Got unexpected number of tests");

    for group in test_groups {
        let public_key = hex::decode(group.public_key.uncompressed).unwrap();
        assert_eq!(public_key.len(), 65);

        for tc in group.tests {
            tested += 1;
            assert_eq!(tc.tc_id as usize, tested);
            // eprintln!("Test case ID: {}", tc.tc_id);
            let message = hex::decode(tc.msg).unwrap();
            let message_hash = hashers::sha256(&message);
            let der_signature = hex::decode(tc.sig).unwrap();

            match tc.result.as_str() {
                "valid" | "acceptable" => {
                    let signature = from_der(&der_signature).unwrap();
                    let valid = secp256k1_verify(&message_hash, &signature, &public_key).unwrap();
                    assert!(valid);
                    if tc.comment != "k*G has a large x-coordinate" {
                        test_secp256k1_recover_pubkey(&message_hash, &signature, &public_key);
                    }
                }
                "invalid" => {
                    if let Ok(signature) = from_der(&der_signature) {
                        match secp256k1_verify(&message_hash, &signature, &public_key) {
                            Ok(valid) => assert!(!valid),
                            Err(_) => { /* this is expected for "invalid", all good */ }
                        }
                    } else {
                        // invalid DER encoding, okay
                    }
                }
                _ => panic!("Found unexpected result value"),
            }
            if tc.result == "valid" {}
        }
    }
    assert_eq!(tested, number_of_tests);
}

#[test]
fn ecdsa_secp256k1_sha512() {
    let mut tested: usize = 0;
    let File {
        number_of_tests,
        test_groups,
    } = read_file(SECP256K1_SHA512);
    assert!(number_of_tests >= 533, "Got unexpected number of tests");

    for group in test_groups {
        let public_key = hex::decode(group.public_key.uncompressed).unwrap();
        assert_eq!(public_key.len(), 65);

        for tc in group.tests {
            tested += 1;
            assert_eq!(tc.tc_id as usize, tested);
            // eprintln!("Test case ID: {}", tc.tc_id);
            let message = hex::decode(tc.msg).unwrap();
            let message_hash = hashers::sha512(&message);
            let der_signature = hex::decode(tc.sig).unwrap();

            match tc.result.as_str() {
                "valid" | "acceptable" => {
                    let signature = from_der(&der_signature).unwrap();
                    let valid = secp256k1_verify(&message_hash, &signature, &public_key).unwrap();
                    assert!(valid);
                    if tc.comment != "k*G has a large x-coordinate" {
                        test_secp256k1_recover_pubkey(&message_hash, &signature, &public_key);
                    }
                }
                "invalid" => {
                    if let Ok(signature) = from_der(&der_signature) {
                        match secp256k1_verify(&message_hash, &signature, &public_key) {
                            Ok(valid) => assert!(!valid),
                            Err(_) => { /* this is expected for "invalid", all good */ }
                        }
                    } else {
                        // invalid DER encoding, okay
                    }
                }
                _ => panic!("Found unexpected result value"),
            }
            if tc.result == "valid" {}
        }
    }
    assert_eq!(tested, number_of_tests);
}

#[test]
fn ecdsa_secp256k1_sha3_256() {
    let mut tested: usize = 0;
    let File {
        number_of_tests,
        test_groups,
    } = read_file(SECP256K1_SHA3_256);
    assert!(number_of_tests >= 471, "Got unexpected number of tests");

    for group in test_groups {
        let public_key = hex::decode(group.public_key.uncompressed).unwrap();
        assert_eq!(public_key.len(), 65);

        for tc in group.tests {
            tested += 1;
            assert_eq!(tc.tc_id as usize, tested);
            // eprintln!("Test case ID: {}", tc.tc_id);
            let message = hex::decode(tc.msg).unwrap();
            let message_hash = hashers::sha3_256(&message);
            let der_signature = hex::decode(tc.sig).unwrap();

            match tc.result.as_str() {
                "valid" | "acceptable" => {
                    let signature = from_der(&der_signature).unwrap();
                    let valid = secp256k1_verify(&message_hash, &signature, &public_key).unwrap();
                    assert!(valid);
                    if tc.comment != "k*G has a large x-coordinate" {
                        test_secp256k1_recover_pubkey(&message_hash, &signature, &public_key);
                    }
                }
                "invalid" => {
                    if let Ok(signature) = from_der(&der_signature) {
                        match secp256k1_verify(&message_hash, &signature, &public_key) {
                            Ok(valid) => assert!(!valid),
                            Err(_) => { /* this is expected for "invalid", all good */ }
                        }
                    } else {
                        // invalid DER encoding, okay
                    }
                }
                _ => panic!("Found unexpected result value"),
            }
            if tc.result == "valid" {}
        }
    }
    assert_eq!(tested, number_of_tests);
}

#[test]
fn ecdsa_secp256k1_sha3_512() {
    let mut tested: usize = 0;
    let File {
        number_of_tests,
        test_groups,
    } = read_file(SECP256K1_SHA3_512);
    assert!(number_of_tests >= 537, "Got unexpected number of tests");

    for group in test_groups {
        let public_key = hex::decode(group.public_key.uncompressed).unwrap();
        assert_eq!(public_key.len(), 65);

        for tc in group.tests {
            tested += 1;
            assert_eq!(tc.tc_id as usize, tested);
            // eprintln!("Test case ID: {}", tc.tc_id);
            let message = hex::decode(tc.msg).unwrap();
            let message_hash = hashers::sha3_512(&message);
            let der_signature = hex::decode(tc.sig).unwrap();

            match tc.result.as_str() {
                "valid" | "acceptable" => {
                    let signature = from_der(&der_signature).unwrap();
                    let valid = secp256k1_verify(&message_hash, &signature, &public_key).unwrap();
                    assert!(valid);
                    if tc.comment != "k*G has a large x-coordinate" {
                        test_secp256k1_recover_pubkey(&message_hash, &signature, &public_key);
                    }
                }
                "invalid" => {
                    if let Ok(signature) = from_der(&der_signature) {
                        match secp256k1_verify(&message_hash, &signature, &public_key) {
                            Ok(valid) => assert!(!valid),
                            Err(_) => { /* this is expected for "invalid", all good */ }
                        }
                    } else {
                        // invalid DER encoding, okay
                    }
                }
                _ => panic!("Found unexpected result value"),
            }
            if tc.result == "valid" {}
        }
    }
    assert_eq!(tested, number_of_tests);
}

fn test_secp256k1_recover_pubkey(message_hash: &[u8], signature: &[u8], public_key: &[u8]) {
    // Since the recovery param is missing in the test vectors, we try both 0 and 1
    let recovered0 = secp256k1_recover_pubkey(message_hash, signature, 0).unwrap();
    let recovered1 = secp256k1_recover_pubkey(message_hash, signature, 1).unwrap();
    // Got two different pubkeys. Without the recovery param, we don't know which one is the right one.
    assert_ne!(recovered0, recovered1);
    assert!(recovered0 == public_key || recovered1 == public_key);
}

fn from_der(data: &[u8]) -> Result<[u8; 64], String> {
    const DER_TAG_INTEGER: u8 = 0x02;

    let mut pos = 0;

    let Some(prefix) = data.get(pos) else {
        return Err("Could not read prefix".to_string());
    };
    pos += 1;
    if *prefix != 0x30 {
        return Err("Prefix 0x30 expected".to_string());
    }

    let Some(body_length) = data.get(pos) else {
        return Err("Could not read body length".to_string());
    };
    pos += 1;
    if data.len() - pos != *body_length as usize {
        return Err("Data length mismatch detected".to_string());
    }

    // r
    let Some(r_tag) = data.get(pos) else {
        return Err("Could not read r_tag".to_string());
    };
    pos += 1;
    if *r_tag != DER_TAG_INTEGER {
        return Err("INTEGER tag expected".to_string());
    }
    let Some(r_length) = data.get(pos).map(|rl: &u8| *rl as usize) else {
        return Err("Could not read r_length".to_string());
    };
    pos += 1;
    if r_length >= 0x80 {
        return Err("Decoding length values above 127 not supported".to_string());
    }
    if pos + r_length > data.len() {
        return Err("R length exceeds end of data".to_string());
    }
    let r_data = &data[pos..pos + r_length];
    pos += r_length;

    // s
    let Some(s_tag) = data.get(pos) else {
        return Err("Could not read s_tag".to_string());
    };
    pos += 1;
    if *s_tag != DER_TAG_INTEGER {
        return Err("INTEGER tag expected".to_string());
    }
    let Some(s_length) = data.get(pos).map(|sl| *sl as usize) else {
        return Err("Could not read s_length".to_string());
    };
    pos += 1;
    if s_length >= 0x80 {
        return Err("Decoding length values above 127 not supported".to_string());
    }
    if pos + s_length > data.len() {
        return Err("S length exceeds end of data".to_string());
    }
    let s_data = &data[pos..pos + s_length];
    pos += s_length;

    if pos != data.len() {
        return Err("Extra bytes in data input".to_string());
    }

    let r = decode_unsigned_integer(r_data, "r")?;
    let s = decode_unsigned_integer(s_data, "s")?;

    let mut out = [0u8; 64];
    out[0..32].copy_from_slice(&r);
    out[32..].copy_from_slice(&s);
    Ok(out)
}

fn decode_unsigned_integer(mut data: &[u8], name: &str) -> Result<[u8; 32], String> {
    if data.is_empty() {
        return Err(format!("{name} data is empty"));
    }

    // If high bit of first byte is set, this is interpreted as a negative integer.
    // A leading zero is needed to prevent this.
    if (data[0] & 0x80) != 0 {
        return Err(format!("{name} data missing leading zero"));
    }

    // "Leading octets of all 0's (or all 1's) are not allowed. In other words, the leftmost
    // nine bits of an encoded INTEGER value may not be all 0's or all 1's. This ensures that
    // an INTEGER value is encoded in the smallest possible number of octets."
    // https://www.oss.com/asn1/resources/asn1-made-simple/asn1-quick-reference/basic-encoding-rules.html

    // If leading byte is 0 and there is more than 1 byte, trim it.
    // If the high bit of the following byte is zero as well, the leading 0x00 was invalid.
    if data.len() > 1 && data[0] == 0 {
        data = &data[1..];
        if (data[0] & 0x80) == 0 {
            return Err(format!("{name} data has invalid leading zero"));
        }
    }

    // The other requirement (first 9 bits being all 1) is not yet checked

    // Do we need a better value range check here?
    if data.len() > 32 {
        return Err(format!("{name} data exceeded 32 bytes"));
    }

    Ok(pad_to_32(data))
}

fn pad_to_32(input: &[u8]) -> [u8; 32] {
    let shift = 32 - input.len();
    let mut out = [0u8; 32];
    out[shift..].copy_from_slice(input);
    out
}
