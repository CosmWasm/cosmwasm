use ark_bls12_381::{G1Affine, G2Affine, G2Projective};
use ark_ec::AffineRepr;
use ark_ff::UniformRand;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use criterion::{criterion_group, criterion_main, Criterion, PlottingBackend};
use rand_core::OsRng;
use std::{hint::black_box, io, time::Duration};

use english_numbers::convert_no_fmt;
use hex_literal::hex;
use serde::Deserialize;

// Crypto stuff
use digest::Digest;
use k256::ecdsa::SigningKey; // type alias
use sha2::Sha256;

use cosmwasm_crypto::{
    bls12_381_aggregate_g1, bls12_381_aggregate_g2, bls12_381_hash_to_g1, bls12_381_hash_to_g2,
    bls12_381_pairing_equality, ed25519_batch_verify, ed25519_verify, secp256k1_recover_pubkey,
    secp256k1_verify, secp256r1_recover_pubkey, secp256r1_verify, HashFunction,
    BLS12_381_G1_GENERATOR, BLS12_381_G1_POINT_LEN, BLS12_381_G2_POINT_LEN,
};
use std::cmp::min;

const COSMOS_SECP256K1_MSG_HEX: &str = "0a93010a90010a1c2f636f736d6f732e62616e6b2e763162657461312e4d736753656e6412700a2d636f736d6f7331706b707472653766646b6c366766727a6c65736a6a766878686c63337234676d6d6b38727336122d636f736d6f7331717970717870713971637273737a673270767871367273307a716733797963356c7a763778751a100a0575636f736d12073132333435363712650a4e0a460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a21034f04181eeba35391b858633a765c4a0c189697b40d216354d50890d350c7029012040a02080112130a0d0a0575636f736d12043230303010c09a0c1a0c73696d642d74657374696e672001";
const COSMOS_SECP256K1_SIGNATURE_HEX: &str = "c9dd20e07464d3a688ff4b710b1fbc027e495e797cfa0b4804da2ed117959227772de059808f765aa29b8f92edf30f4c2c5a438e30d3fe6897daa7141e3ce6f9";
const COSMOS_SECP256K1_PUBKEY_HEX: &str =
    "034f04181eeba35391b858633a765c4a0c189697b40d216354d50890d350c70290";

const COSMOS_SECP256R1_MSG_HEX: &str = "5905238877c77421f73e43ee3da6f2d9e2ccad5fc942dcec0cbd25482935faaf416983fe165b1a045ee2bcd2e6dca3bdf46c4310a7461f9a37960ca672d3feb5473e253605fb1ddfd28065b53cb5858a8ad28175bf9bd386a5e471ea7a65c17cc934a9d791e91491eb3754d03799790fe2d308d16146d5c9b0d0debd97d79ce8";
const COSMOS_SECP256R1_SIGNATURE_HEX: &str = "f3ac8061b514795b8843e3d6629527ed2afd6b1f6a555a7acabb5e6f79c8c2ac8bf77819ca05a6b2786c76262bf7371cef97b218e96f175a3ccdda2acc058903";
const COSMOS_SECP256R1_PUBKEY_HEX: &str = "041ccbe91c075fc7f4f033bfa248db8fccd3565de94bbfb12f3c59ff46c271bf83ce4014c68811f9a21a1fdb2c0e6113e06db7ca93b7404e78dc7ccd5ca89a4ca9";

// TEST 3 test vector from https://tools.ietf.org/html/rfc8032#section-7.1
const COSMOS_ED25519_MSG_HEX: &str = "af82";
const COSMOS_ED25519_SIGNATURE_HEX: &str = "6291d657deec24024827e69c3abe01a30ce548a284743a445e3680d7db5ac3ac18ff9b538d16f290ae67f760984dc6594a7c15e9716ed28dc027beceea1ec40a";
const COSMOS_ED25519_PUBLIC_KEY_HEX: &str =
    "fc51cd8e6218a1a38da47ed00230f0580816ed13ba3303ac5deb911548908025";

// Test data from https://tools.ietf.org/html/rfc8032#section-7.1
const COSMOS_ED25519_TESTS_JSON: &str = "./testdata/ed25519_tests.json";

// BLS test vector
// Path: "packages/crypto/testdata/bls-tests/verify/verify_valid_case_2ea479adf8c40300.json"
const BLS_PUBKEY: [u8; 48] = hex!("a491d1b0ecd9bb917989f0e74f0dea0422eac4a873e5e2644f368dffb9a6e20fd6e10c1b77654d067c0618f6e5a7f79a");
const BLS_MESSAGE: [u8; 32] =
    hex!("5656565656565656565656565656565656565656565656565656565656565656");
