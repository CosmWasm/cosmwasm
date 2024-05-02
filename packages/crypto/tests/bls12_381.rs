#![cfg(feature = "std")]

use std::{error::Error, fs};

use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use base64::engine::general_purpose::STANDARD;
use base64_serde::base64_serde_type;
use cosmwasm_crypto::{
    bls12_381_aggregate_g1, bls12_381_aggregate_g2, bls12_381_g1_is_identity,
    bls12_381_g2_is_identity, bls12_381_hash_to_g2, bls12_381_pairing_equality, HashFunction,
    BLS12_381_G1_GENERATOR_COMPRESSED, BLS12_381_G2_POINT_LEN,
};

const PROOF_OF_POSSESSION_DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";

base64_serde_type!(Base64Standard, STANDARD);

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct EthPubkey(#[serde(with = "Base64Standard")] Vec<u8>);

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct EthHeaders {
    public_keys: Vec<EthPubkey>,
    #[serde(with = "Base64Standard")]
    message: Vec<u8>,
    #[serde(with = "Base64Standard")]
    signature: Vec<u8>,
    #[serde(with = "Base64Standard")]
    aggregate_pubkey: Vec<u8>,
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct AggregateTestFile {
    input: Vec<String>,
    output: Option<String>,
}

struct AggregateTest {
    input: Vec<Vec<u8>>,
    output: Option<Vec<u8>>,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct HashTestInput {
    msg: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct HashTestOutput {
    x: String,
    y: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct HashTestFile {
    input: HashTestInput,
    output: HashTestOutput,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct VerifyTestInput {
    pubkey: String,
    message: String,
    signature: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct VerifyTestFile {
    input: VerifyTestInput,
    output: bool,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct AggregateVerifyInput {
    pubkeys: Vec<String>,
    messages: Vec<String>,
    signature: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct AggregateVerifyFile {
    input: AggregateVerifyInput,
    output: bool,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct BatchVerifyInput {
    pubkeys: Vec<String>,
    messages: Vec<String>,
    signatures: Vec<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct BatchVerifyFile {
    input: BatchVerifyInput,
    output: bool,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct FastAggregateVerifyInput {
    pubkeys: Vec<String>,
    message: String,
    signature: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct FastAggregateVerifyFile {
    input: FastAggregateVerifyInput,
    output: bool,
}

const ETH_HEADER_FILE: &str = include_str!("../testdata/eth-headers/1699693797.394876721s.json");
const AGGREGATE_1: &str = include_str!("../testdata/bls-tests/aggregate/aggregate_0x0000000000000000000000000000000000000000000000000000000000000000.json");
const AGGREGATE_2: &str = include_str!("../testdata/bls-tests/aggregate/aggregate_0x5656565656565656565656565656565656565656565656565656565656565656.json");
const AGGREGATE_3: &str = include_str!("../testdata/bls-tests/aggregate/aggregate_0xabababababababababababababababababababababababababababababababab.json");
const AGGREGATE_4: &str =
    include_str!("../testdata/bls-tests/aggregate/aggregate_infinity_signature.json");
const AGGREGATE_5: &str =
    include_str!("../testdata/bls-tests/aggregate/aggregate_na_signatures.json");
const AGGREGATE_6: &str =
    include_str!("../testdata/bls-tests/aggregate/aggregate_single_signature.json");

fn read_eth_header_file() -> EthHeaders {
    serde_json::from_str(ETH_HEADER_FILE).unwrap()
}

fn read_aggregate_test(json: &str) -> AggregateTest {
    let file: AggregateTestFile = serde_json::from_str(json).unwrap();
    AggregateTest {
        input: file
            .input
            .into_iter()
            .map(|entry| hex::decode(&entry[2..]).unwrap())
            .collect(),
        output: file.output.map(|entry| hex::decode(&entry[2..]).unwrap()),
    }
}

// Test for https://eth2book.info/capella/part2/building_blocks/signatures/#aggregating-public-keys
#[test]
fn bls12_381_aggregate_g1_works() {
    let file = read_eth_header_file();

    let pubkeys: Vec<&[u8]> = file.public_keys.iter().map(|m| m.0.as_slice()).collect();
    let pubkeys_combined: Vec<u8> = pubkeys.concat();

    let sum = bls12_381_aggregate_g1(&pubkeys_combined).unwrap();
    assert_eq!(sum.as_slice(), file.aggregate_pubkey);
}

// Test for https://eth2book.info/capella/part2/building_blocks/signatures/#aggregating-signatures
#[test]
fn bls12_381_aggregate_g2_works() {
    for json in [
        AGGREGATE_1,
        AGGREGATE_2,
        AGGREGATE_3,
        AGGREGATE_4,
        AGGREGATE_5,
        AGGREGATE_6,
    ] {
        let test = read_aggregate_test(json);
        let signatures: Vec<&[u8]> = test.input.iter().map(|m| m.as_slice()).collect();
        let signatures_combined: Vec<u8> = signatures.concat();

        // Skip empty signatures since we explicitly error on empty inputs
        if signatures_combined.is_empty() {
            continue;
        }

        let sum = bls12_381_aggregate_g2(&signatures_combined).unwrap();
        match test.output {
            Some(expected) => assert_eq!(sum.as_slice(), expected),
            None => assert_eq!(
                sum.as_slice(),
                // point at infinity – is this what we want here?
                [
                    // C_bit set (compression)
                    // I_bit set (point at infinity)
                    // S_bit unset (sign)
                    0b11000000, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ]
            ),
        }
    }
}

#[test]
fn bls12_381_hash_to_g2_works() {
    let paths = glob::glob("testdata/bls-tests/hash_to_G2/*.json")
        .unwrap()
        .flatten();

    for path in paths {
        let test_data = fs::read(&path).unwrap();
        let test_data: HashTestFile = serde_json::from_slice(&test_data).unwrap();
        let g2_point = bls12_381_hash_to_g2(
            HashFunction::Sha256,
            test_data.input.msg.as_bytes(),
            b"QUUX-V01-CS02-with-BLS12381G2_XMD:SHA-256_SSWU_RO_",
        );

        let prepared_x = test_data.output.x.replace("0x", "");
        let (x1, x2) = prepared_x.split_once(',').unwrap();
        let decoded_x = hex::decode(format!("{x2}{x1}")).unwrap();

        let prepared_y = test_data.output.y.replace("0x", "");
        let (y1, y2) = prepared_y.split_once(',').unwrap();
        let decoded_y = hex::decode(format!("{y2}{y1}")).unwrap();
        let uncompressed = [decoded_x.as_slice(), &decoded_y].concat();

        let affine = ark_bls12_381::G2Affine::deserialize_uncompressed(&uncompressed[..]).unwrap();
        let mut compressed_affine = [0; BLS12_381_G2_POINT_LEN];
        affine
            .serialize_compressed(&mut compressed_affine[..])
            .unwrap();

        assert_eq!(
            g2_point,
            compressed_affine,
            "Failed with test vector {}",
            path.display()
        );
    }
}

#[test]
fn bls12_381_verify_works() {
    let paths = glob::glob("testdata/bls-tests/verify/*.json")
        .unwrap()
        .flatten();

    for path in paths {
        let test_data = fs::read(&path).unwrap();
        let test_data: VerifyTestFile = serde_json::from_slice(&test_data).unwrap();

        let pubkey = hex::decode(&test_data.input.pubkey[2..]).unwrap();
        let message = hex::decode(&test_data.input.message[2..]).unwrap();
        let signature = hex::decode(&test_data.input.signature[2..]).unwrap();

        let message_point =
            bls12_381_hash_to_g2(HashFunction::Sha256, &message, PROOF_OF_POSSESSION_DST);

        let pubkey = pubkey.try_into().unwrap();
        let signature = signature.try_into().unwrap();

        let verify_result = (|| {
            if bls12_381_g1_is_identity(&pubkey)? {
                println!("pubkey is identity");
                return Ok(false);
            }

            if bls12_381_g2_is_identity(&signature)? {
                println!("signature is identity");
                return Ok(false);
            }

            let bool_result = bls12_381_pairing_equality(
                &pubkey,
                &message_point,
                &BLS12_381_G1_GENERATOR_COMPRESSED,
                &signature,
            )?;

            if !bool_result {
                println!("pairing is not equal");
            }

            Ok::<_, Box<dyn Error>>(bool_result)
        })();

        let verify_result = verify_result
            .map_err(|err| eprintln!("error: {err}"))
            .unwrap_or(false);

        assert_eq!(
            verify_result,
            test_data.output,
            "Failed with test vector {}",
            path.display()
        );

        println!("Finished case {}", path.display());
        println!("========================");
    }
}

#[test]
fn bls12_381_aggregate_verify_works() {
    let paths = glob::glob("testdata/bls-tests/aggregate_verify/*.json")
        .unwrap()
        .flatten();

    for path in paths {
        let test_data = fs::read(&path).unwrap();
        let test_data: AggregateVerifyFile = serde_json::from_slice(&test_data).unwrap();

        let signature = hex::decode(&test_data.input.signature[2..]).unwrap();

        let messages: Vec<u8> = test_data
            .input
            .messages
            .iter()
            .flat_map(|message| {
                let msg = hex::decode(&message[2..]).unwrap();
                bls12_381_hash_to_g2(HashFunction::Sha256, &msg, PROOF_OF_POSSESSION_DST)
            })
            .collect();

        let verify_result = (|| {
            let signature = signature.as_slice().try_into()?;
            if bls12_381_g2_is_identity(&signature)? {
                println!("signature is identity");
                return Ok(false);
            }

            let mut pubkeys: Vec<u8> = Vec::with_capacity(test_data.input.pubkeys.len() * 48);
            for pubkey in test_data.input.pubkeys {
                let pubkey = hex::decode(&pubkey[2..]).unwrap();

                if bls12_381_g1_is_identity(&pubkey.as_slice().try_into()?)? {
                    println!("pubkey is identity");
                    return Ok(false);
                }

                pubkeys.extend(pubkey);
            }

            if pubkeys.is_empty() || messages.is_empty() {
                println!("no keys or no signatures");
                return Ok(false);
            }

            let bool_result = bls12_381_pairing_equality(
                &pubkeys,
                &messages,
                &BLS12_381_G1_GENERATOR_COMPRESSED,
                &signature,
            )?;

            if !bool_result {
                println!("pairing is not equal");
            }

            Ok::<_, Box<dyn Error>>(bool_result)
        })();

        let verify_result = verify_result
            .map_err(|err| eprintln!("error: {err:?}"))
            .unwrap_or(false);

        assert_eq!(
            verify_result,
            test_data.output,
            "Failed with test vector {}",
            path.display()
        );

        println!("Finished case {}", path.display());
        println!("========================");
    }
}

#[test]
fn bls12_381_fast_aggregate_verify_works() {
    let paths = glob::glob("testdata/bls-tests/fast_aggregate_verify/*.json")
        .unwrap()
        .flatten();

    for path in paths {
        let test_data = fs::read(&path).unwrap();
        let test_data: FastAggregateVerifyFile = serde_json::from_slice(&test_data).unwrap();

        let message = hex::decode(&test_data.input.message[2..]).unwrap();
        let signature = hex::decode(&test_data.input.signature[2..]).unwrap();

        let message_point =
            bls12_381_hash_to_g2(HashFunction::Sha256, &message, PROOF_OF_POSSESSION_DST);
        let signature = signature.try_into().unwrap();

        let verify_result = (|| {
            let mut pubkeys: Vec<u8> = Vec::with_capacity(test_data.input.pubkeys.len() * 48);
            for pubkey in test_data.input.pubkeys {
                let pubkey = hex::decode(&pubkey[2..]).unwrap();

                if bls12_381_g1_is_identity(&pubkey.as_slice().try_into()?)? {
                    println!("pubkey is identity");
                    return Ok(false);
                }

                pubkeys.extend(pubkey);
            }

            // Reject cases with empty public keys since the aggregation will:
            //
            // 1. error out with our implementation specifically
            // 2. if it wouldn't error out, it would return the identity element of G1, making the
            //    signature validation return invalid anyway
            if pubkeys.is_empty() {
                return Ok(false);
            }
            let pubkey = bls12_381_aggregate_g1(&pubkeys).unwrap();

            if bls12_381_g2_is_identity(&signature)? {
                println!("signature is identity");
                return Ok(false);
            }

            let bool_result = bls12_381_pairing_equality(
                &pubkey,
                &message_point,
                &BLS12_381_G1_GENERATOR_COMPRESSED,
                &signature,
            )?;

            if !bool_result {
                println!("pairing is not equal");
            }

            Ok::<_, Box<dyn Error>>(bool_result)
        })();

        let verify_result = verify_result
            .map_err(|err| eprintln!("error: {err}"))
            .unwrap_or(false);

        assert_eq!(
            verify_result,
            test_data.output,
            "Failed with test vector {}",
            path.display()
        );

        println!("Finished case {}", path.display());
        println!("========================");
    }
}
