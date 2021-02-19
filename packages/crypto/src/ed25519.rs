use ed25519_zebra as ed25519;
use rand_core::OsRng;
use std::convert::TryFrom;

use crate::errors::{CryptoError, CryptoResult};

/// Max length of a message for ed25519 verification in bytes.
/// This is an arbitrary value, for performance / memory contraints. If you need to verify larger
/// messages, let us know.
pub const MESSAGE_MAX_LEN: usize = 131072;

/// Max number of batch messages / signatures / public_keys.
/// This is an arbitrary value, for performance / memory contraints. If you need to batch-verify a
/// larger number of signatures, let us know.
pub const BATCH_MAX_LEN: usize = 128;

/// EdDSA (ed25519) parameters
/// Length of a serialized signature
pub const EDDSA_SIGNATURE_LEN: usize = 64;

/// Length of a serialized public key
pub const EDDSA_PUBKEY_LEN: usize = 32;

/// EdDSA ed25519 implementation.
///
/// This function verifies messages against a signature, with the public key of the signer,
/// using the ed25519 elliptic curve digital signature parametrization / algorithm.
///
/// The maximum currently supported message length is 4096 bytes.
/// The signature and public key are in [Tendermint](https://docs.tendermint.com/v0.32/spec/blockchain/encoding.html#public-key-cryptography)
/// format:
/// - signature: raw ED25519 signature (64 bytes).
/// - public key: raw ED25519 public key (32 bytes).
pub fn ed25519_verify(message: &[u8], signature: &[u8], public_key: &[u8]) -> CryptoResult<bool> {
    // Validation
    if message.len() > MESSAGE_MAX_LEN {
        return Err(CryptoError::msg_err(format!(
            "too large: {}",
            message.len()
        )));
    }
    if signature.len() != EDDSA_SIGNATURE_LEN {
        return Err(CryptoError::sig_err(format!(
            "wrong / unsupported length: {}",
            signature.len()
        )));
    }
    let pubkey_len = public_key.len();
    if pubkey_len == 0 {
        return Err(CryptoError::pubkey_err("empty"));
    }
    if pubkey_len != EDDSA_PUBKEY_LEN {
        return Err(CryptoError::pubkey_err(format!(
            "wrong / unsupported length: {}",
            pubkey_len,
        )));
    }
    // Deserialization
    let signature = ed25519::Signature::try_from(signature)
        .map_err(|err| CryptoError::generic_err(err.to_string()))?;

    let public_key = ed25519::VerificationKey::try_from(public_key)
        .map_err(|err| CryptoError::generic_err(err.to_string()))?;

    // Verification
    match public_key.verify(&signature, &message) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Performs batch Ed25519 signature verification.
///
/// Batch verification asks whether all signatures in some set are valid, rather than asking whether
/// each of them is valid. This allows sharing computations among all signature verifications,
/// performing less work overall, at the cost of higher latency (the entire batch must complete),
/// complexity of caller code (which must assemble a batch of signatures across work-items),
/// and loss of the ability to easily pinpoint failing signatures.
///
/// This batch verification implementation is adaptive, in the sense that it detects multiple
/// signatures created with the same verification key, and automatically coalesces terms
/// in the final verification equation.
///
/// In the limiting case where all signatures in the batch are made with the same verification key,
/// coalesced batch verification runs twice as fast as ordinary batch verification.
pub fn ed25519_batch_verify(
    messages: &[&[u8]],
    signatures: &[&[u8]],
    public_keys: &[&[u8]],
) -> CryptoResult<bool> {
    let mut batch = ed25519::batch::Verifier::new();

    for (((i, &message), &signature), &public_key) in
        (1..).zip(messages).zip(signatures).zip(public_keys)
    {
        // Validation
        if message.len() > MESSAGE_MAX_LEN {
            return Err(CryptoError::msg_err(format!(
                "message {}: too large: {}",
                i,
                message.len()
            )));
        }
        if signature.len() != EDDSA_SIGNATURE_LEN {
            return Err(CryptoError::sig_err(format!(
                "signature {}: wrong / unsupported length: {}",
                i,
                signature.len()
            )));
        }
        let pubkey_len = public_key.len();
        if pubkey_len == 0 {
            return Err(CryptoError::pubkey_err(format!("public key {}: empty", i)));
        }
        if pubkey_len != EDDSA_PUBKEY_LEN {
            return Err(CryptoError::pubkey_err(format!(
                "public key {}: wrong / unsupported length: {}",
                i, pubkey_len,
            )));
        }

        // Deserialization
        let signature = ed25519::Signature::try_from(signature).map_err(|err| {
            CryptoError::generic_err(format!("signature {}: {}", i, err.to_string()))
        })?;

        let public_key = ed25519::VerificationKey::try_from(public_key).map_err(|err| {
            CryptoError::generic_err(format!("public key {}: {}", i, err.to_string()))
        })?;

        // Enqueing
        batch.queue((public_key.into(), signature, message));
    }

    // Batch Verification
    match batch.verify(&mut OsRng) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // For generic signature verification
    const MSG: &str = "Hello World!";

    // Cosmos ed25519 signature verification
    // TEST 1 from https://tools.ietf.org/html/rfc8032#section-7.1
    const COSMOS_ED25519_MSG: &str = "";
    const COSMOS_ED25519_PRIVATE_KEY_HEX: &str =
        "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";
    const COSMOS_ED25519_PUBLIC_KEY_HEX: &str =
        "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
    const COSMOS_ED25519_SIGNATURE_HEX: &str = "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b";

    // Test data from https://tools.ietf.org/html/rfc8032#section-7.1
    const COSMOS_ED25519_TESTS_JSON: &str = "./testdata/ed25519_tests.json";

    #[test]
    fn test_ed25519_verify() {
        let message = MSG.as_bytes();
        // Signing
        let secret_key = ed25519::SigningKey::new(&mut OsRng);
        let signature = secret_key.sign(&message);

        let public_key = ed25519::VerificationKey::from(&secret_key);

        // Serialization. Types can be converted to raw byte arrays with From/Into
        let signature_bytes: [u8; 64] = signature.into();
        let public_key_bytes: [u8; 32] = public_key.into();

        // Verification
        assert!(ed25519_verify(&message, &signature_bytes, &public_key_bytes).unwrap());

        // Wrong message fails
        let bad_message = [message, b"\0"].concat();
        assert!(!ed25519_verify(&bad_message, &signature_bytes, &public_key_bytes).unwrap());

        // Other pubkey fails
        let other_secret_key = ed25519::SigningKey::new(&mut OsRng);
        let other_public_key = ed25519::VerificationKey::from(&other_secret_key);
        let other_public_key_bytes: [u8; 32] = other_public_key.into();
        assert!(!ed25519_verify(&message, &signature_bytes, &other_public_key_bytes).unwrap());
    }

    #[test]
    fn test_cosmos_ed25519_verify() {
        let secret_key = ed25519::SigningKey::try_from(
            hex::decode(COSMOS_ED25519_PRIVATE_KEY_HEX)
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        let public_key = ed25519::VerificationKey::try_from(
            hex::decode(COSMOS_ED25519_PUBLIC_KEY_HEX)
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        let signature = secret_key.sign(&COSMOS_ED25519_MSG.as_bytes());

        let signature_bytes: [u8; 64] = signature.into();
        let public_key_bytes: [u8; 32] = public_key.into();

        assert_eq!(
            signature_bytes,
            hex::decode(&COSMOS_ED25519_SIGNATURE_HEX)
                .unwrap()
                .as_slice()
        );

        assert!(ed25519_verify(
            &COSMOS_ED25519_MSG.as_bytes(),
            &signature_bytes,
            &public_key_bytes
        )
        .unwrap());
    }

    #[test]
    fn test_cosmos_extra_ed25519_verify() {
        use std::fs::File;
        use std::io::BufReader;

        use serde::Deserialize;

        #[derive(Deserialize, Debug)]
        struct Encoded {
            #[serde(rename = "privkey")]
            private_key: String,
            #[serde(rename = "pubkey")]
            public_key: String,
            message: String,
            signature: String,
        }

        // Open the file in read-only mode with buffer.
        let file = File::open(COSMOS_ED25519_TESTS_JSON).unwrap();
        let reader = BufReader::new(file);

        let codes: Vec<Encoded> = serde_json::from_reader(reader).unwrap();

        for (i, encoded) in (1..).zip(codes) {
            let message = hex::decode(&encoded.message).unwrap();

            let signature = hex::decode(&encoded.signature).unwrap();

            let public_key = hex::decode(&encoded.public_key).unwrap();

            // ed25519_verify() works
            assert!(
                ed25519_verify(&message, &signature, &public_key).unwrap(),
                format!("verify() failed (test case {})", i)
            );
        }
    }

    #[test]
    fn test_cosmos_ed25519_batch_verify() {
        use std::fs::File;
        use std::io::BufReader;

        use serde::Deserialize;

        #[derive(Deserialize, Debug)]
        struct Encoded {
            #[serde(rename = "privkey")]
            private_key: String,
            #[serde(rename = "pubkey")]
            public_key: String,
            message: String,
            signature: String,
        }

        // Open the file in read-only mode with buffer.
        let file = File::open(COSMOS_ED25519_TESTS_JSON).unwrap();
        let reader = BufReader::new(file);

        let codes: Vec<Encoded> = serde_json::from_reader(reader).unwrap();

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

        let messages: Vec<&[u8]> = messages.iter().map(|m| m.as_slice()).collect();
        let signatures: Vec<&[u8]> = signatures.iter().map(|m| m.as_slice()).collect();
        let public_keys: Vec<&[u8]> = public_keys.iter().map(|m| m.as_slice()).collect();

        // ed25519_batch_verify() works
        assert!(ed25519_batch_verify(&messages, &signatures, &public_keys).unwrap());
    }
}
