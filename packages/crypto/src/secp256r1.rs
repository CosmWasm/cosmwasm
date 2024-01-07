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

    // Cosmos secp256r1 signature verification. Matches tendermint/PubKeySecp256r1 pubkey format.
    // ECDSA/P-256 test vectors adapted from the FIPS 186-4 ECDSA test vectors (P-256, SHA-256, from
    // `SigGen.txt` in `186-4ecdsatestvectors.zip`).
    // <https://csrc.nist.gov/projects/cryptographic-algorithm-validation-program/digital-signatures>
    const COSMOS_SECP256R1_PUBKEY_HEX1: &str =
        "041ccbe91c075fc7f4f033bfa248db8fccd3565de94bbfb12f3c59ff46c271bf83ce4014c68811f9a21a1fdb2c0e6113e06db7ca93b7404e78dc7ccd5ca89a4ca9";
    const COSMOS_SECP256R1_PUBKEY_HEX2: &str = "04e266ddfdc12668db30d4ca3e8f7749432c416044f2d2b8c10bf3d4012aeffa8abfa86404a2e9ffe67d47c587ef7a97a7f456b863b4d02cfc6928973ab5b1cb39";
    const COSMOS_SECP256R1_PUBKEY_HEX3: &str = "0474ccd8a62fba0e667c50929a53f78c21b8ff0c3c737b0b40b1750b2302b0bde829074e21f3a0ef88b9efdf10d06aa4c295cc1671f758ca0e4cd108803d0f2614";

    const COSMOS_SECP256R1_MSG_HEX1: &str = "5905238877c77421f73e43ee3da6f2d9e2ccad5fc942dcec0cbd25482935faaf416983fe165b1a045ee2bcd2e6dca3bdf46c4310a7461f9a37960ca672d3feb5473e253605fb1ddfd28065b53cb5858a8ad28175bf9bd386a5e471ea7a65c17cc934a9d791e91491eb3754d03799790fe2d308d16146d5c9b0d0debd97d79ce8";
    const COSMOS_SECP256R1_MSG_HEX2: &str = "c35e2f092553c55772926bdbe87c9796827d17024dbb9233a545366e2e5987dd344deb72df987144b8c6c43bc41b654b94cc856e16b96d7a821c8ec039b503e3d86728c494a967d83011a0e090b5d54cd47f4e366c0912bc808fbb2ea96efac88fb3ebec9342738e225f7c7c2b011ce375b56621a20642b4d36e060db4524af1";
    const COSMOS_SECP256R1_MSG_HEX3: &str = "3c054e333a94259c36af09ab5b4ff9beb3492f8d5b4282d16801daccb29f70fe61a0b37ffef5c04cd1b70e85b1f549a1c4dc672985e50f43ea037efa9964f096b5f62f7ffdf8d6bfb2cc859558f5a393cb949dbd48f269343b5263dcdb9c556eca074f2e98e6d94c2c29a677afaf806edf79b15a3fcd46e7067b7669f83188ee";

    const COSMOS_SECP256R1_SIGNATURE_HEX1: &str = "f3ac8061b514795b8843e3d6629527ed2afd6b1f6a555a7acabb5e6f79c8c2ac8bf77819ca05a6b2786c76262bf7371cef97b218e96f175a3ccdda2acc058903";
    const COSMOS_SECP256R1_SIGNATURE_HEX2: &str = "976d3a4e9d23326dc0baa9fa560b7c4e53f42864f508483a6473b6a11079b2db1b766e9ceb71ba6c01dcd46e0af462cd4cfa652ae5017d4555b8eeefe36e1932";
    const COSMOS_SECP256R1_SIGNATURE_HEX3: &str = "35fb60f5ca0f3ca08542fb3cc641c8263a2cab7a90ee6a5e1583fac2bb6f6bd1ee59d81bc9db1055cc0ed97b159d8784af04e98511d0a9a407b99bb292572e96";

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
        for (((i, pk), msg), sig) in (1..)
            .zip(&[
                COSMOS_SECP256R1_PUBKEY_HEX1,
                COSMOS_SECP256R1_PUBKEY_HEX2,
                COSMOS_SECP256R1_PUBKEY_HEX3,
             ])
            .zip(&[
                COSMOS_SECP256R1_MSG_HEX1,
                COSMOS_SECP256R1_MSG_HEX2,
                COSMOS_SECP256R1_MSG_HEX3,
            ])
            .zip(&[
                COSMOS_SECP256R1_SIGNATURE_HEX1,
                COSMOS_SECP256R1_SIGNATURE_HEX2,
                COSMOS_SECP256R1_SIGNATURE_HEX3,
            ])
        {
            let public_key = hex::decode(pk).unwrap();
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
