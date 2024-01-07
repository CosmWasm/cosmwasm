use digest::{Digest, Update}; // trait
use p256::{
    ecdsa::signature::DigestVerifier, // traits
    ecdsa::{Signature, VerifyingKey}, // type aliases
};
use std::convert::TryInto;

use crate::errors::{CryptoError, CryptoResult};
use crate::identity_digest::Identity256;

/// Length of a serialized compressed public key
const ECDSA_COMPRESSED_PUBKEY_LEN: usize = 33;
/// Length of a serialized uncompressed public key
const ECDSA_UNCOMPRESSED_PUBKEY_LEN: usize = 65;

/// ECDSA secp256r1 implementation.
///
/// This function verifies message hashes (typically, hashed using SHA-256) against a signature,
/// with the public key of the signer, using the secp256r1 elliptic curve digital signature
/// parametrization / algorithm.
///
/// The signature and public key are in "Cosmos" format:
/// - signature:  Serialized "compact" signature (64 bytes).
/// - public key: [Serialized according to SEC 2](https://www.oreilly.com/library/view/programming-bitcoin/9781492031482/ch04.html)
/// (33 or 65 bytes).
pub fn secp256r1_verify(
    message_hash: &[u8],
    signature: &[u8],
    public_key: &[u8],
) -> CryptoResult<bool> {
    let message_hash = read_hash(message_hash)?;
    let signature = read_signature(signature)?;
    check_pubkey(public_key)?;

    // Already hashed, just build Digest container
    let message_digest = Identity256::new().chain(message_hash);

    let mut signature = Signature::from_bytes(&signature.into())
        .map_err(|e| CryptoError::generic_err(e.to_string()))?;

    // High-S signatures require normalization since our verification implementation
    // rejects them by default. If we had a verifier that does not restrict to
    // low-S only, this step was not needed.
    if let Some(normalized) = signature.normalize_s() {
        signature = normalized;
    }

    let public_key = VerifyingKey::from_sec1_bytes(public_key)
        .map_err(|e| CryptoError::generic_err(e.to_string()))?;

    match public_key.verify_digest(message_digest, &signature) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Error raised when hash is not 32 bytes long
struct InvalidSecp256r1HashFormat;

impl From<InvalidSecp256r1HashFormat> for CryptoError {
    fn from(_original: InvalidSecp256r1HashFormat) -> Self {
        CryptoError::invalid_hash_format()
    }
}

fn read_hash(data: &[u8]) -> Result<[u8; 32], InvalidSecp256r1HashFormat> {
    data.try_into().map_err(|_| InvalidSecp256r1HashFormat)
}

/// Error raised when signature is not 64 bytes long (32 bytes r, 32 bytes s)
struct InvalidSecp256r1SignatureFormat;

impl From<InvalidSecp256r1SignatureFormat> for CryptoError {
    fn from(_original: InvalidSecp256r1SignatureFormat) -> Self {
        CryptoError::invalid_signature_format()
    }
}

fn read_signature(data: &[u8]) -> Result<[u8; 64], InvalidSecp256r1SignatureFormat> {
    data.try_into().map_err(|_| InvalidSecp256r1SignatureFormat)
}

/// Error raised when public key is not in one of the two supported formats:
/// 1. Uncompressed: 65 bytes starting with 0x04
/// 2. Compressed: 33 bytes starting with 0x02 or 0x03
struct InvalidSecp256r1PubkeyFormat;

impl From<InvalidSecp256r1PubkeyFormat> for CryptoError {
    fn from(_original: InvalidSecp256r1PubkeyFormat) -> Self {
        CryptoError::invalid_pubkey_format()
    }
}

fn check_pubkey(data: &[u8]) -> Result<(), InvalidSecp256r1PubkeyFormat> {
    let ok = match data.first() {
        Some(0x02) | Some(0x03) => data.len() == ECDSA_COMPRESSED_PUBKEY_LEN,
        Some(0x04) => data.len() == ECDSA_UNCOMPRESSED_PUBKEY_LEN,
        _ => false,
    };
    if ok {
        Ok(())
    } else {
        Err(InvalidSecp256r1PubkeyFormat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ecdsa::RecoveryId;
    use p256::{
        ecdsa::signature::DigestSigner, ecdsa::SigningKey, elliptic_curve::rand_core::OsRng,
    };
    use serde::Deserialize;
    use sha2::Sha256;

    // For generic signature verification
    const MSG: &str = "Hello World!";

    // Cosmos secp256r1 signature verification
    // tendermint/PubKeySecp256r1 pubkey
    const COSMOS_SECP256R1_PUBKEY_HEX: &str =
        "049a2c7b27b132246e170dfb9167db5c5bd302033dbece2bc3f2541a6cd11851821a775f1fc6c4f89e0d019888057f0d574f1c4eb1f90a7a41c4ea9b99b538d932";

    const COSMOS_SECP256R1_MSG_HEX1: &str = "6265206b696e64";
    // const COSMOS_SECP256R1_MSG_HEX2: &str = "6265206b696e64";
    // const COSMOS_SECP256R1_MSG_HEX3: &str = "6265206b696e64";

    const COSMOS_SECP256R1_SIGNATURE_HEX1: &str = "453020029250fb9eb22b21b881319a123244e463a329356b75ce804fc2dda174e715104621028d009abee7d523894b425d974bc38cfae5d05cdf5a550c8eceae1f20f0c9913f0038";
    // const COSMOS_SECP256R1_SIGNATURE_HEX2: &str = "30450220658fc9271b09bd53edf3a5bd31b7bd99bd3c3de7859cd8dd1133e76ed44fcb580221009e43d091911de0fc90d22960517211f5cf6c624b326759e219326f3af807ac31";
    // const COSMOS_SECP256R1_SIGNATURE_HEX3: &str = "30450220658fc9271b09bd53edf3a5bd31b7bd99bd3c3de7859cd8dd1133e76ed44fcb580221009e43d091911de0fc90d22960517211f5cf6c624b326759e219326f3af807ac31";

    // Test data originally from https://github.com/cosmos/cosmjs/blob/v0.24.0-alpha.22/packages/crypto/src/secp256k1.spec.ts#L195-L394
    const COSMOS_SECP256R1_TESTS_JSON: &str = "./testdata/secp256r1_tests.json";

    #[test]
    fn test_secp256r1_verify() {
        // Explicit / external hashing
        let message_digest = Sha256::new().chain(MSG);
        let message_hash = message_digest.clone().finalize();

        // Signing
        let secret_key = SigningKey::random(&mut OsRng); // Serialize with `::to_bytes()`

        // Note: the signature type must be annotated or otherwise inferrable as
        // `Signer` has many impls of the `Signer` trait (for both regular and
        // recoverable signature types).
        let (signature, _recovery_id): (Signature, RecoveryId) =
            secret_key.sign_digest(message_digest);

        let public_key = VerifyingKey::from(&secret_key); // Serialize with `::to_encoded_point()`

        // Verification (uncompressed public key)
        assert!(secp256r1_verify(
            &message_hash,
            signature.to_bytes().as_slice(),
            public_key.to_encoded_point(false).as_bytes()
        )
        .unwrap());

        // Verification (compressed public key)
        assert!(secp256r1_verify(
            &message_hash,
            signature.to_bytes().as_slice(),
            public_key.to_encoded_point(true).as_bytes()
        )
        .unwrap());

        // Wrong message fails
        let bad_message_hash = Sha256::new().chain(MSG).chain("\0").finalize();
        assert!(!secp256r1_verify(
            &bad_message_hash,
            signature.to_bytes().as_slice(),
            public_key.to_encoded_point(false).as_bytes()
        )
        .unwrap());

        // Other pubkey fails
        let other_secret_key = SigningKey::random(&mut OsRng);
        let other_public_key = VerifyingKey::from(&other_secret_key);
        assert!(!secp256r1_verify(
            &message_hash,
            signature.to_bytes().as_slice(),
            other_public_key.to_encoded_point(false).as_bytes()
        )
        .unwrap());
    }

    #[test]
    fn test_cosmos_secp256r1_verify() {
        let public_key = hex::decode(COSMOS_SECP256R1_PUBKEY_HEX).unwrap();

        for ((i, msg), sig) in (1..)
            .zip(&[
                COSMOS_SECP256R1_MSG_HEX1,
                //COSMOS_SECP256R1_MSG_HEX2,
                //COSMOS_SECP256R1_MSG_HEX3,
            ])
            .zip(&[
                COSMOS_SECP256R1_SIGNATURE_HEX1,
                //COSMOS_SECP256R1_SIGNATURE_HEX2,
                //COSMOS_SECP256R1_SIGNATURE_HEX3,
            ])
        {
            let message = hex::decode(msg).unwrap();
            let signature = hex::decode(sig).unwrap();

            // Explicit hash
            let message_hash = Sha256::digest(&message);

            // secp256r1_verify works
            let valid = secp256r1_verify(&message_hash, &signature, &public_key).unwrap();
            assert!(valid, "secp256r1_verify() failed (test case {i})",);
        }
    }

    #[test]
    #[ignore]
    fn test_cosmos_extra_secp256r1_verify() {
        use std::fs::File;
        use std::io::BufReader;

        use serde::Deserialize;

        #[derive(Deserialize, Debug)]
        struct Encoded {
            message: String,
            message_hash: String,
            signature: String,
            #[serde(rename = "pubkey")]
            public_key: String,
        }

        // Open the file in read-only mode with buffer.
        let file = File::open(COSMOS_SECP256R1_TESTS_JSON).unwrap();
        let reader = BufReader::new(file);

        let codes: Vec<Encoded> = serde_json::from_reader(reader).unwrap();

        for (i, encoded) in (1..).zip(codes) {
            let message = hex::decode(&encoded.message).unwrap();

            let hash = hex::decode(&encoded.message_hash).unwrap();
            let message_hash = Sha256::digest(&message);
            assert_eq!(hash.as_slice(), message_hash.as_slice());

            let signature = hex::decode(&encoded.signature).unwrap();

            let public_key = hex::decode(&encoded.public_key).unwrap();

            // secp256r1_verify() works
            let valid = secp256r1_verify(&message_hash, &signature, &public_key).unwrap();
            assert!(
                valid,
                "secp256r1_verify failed (test case {i} in {COSMOS_SECP256R1_TESTS_JSON})"
            );
        }
    }
}
