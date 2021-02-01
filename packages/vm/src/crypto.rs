use secp256k1::{Secp256k1, VerifyOnly};

#[derive(Debug)]
pub struct SignatureVerification {
    secp: Secp256k1<VerifyOnly>,
}

impl SignatureVerification {
    #[allow(dead_code)]
    pub fn new() -> SignatureVerification {
        SignatureVerification {
            secp: Secp256k1::verification_only(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use secp256k1::bitcoin_hashes::sha256;
    use secp256k1::{Message, PublicKey, Signature};

    use secp256k1::rand::rngs::SmallRng;
    use secp256k1::rand::SeedableRng;

    // Create small fast (insecure) RNG with a fixed seed (just for testing)
    const SEED: [u8; 16] = [1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 6, 0, 7, 0, 8, 0];

    // Generic signature verification
    const MSG: &str = "Hello World!";

    // Cosmos signature verification
    const COSMOS_MSG_HEX: &str = "0a93010a90010a1c2f636f736d6f732e62616e6b2e763162657461312e4d736753656e6412700a2d636f736d6f7331706b707472653766646b6c366766727a6c65736a6a766878686c63337234676d6d6b38727336122d636f736d6f7331717970717870713971637273737a673270767871367273307a716733797963356c7a763778751a100a0575636f736d12073132333435363712650a4e0a460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a21034f04181eeba35391b858633a765c4a0c189697b40d216354d50890d350c7029012040a02080112130a0d0a0575636f736d12043230303010c09a0c1a0c73696d642d74657374696e672001";
    // tendermint/PubKeySecp256k1 pubkey
    const COSMOS_PUBKEY_BASE64: &str = "A08EGB7ro1ORuFhjOnZcSgwYlpe0DSFjVNUIkNNQxwKQ";
    const COSMOS_SIGNATURE_HEX: &str = "c9dd20e07464d3a688ff4b710b1fbc027e495e797cfa0b4804da2ed117959227772de059808f765aa29b8f92edf30f4c2c5a438e30d3fe6897daa7141e3ce6f9";

    #[test]
    fn secp256k1_verify() {
        let mut rng = SmallRng::from_seed(SEED);

        // Create full-featured secp context (for testing)
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut rng);
        let message = Message::from_hashed_data::<sha256::Hash>(MSG.as_bytes());
        let signature = secp.sign(&message, &secret_key);

        // Create our verification-only context
        let crypto = SignatureVerification::new();

        // Verify works
        assert!(crypto
            .secp
            .verify(&message, &signature, &public_key)
            .is_ok());

        // Wrong message fails
        let message_bad =
            Message::from_hashed_data::<sha256::Hash>([MSG, "\0"].concat().as_bytes());
        assert!(crypto
            .secp
            .verify(&message_bad, &signature, &public_key)
            .is_err());

        // Wrong pubkey fails
        let (_, public_key_other) = secp.generate_keypair(&mut rng);
        assert!(crypto
            .secp
            .verify(&message, &signature, &public_key_other)
            .is_err());
    }

    #[test]
    fn cosmos_secp256k1_verify() {
        let message =
            Message::from_hashed_data::<sha256::Hash>(&hex::decode(COSMOS_MSG_HEX).unwrap());
        let signature =
            Signature::from_compact(&hex::decode(COSMOS_SIGNATURE_HEX).unwrap()).unwrap();
        let public_key =
            PublicKey::from_slice(&base64::decode(COSMOS_PUBKEY_BASE64).unwrap()).unwrap();

        // Create our verification-only context
        let crypto = SignatureVerification::new();

        // Verify works
        assert!(crypto
            .secp
            .verify(&message, &signature, &public_key)
            .is_ok());
    }
}
