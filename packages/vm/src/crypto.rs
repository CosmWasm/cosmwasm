use ed25519_zebra as ed25519;

use k256::{
    ecdsa::signature::{DigestVerifier, Signature as _}, // traits
    ecdsa::{Signature, VerifyingKey},                   // type aliases
};
use sha2::Digest; // trait

use std::convert::TryFrom;

use crate::errors::{VmError, VmResult};
use crate::identity_digest::Identity256;

pub fn ed25519_verify(
    message: &[u8],
    signature_bytes: &[u8],
    public_key_bytes: &[u8],
) -> VmResult<()> {
    // Deserialize
    let res = ed25519::Signature::try_from(signature_bytes);
    let signature = res.map_err(|err| VmError::crypto_err(err.to_string()))?;

    ed25519::VerificationKey::try_from(public_key_bytes)
        .and_then(|vk| vk.verify(&signature, &message))
        .map_err(|err| VmError::crypto_err(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    use elliptic_curve::sec1::ToEncodedPoint;
    use rand_core::OsRng;

    use crate::crypto::ed25519_verify;
        ecdsa::signature::Signature as _,
        ecdsa::{signature::Signer, Signature, SigningKey},
        ecdsa::{signature::Verifier, VerifyingKey},
    use k256::{
        ecdsa::signature::DigestSigner, // trait
        ecdsa::SigningKey,              // type alias
    };

    use elliptic_curve::rand_core::OsRng;
    // use bitcoin_hashes::{sha256, Hash};

    // Generic signature verification
    const MSG: &str = "Hello World!";

    // Cosmos signature verification
    // tendermint/PubKeySecp256k1 pubkey
    const COSMOS_PUBKEY_BASE64: &str = "A08EGB7ro1ORuFhjOnZcSgwYlpe0DSFjVNUIkNNQxwKQ";

    const COSMOS_MSG_HEX1: &str = "0a93010a90010a1c2f636f736d6f732e62616e6b2e763162657461312e4d736753656e6412700a2d636f736d6f7331706b707472653766646b6c366766727a6c65736a6a766878686c63337234676d6d6b38727336122d636f736d6f7331717970717870713971637273737a673270767871367273307a716733797963356c7a763778751a100a0575636f736d12073132333435363712650a4e0a460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a21034f04181eeba35391b858633a765c4a0c189697b40d216354d50890d350c7029012040a02080112130a0d0a0575636f736d12043230303010c09a0c1a0c73696d642d74657374696e672001";
    const COSMOS_MSG_HEX2: &str = "0a93010a90010a1c2f636f736d6f732e62616e6b2e763162657461312e4d736753656e6412700a2d636f736d6f7331706b707472653766646b6c366766727a6c65736a6a766878686c63337234676d6d6b38727336122d636f736d6f7331717970717870713971637273737a673270767871367273307a716733797963356c7a763778751a100a0575636f736d12073132333435363712670a500a460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a21034f04181eeba35391b858633a765c4a0c189697b40d216354d50890d350c7029012040a020801180112130a0d0a0575636f736d12043230303010c09a0c1a0c73696d642d74657374696e672001";
    const COSMOS_MSG_HEX3: &str = "0a93010a90010a1c2f636f736d6f732e62616e6b2e763162657461312e4d736753656e6412700a2d636f736d6f7331706b707472653766646b6c366766727a6c65736a6a766878686c63337234676d6d6b38727336122d636f736d6f7331717970717870713971637273737a673270767871367273307a716733797963356c7a763778751a100a0575636f736d12073132333435363712670a500a460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a21034f04181eeba35391b858633a765c4a0c189697b40d216354d50890d350c7029012040a020801180212130a0d0a0575636f736d12043230303010c09a0c1a0c73696d642d74657374696e672001";

    const COSMOS_SIGNATURE_HEX1: &str = "c9dd20e07464d3a688ff4b710b1fbc027e495e797cfa0b4804da2ed117959227772de059808f765aa29b8f92edf30f4c2c5a438e30d3fe6897daa7141e3ce6f9";
    const COSMOS_SIGNATURE_HEX2: &str = "525adc7e61565a509c60497b798c549fbf217bb5cd31b24cc9b419d098cc95330c99ecc4bc72448f85c365a4e3f91299a3d40412fb3751bab82f1940a83a0a4c";
    const COSMOS_SIGNATURE_HEX3: &str = "f3f2ca73806f2abbf6e0fe85f9b8af66f0e9f7f79051fdb8abe5bb8633b17da132e82d577b9d5f7a6dae57a144efc9ccc6eef15167b44b3b22a57240109762af";

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
        assert!(ed25519_verify(&message, &signature_bytes, &public_key_bytes).is_ok());

        // Wrong message fails
        let bad_message = [message, b"\0"].concat();
        assert!(ed25519_verify(&bad_message, &signature_bytes, &public_key_bytes).is_err());

        // Other pubkey fails
        let other_secret_key = ed25519::SigningKey::new(&mut OsRng);
        let other_public_key = ed25519::VerificationKey::from(&other_secret_key);
        let other_public_key_bytes: [u8; 32] = other_public_key.into();
        assert!(ed25519_verify(&message, &signature_bytes, &other_public_key_bytes).is_err());
    }
}