const BLS_DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";
const BLS_SIGNATURE: [u8; 96] = hex!("882730e5d03f6b42c3abc26d3372625034e1d871b65a8a6b900a56dae22da98abbe1b68f85e49fe7652a55ec3d0591c20767677e33e5cbb1207315c41a9ac03be39c2e7668edc043d6cb1d9fd93033caa8a1c5b0e84bedaeb6c64972503a43eb");

#[derive(Deserialize, Debug)]
struct Encoded {
    #[serde(rename = "privkey")]
    #[allow(dead_code)]
    private_key: String,
    #[serde(rename = "pubkey")]
    public_key: String,
    message: String,
    signature: String,
}

fn read_cosmos_sigs() -> Vec<Encoded> {
    use std::fs::File;
    use std::io::BufReader;

    // Open the file in read-only mode with buffer.
    let file = File::open(COSMOS_ED25519_TESTS_JSON).unwrap();
    let reader = BufReader::new(file);

    serde_json::from_reader(reader).unwrap()
}

#[allow(clippy::type_complexity)]
fn read_decode_cosmos_sigs() -> (Vec<Vec<u8>>, Vec<Vec<u8>>, Vec<Vec<u8>>) {
    let codes = read_cosmos_sigs();

    let mut messages: Vec<Vec<u8>> = vec![];
    let mut signatures: Vec<Vec<u8>> = vec![];
    let mut public_keys: Vec<Vec<u8>> = vec![];

    for encoded in codes {
        let message = hex::decode(&encoded.message).unwrap();
        messages.push(message);

        let signature = hex::decode(&encoded.signature).unwrap();
        signatures.push(signature);

        let public_key = hex::decode(&encoded.public_key).unwrap();
        public_keys.push(public_key);
    }

    (messages, signatures, public_keys)
}

