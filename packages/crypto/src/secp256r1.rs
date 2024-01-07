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
    // ECDSA/P-256 test vectors adapted from the FIPS 186-4 ECDSA test vectors.
    // (P-256, SHA-256, from `SigGen.txt` in `186-4ecdsatestvectors.zip`)
    // <https://csrc.nist.gov/projects/cryptographic-algorithm-validation-program/digital-signatures>
    const COSMOS_SECP256R1_PUBKEY_HEX: &str =
        "041ccbe91c075fc7f4f033bfa248db8fccd3565de94bbfb12f3c59ff46c271bf83ce4014c68811f9a21a1fdb2c0e6113e06db7ca93b7404e78dc7ccd5ca89a4ca9";

    const COSMOS_SECP256R1_MSG_HEX1: &str = "5905238877c77421f73e43ee3da6f2d9e2ccad5fc942dcec0cbd25482935faaf416983fe165b1a045ee2bcd2e6dca3bdf46c4310a7461f9a37960ca672d3feb5473e253605fb1ddfd28065b53cb5858a8ad28175bf9bd386a5e471ea7a65c17cc934a9d791e91491eb3754d03799790fe2d308d16146d5c9b0d0debd97d79ce8";
    // const COSMOS_SECP256R1_MSG_HEX2: &str = "";
    // const COSMOS_SECP256R1_MSG_HEX3: &str = "";

    const COSMOS_SECP256R1_SIGNATURE_HEX1: &str = "f3ac8061b514795b8843e3d6629527ed2afd6b1f6a555a7acabb5e6f79c8c2ac8bf77819ca05a6b2786c76262bf7371cef97b218e96f175a3ccdda2acc058903";
    // const COSMOS_SECP256R1_SIGNATURE_HEX2: &str = "";
    // const COSMOS_SECP256R1_SIGNATURE_HEX3: &str = "";

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
