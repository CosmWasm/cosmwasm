use sha1::{Digest, Sha1};

pub fn sha1_calculate(message: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(message);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha1_calculate() {
        let message = b"The quick brown fox jumps over the lazy dog";
        let result = sha1_calculate(message);
        assert_eq!(
            result.to_vec(),
            hex::decode("2fd4e1c67a2d28fced849ee1bb76e7391b93eb12").unwrap()
        )
    }
}