fn bench_bls<M>(group: &mut criterion::BenchmarkGroup<'_, M>)
where
    M: criterion::measurement::Measurement,
{
    let two_pow_max = 8;
    let num_random_points = 2_usize.pow(two_pow_max);

    {
        let random_points_g1: Vec<G1Affine> = (0..num_random_points)
            .map(|_| G1Affine::rand(&mut OsRng))
            .collect();
        let mut g1_serialized = io::Cursor::new(Vec::new());
        random_points_g1
            .serialize_compressed(&mut g1_serialized)
            .unwrap();
        let g1_serialized = &g1_serialized.into_inner()[8..];

        let random_points_g2: Vec<G2Affine> = (0..num_random_points)
            .map(|_| G2Affine::rand(&mut OsRng))
            .collect();
        let mut g2_serialized = io::Cursor::new(Vec::new());
        random_points_g2
            .serialize_compressed(&mut g2_serialized)
            .unwrap();
        let g2_serialized = &g2_serialized.into_inner()[8..];

        for i in 1..=two_pow_max {
            let num_points = 2_usize.pow(i);
            let points_to_aggregate_g1 = &g1_serialized[..num_points * BLS12_381_G1_POINT_LEN];
            group.bench_function(format!("bls12_381_aggregate_g1_{num_points}"), |b| {
                b.iter(|| bls12_381_aggregate_g1(points_to_aggregate_g1).unwrap());
            });
        }

        for i in 1..=two_pow_max {
            let num_points = 2_usize.pow(i);
            let points_to_aggregate_g2 = &g2_serialized[..num_points * BLS12_381_G2_POINT_LEN];
            group.bench_function(format!("bls12_381_aggregate_g2_{num_points}"), |b| {
                b.iter(|| bls12_381_aggregate_g2(points_to_aggregate_g2).unwrap());
            });
        }
    }

    {
        const MESSAGE: &[u8] = b"message";
        const DST: &[u8] = b"dst";
        let secret_keys: Vec<ark_bls12_381::Fr> = (0..num_random_points)
            .map(|_| ark_bls12_381::Fr::rand(&mut OsRng))
            .collect();
        let public_keys: Vec<G1Affine> = secret_keys
            .iter()
            .map(|secret_key| G1Affine::generator() * secret_key)
            .map(Into::into)
            .collect();
        let messages: Vec<G2Affine> = (0..num_random_points)
            .map(|_| bls12_381_hash_to_g2(HashFunction::Sha256, MESSAGE, DST))
            .map(|bytes| G2Affine::deserialize_compressed(&bytes[..]).unwrap())
            .collect();
        let signatures: Vec<G2Projective> = secret_keys
            .iter()
            .zip(messages.iter())
            .map(|(secret_key, message)| *message * secret_key)
            .collect();

        for i in 0..=two_pow_max {
            let n = 2_usize.pow(i); // the number of pairings on the left hand side
            let k = n + 1; // the number of pairings in total
            let messages = &messages[..n];
            let keys = &public_keys[..n];
            let aggregated_signature: G2Affine =
                signatures[..n].iter().sum::<G2Projective>().into();

            let serialized_pubkeys: Vec<u8> = keys
                .iter()
                .flat_map(|key| {
                    let mut serialized = [0_u8; 48];
                    key.serialize_compressed(&mut serialized[..]).unwrap();
                    serialized
                })
                .collect();

            let serialized_messages: Vec<u8> = messages
                .iter()
                .flat_map(|message| {
                    let mut serialized = [0_u8; 96];
                    message.serialize_compressed(&mut serialized[..]).unwrap();
                    serialized
                })
                .collect();

            let mut serialized_signature = [0_u8; 96];
            aggregated_signature
                .serialize_compressed(&mut serialized_signature[..])
                .unwrap();

            group.bench_function(format!("bls12_381_pairing_equality_k={k}"), |b| {
                b.iter(|| {
                    let is_valid = black_box(bls12_381_pairing_equality(
                        &serialized_pubkeys,
                        &serialized_messages,
                        &BLS12_381_G1_GENERATOR,
                        &serialized_signature,
                    ))
                    .unwrap();

                    assert!(is_valid);
                });
            });
        }
    }

    group.bench_function("bls12_381_hash_to_g1", |b| {
        b.iter(|| {
            bls12_381_hash_to_g1(
                black_box(HashFunction::Sha256),
                black_box(&BLS_MESSAGE),
                black_box(BLS_DST),
            )
        });
    });

    group.bench_function("bls12_381_hash_to_g2", |b| {
        b.iter(|| {
            bls12_381_hash_to_g2(
                black_box(HashFunction::Sha256),
                black_box(&BLS_MESSAGE),
                black_box(BLS_DST),
            )
        });
    });

    group.bench_function("bls12_381_verify", |b| {
        let generator = BLS12_381_G1_GENERATOR;
        let message = bls12_381_hash_to_g2(HashFunction::Sha256, &BLS_MESSAGE, BLS_DST);

        b.iter(|| {
            let is_equal = bls12_381_pairing_equality(
                black_box(&BLS_PUBKEY),
                &message,
                &generator,
                black_box(&BLS_SIGNATURE),
            )
            .unwrap();

            assert!(is_equal);
        });
    });
}

fn bench_crypto(c: &mut Criterion) {
    let mut group = c.benchmark_group("Crypto");

    group.bench_function("secp256k1_verify", |b| {
        let message = hex::decode(COSMOS_SECP256K1_MSG_HEX).unwrap();
        let message_hash = Sha256::digest(message);
        let signature = hex::decode(COSMOS_SECP256K1_SIGNATURE_HEX).unwrap();
        let public_key = hex::decode(COSMOS_SECP256K1_PUBKEY_HEX).unwrap();
        b.iter(|| {
            assert!(secp256k1_verify(&message_hash, &signature, &public_key).unwrap());
        });
    });

    group.bench_function("secp256k1_recover_pubkey", |b| {
        let message_hash =
            hex!("82ff40c0a986c6a5cfad4ddf4c3aa6996f1a7837f9c398e17e5de5cbd5a12b28");
        let private_key =
            hex!("3c9229289a6125f7fdf1885a77bb12c37a8d3b4962d936f7e3084dece32a3ca1");
        let r_s = hex!("99e71a99cb2270b8cac5254f9e99b6210c6c10224a1579cf389ef88b20a1abe9129ff05af364204442bdb53ab6f18a99ab48acc9326fa689f228040429e3ca66");
        let recovery_param: u8 = 0;

        let expected = SigningKey::from_bytes(&private_key.into())
            .unwrap()
            .verifying_key()
            .to_encoded_point(false)
            .as_bytes()
            .to_vec();

        b.iter(|| {
            let pubkey = secp256k1_recover_pubkey(&message_hash, &r_s, recovery_param).unwrap();
            assert_eq!(pubkey, expected);
        });
    });

    group.bench_function("secp256r1_verify", |b| {
        let message = hex::decode(COSMOS_SECP256R1_MSG_HEX).unwrap();
        let message_hash = Sha256::digest(message);
        let signature = hex::decode(COSMOS_SECP256R1_SIGNATURE_HEX).unwrap();
        let public_key = hex::decode(COSMOS_SECP256R1_PUBKEY_HEX).unwrap();
        b.iter(|| {
            assert!(secp256r1_verify(&message_hash, &signature, &public_key).unwrap());
        });
    });

    group.bench_function("secp256r1_recover_pubkey", |b| {
        let message_hash =
            hex!("aea3e069e03c0ff4d6b3fa2235e0053bbedc4c7e40efbc686d4dfb5efba4cfed");
        let expected  =
            hex!("04105d22d9c626520faca13e7ced382dcbe93498315f00cc0ac39c4821d0d737376c47f3cbbfa97dfcebe16270b8c7d5d3a5900b888c42520d751e8faf3b401ef4");
        let r_s = hex!("542c40a18140a6266d6f0286e24e9a7bad7650e72ef0e2131e629c076d9626634f7f65305e24a6bbb5cff714ba8f5a2cee5bdc89ba8d75dcbf21966ce38eb66f");
        let recovery_param: u8 = 1;

        b.iter(|| {
            let pubkey = secp256r1_recover_pubkey(&message_hash, &r_s, recovery_param).unwrap();
            assert_eq!(pubkey, expected);
        });
    });

    bench_bls(&mut group);

    group.bench_function("ed25519_verify", |b| {
        let message = hex::decode(COSMOS_ED25519_MSG_HEX).unwrap();
        let signature = hex::decode(COSMOS_ED25519_SIGNATURE_HEX).unwrap();
        let public_key = hex::decode(COSMOS_ED25519_PUBLIC_KEY_HEX).unwrap();
        b.iter(|| {
            assert!(ed25519_verify(&message, &signature, &public_key).unwrap());
        });
    });

    // Ed25519 batch verification of different batch lengths
    {
        let (messages, signatures, public_keys) = read_decode_cosmos_sigs();
        let messages: Vec<&[u8]> = messages.iter().map(|m| m.as_slice()).collect();
        let signatures: Vec<&[u8]> = signatures.iter().map(|m| m.as_slice()).collect();
        let public_keys: Vec<&[u8]> = public_keys.iter().map(|m| m.as_slice()).collect();

        for n in (1..=min(messages.len(), 10)).step_by(2) {
            group.bench_function(
                format!("ed25519_batch_verify_{}", convert_no_fmt(n as i64)),
                |b| {
                    b.iter(|| {
                        assert!(ed25519_batch_verify(
                            &mut OsRng,
                            &messages[..n],
                            &signatures[..n],
                            &public_keys[..n]
                        )
                        .unwrap());
                    });
                },
            );
        }
    }

    // Ed25519 batch verification of different batch lengths, with the same pubkey
    {
        //FIXME: Use different messages / signatures
        let messages = [hex::decode(COSMOS_ED25519_MSG_HEX).unwrap()];
        let signatures = [hex::decode(COSMOS_ED25519_SIGNATURE_HEX).unwrap()];
        let public_keys = [hex::decode(COSMOS_ED25519_PUBLIC_KEY_HEX).unwrap()];

        let messages: Vec<&[u8]> = messages.iter().map(|m| m.as_slice()).collect();
        let signatures: Vec<&[u8]> = signatures.iter().map(|m| m.as_slice()).collect();
        let public_keys: Vec<&[u8]> = public_keys.iter().map(|m| m.as_slice()).collect();

        for n in (1..10).step_by(2) {
            group.bench_function(
                format!(
                    "ed25519_batch_verify_one_pubkey_{}",
                    convert_no_fmt(n as i64)
                ),
                |b| {
                    b.iter(|| {
                        assert!(ed25519_batch_verify(
                            &mut OsRng,
                            &messages.repeat(n),
                            &signatures.repeat(n),
                            &public_keys
                        )
                        .unwrap());
                    });
                },
            );
        }
    }

    group.finish();
}

fn make_config() -> Criterion {
    Criterion::default()
        .plotting_backend(PlottingBackend::Plotters)
        .without_plots()
        .measurement_time(Duration::new(10, 0))
        .sample_size(12)
}

criterion_group!(
    name = crypto;
    config = make_config();
    targets = bench_crypto
);
criterion_main!(crypto);
