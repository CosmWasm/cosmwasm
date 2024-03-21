use base64::engine::general_purpose::STANDARD;
use base64_serde::base64_serde_type;
use cosmwasm_crypto::{bls12_318_aggregate_g1, bls12_318_aggregate_g2};

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
fn bls12_318_aggregate_g1_works() {
    let file = read_eth_header_file();

    let pubkeys: Vec<&[u8]> = file.public_keys.iter().map(|m| m.0.as_slice()).collect();
    let pubkeys_combined: Vec<u8> = pubkeys.concat();

    let sum = bls12_318_aggregate_g1(&pubkeys_combined).unwrap();
    assert_eq!(sum.as_slice(), file.aggregate_pubkey);
}

// Test for https://eth2book.info/capella/part2/building_blocks/signatures/#aggregating-signatures
#[test]
fn bls12_318_aggregate_g2_works() {
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
        let sum = bls12_318_aggregate_g2(&signatures_combined).unwrap();
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
