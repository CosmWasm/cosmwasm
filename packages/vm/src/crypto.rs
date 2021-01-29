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
    use secp256k1::Message;

    // Small fast (insecure) RNG (just for testing)
    use secp256k1::rand::rngs::SmallRng;
    use secp256k1::rand::SeedableRng;

    const TESTING_MSG: &str = "Hello World!";

    #[test]
    fn secp256k1_verify_works() {
        // Create small fast RNG with a fixed seed.
        let seed = [
            1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 6, 0, 7, 0, 8, 0,
        ];
        let mut rng = SmallRng::from_seed(seed);

        // Create full-featured secp context (for testing)
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut rng);
        let message = Message::from_hashed_data::<sha256::Hash>(TESTING_MSG.as_bytes());
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
            Message::from_hashed_data::<sha256::Hash>([TESTING_MSG, "\0"].concat().as_bytes());
        assert!(crypto
            .secp
            .verify(&message_bad, &signature, &public_key)
            .is_err());

        // Wrong pubkey fails
        let (_, public_key_other) = secp.generate_keypair(&mut rng);
        assert!(crypto
            .secp
            .verify(&message_bad, &signature, &public_key_other)
            .is_err());
    }
}
