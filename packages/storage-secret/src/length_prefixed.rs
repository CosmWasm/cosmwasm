//! This module is an implemention of a namespacing scheme described
//! in https://github.com/webmaster128/key-namespacing#length-prefixed-keys
//!
//! Everything in this file is only responsible for building such keys
//! and is in no way specific to any kind of storage.

/// Calculates the raw key prefix for a given namespace as documented
/// in https://github.com/webmaster128/key-namespacing#length-prefixed-keys
pub fn to_length_prefixed(namespace: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(namespace.len() + 2);
    out.extend_from_slice(&encode_length(namespace));
    out.extend_from_slice(namespace);
    out
}

/// Calculates the raw key prefix for a given nested namespace
/// as documented in https://github.com/webmaster128/key-namespacing#nesting
pub fn to_length_prefixed_nested(namespaces: &[&[u8]]) -> Vec<u8> {
    let mut size = 0;
    for &namespace in namespaces {
        size += namespace.len() + 2;
    }

    let mut out = Vec::with_capacity(size);
    for &namespace in namespaces {
        out.extend_from_slice(&encode_length(namespace));
        out.extend_from_slice(namespace);
    }
    out
}

/// Encodes the length of a given namespace as a 2 byte big endian encoded integer
fn encode_length(namespace: &[u8]) -> [u8; 2] {
    if namespace.len() > 0xFFFF {
        panic!("only supports namespaces up to length 0xFFFF")
    }
    let length_bytes = (namespace.len() as u32).to_be_bytes();
    [length_bytes[2], length_bytes[3]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_length_prefixed_works() {
        assert_eq!(to_length_prefixed(b""), b"\x00\x00");
        assert_eq!(to_length_prefixed(b"a"), b"\x00\x01a");
        assert_eq!(to_length_prefixed(b"ab"), b"\x00\x02ab");
        assert_eq!(to_length_prefixed(b"abc"), b"\x00\x03abc");
    }

    #[test]
    fn to_length_prefixed_works_for_long_prefix() {
        let long_namespace1 = vec![0; 256];
        let prefix1 = to_length_prefixed(&long_namespace1);
        assert_eq!(prefix1.len(), 256 + 2);
        assert_eq!(&prefix1[0..2], b"\x01\x00");

        let long_namespace2 = vec![0; 30000];
        let prefix2 = to_length_prefixed(&long_namespace2);
        assert_eq!(prefix2.len(), 30000 + 2);
        assert_eq!(&prefix2[0..2], b"\x75\x30");

        let long_namespace3 = vec![0; 0xFFFF];
        let prefix3 = to_length_prefixed(&long_namespace3);
        assert_eq!(prefix3.len(), 0xFFFF + 2);
        assert_eq!(&prefix3[0..2], b"\xFF\xFF");
    }

    #[test]
    #[should_panic(expected = "only supports namespaces up to length 0xFFFF")]
    fn to_length_prefixed_panics_for_too_long_prefix() {
        let limit = 0xFFFF;
        let long_namespace = vec![0; limit + 1];
        to_length_prefixed(&long_namespace);
    }

    #[test]
    fn to_length_prefixed_calculates_capacity_correctly() {
        // Those tests cannot guarantee the required capacity was calculated correctly before
        // the vector allocation but increase the likelyhood of a proper implementation.

        let key = to_length_prefixed(b"");
        assert_eq!(key.capacity(), key.len());

        let key = to_length_prefixed(b"h");
        assert_eq!(key.capacity(), key.len());

        let key = to_length_prefixed(b"hij");
        assert_eq!(key.capacity(), key.len());
    }

    #[test]
    fn to_length_prefixed_nested_works() {
        assert_eq!(to_length_prefixed_nested(&[]), b"");
        assert_eq!(to_length_prefixed_nested(&[b""]), b"\x00\x00");
        assert_eq!(to_length_prefixed_nested(&[b"", b""]), b"\x00\x00\x00\x00");

        assert_eq!(to_length_prefixed_nested(&[b"a"]), b"\x00\x01a");
        assert_eq!(
            to_length_prefixed_nested(&[b"a", b"ab"]),
            b"\x00\x01a\x00\x02ab"
        );
        assert_eq!(
            to_length_prefixed_nested(&[b"a", b"ab", b"abc"]),
            b"\x00\x01a\x00\x02ab\x00\x03abc"
        );
    }

    #[test]
    fn to_length_prefixed_nested_allows_many_long_namespaces() {
        // The 0xFFFF limit is for each namespace, not for the combination of them

        let long_namespace1 = vec![0xaa; 0xFFFD];
        let long_namespace2 = vec![0xbb; 0xFFFE];
        let long_namespace3 = vec![0xcc; 0xFFFF];

        let prefix =
            to_length_prefixed_nested(&[&long_namespace1, &long_namespace2, &long_namespace3]);
        assert_eq!(&prefix[0..2], b"\xFF\xFD");
        assert_eq!(&prefix[2..(2 + 0xFFFD)], long_namespace1.as_slice());
        assert_eq!(&prefix[(2 + 0xFFFD)..(2 + 0xFFFD + 2)], b"\xFF\xFe");
        assert_eq!(
            &prefix[(2 + 0xFFFD + 2)..(2 + 0xFFFD + 2 + 0xFFFE)],
            long_namespace2.as_slice()
        );
        assert_eq!(
            &prefix[(2 + 0xFFFD + 2 + 0xFFFE)..(2 + 0xFFFD + 2 + 0xFFFE + 2)],
            b"\xFF\xFf"
        );
        assert_eq!(
            &prefix[(2 + 0xFFFD + 2 + 0xFFFE + 2)..(2 + 0xFFFD + 2 + 0xFFFE + 2 + 0xFFFF)],
            long_namespace3.as_slice()
        );
    }

    #[test]
    fn to_length_prefixed_nested_calculates_capacity_correctly() {
        // Those tests cannot guarantee the required capacity was calculated correctly before
        // the vector allocation but increase the likelyhood of a proper implementation.

        let key = to_length_prefixed_nested(&[]);
        assert_eq!(key.capacity(), key.len());

        let key = to_length_prefixed_nested(&[b""]);
        assert_eq!(key.capacity(), key.len());

        let key = to_length_prefixed_nested(&[b"a"]);
        assert_eq!(key.capacity(), key.len());

        let key = to_length_prefixed_nested(&[b"a", b"bc"]);
        assert_eq!(key.capacity(), key.len());

        let key = to_length_prefixed_nested(&[b"a", b"bc", b"def"]);
        assert_eq!(key.capacity(), key.len());
    }

    #[test]
    fn encode_length_works() {
        assert_eq!(encode_length(b""), *b"\x00\x00");
        assert_eq!(encode_length(b"a"), *b"\x00\x01");
        assert_eq!(encode_length(b"aa"), *b"\x00\x02");
        assert_eq!(encode_length(b"aaa"), *b"\x00\x03");
        assert_eq!(encode_length(&vec![1; 255]), *b"\x00\xff");
        assert_eq!(encode_length(&vec![1; 256]), *b"\x01\x00");
        assert_eq!(encode_length(&vec![1; 12345]), *b"\x30\x39");
        assert_eq!(encode_length(&vec![1; 65535]), *b"\xff\xff");
    }

    #[test]
    #[should_panic(expected = "only supports namespaces up to length 0xFFFF")]
    fn encode_length_panics_for_large_values() {
        encode_length(&vec![1; 65536]);
    }
}
