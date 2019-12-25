// This is demo code that is not used by other modules,
// but serves as a proof of concept.
// This can be migrated to other modules when it reaches final implementation
#![allow(dead_code)]

use crate::traits::{ReadonlyStorage, Storage};

// returns the length as a 2 byte big endian encoded integer
fn len(prefix: &[u8]) -> [u8; 2] {
    if prefix.len() > 0xFFFF {
        panic!("only supports namespaces up to length 0xFFFF")
    }
    let length_bytes = (prefix.len() as u64).to_be_bytes();
    [length_bytes[6], length_bytes[7]]
}

// prepend length of the namespace
fn key_prefix(namespace: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(namespace.len() + 2);
    out.extend_from_slice(&len(namespace));
    out.extend_from_slice(namespace);
    out
}

fn multi_key_prefix(namespaces: &[&[u8]]) -> Vec<u8> {
    let mut size = namespaces.len();
    for &namespace in namespaces {
        size += namespace.len() + 2;
    }

    let mut out = Vec::with_capacity(size);
    for &namespace in namespaces {
        let prefix = key_prefix(namespace);
        out.extend_from_slice(&prefix);
    }
    out
}

pub struct ReadonlyPrefixedStorage<'a, T: ReadonlyStorage> {
    prefix: Vec<u8>,
    storage: &'a T,
}

impl<'a, T: ReadonlyStorage> ReadonlyPrefixedStorage<'a, T> {
    fn new(namespace: &[u8], storage: &'a T) -> Self {
        ReadonlyPrefixedStorage {
            prefix: key_prefix(namespace),
            storage,
        }
    }

    // note: multilevel is here for demonstration purposes, but may well be removed
    // before exposing any of these demo apis
    fn multilevel(prefixes: &[&[u8]], storage: &'a T) -> Self {
        ReadonlyPrefixedStorage {
            prefix: multi_key_prefix(prefixes),
            storage,
        }
    }
}

impl<'a, T: ReadonlyStorage> ReadonlyStorage for ReadonlyPrefixedStorage<'a, T> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let mut k = self.prefix.clone();
        k.extend_from_slice(key);
        self.storage.get(&k)
    }
}

pub struct PrefixedStorage<'a, T: Storage> {
    prefix: Vec<u8>,
    storage: &'a mut T,
}

impl<'a, T: Storage> PrefixedStorage<'a, T> {
    fn new(namespace: &[u8], storage: &'a mut T) -> Self {
        PrefixedStorage {
            prefix: key_prefix(namespace),
            storage,
        }
    }

    // note: multilevel is here for demonstration purposes, but may well be removed
    // before exposing any of these demo apis
    fn multilevel(prefixes: &[&[u8]], storage: &'a mut T) -> Self {
        PrefixedStorage {
            prefix: multi_key_prefix(prefixes),
            storage,
        }
    }
}

impl<'a, T: Storage> ReadonlyStorage for PrefixedStorage<'a, T> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let mut k = self.prefix.clone();
        k.extend_from_slice(key);
        self.storage.get(&k)
    }
}

impl<'a, T: Storage> Storage for PrefixedStorage<'a, T> {
    fn set(&mut self, key: &[u8], value: &[u8]) {
        let mut k = self.prefix.clone();
        k.extend_from_slice(key);
        self.storage.set(&k, value)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock::MockStorage;

    #[test]
    fn key_prefix_works() {
        assert_eq!(key_prefix(b""), b"\x00\x00");
        assert_eq!(key_prefix(b"a"), b"\x00\x01a");
        assert_eq!(key_prefix(b"ab"), b"\x00\x02ab");
        assert_eq!(key_prefix(b"abc"), b"\x00\x03abc");
    }

    #[test]
    fn key_prefix_works_for_long_prefix() {
        let long_namespace1 = vec![0; 256];
        let prefix1 = key_prefix(&long_namespace1);
        assert_eq!(prefix1.len(), 256 + 2);
        assert_eq!(&prefix1[0..2], b"\x01\x00");

        let long_namespace2 = vec![0; 30000];
        let prefix2 = key_prefix(&long_namespace2);
        assert_eq!(prefix2.len(), 30000 + 2);
        assert_eq!(&prefix2[0..2], b"\x75\x30");

        let long_namespace3 = vec![0; 0xFFFF];
        let prefix3 = key_prefix(&long_namespace3);
        assert_eq!(prefix3.len(), 0xFFFF + 2);
        assert_eq!(&prefix3[0..2], b"\xFF\xFF");
    }

    #[test]
    #[should_panic(expected = "only supports namespaces up to length 0xFFFF")]
    fn key_prefix_panics_for_too_long_prefix() {
        let limit = 0xFFFF;
        let long_namespace = vec![0; limit + 1];
        key_prefix(&long_namespace);
    }

    #[test]
    fn multi_key_prefix_works() {
        assert_eq!(multi_key_prefix(&[]), b"");
        assert_eq!(multi_key_prefix(&[b""]), b"\x00\x00");
        assert_eq!(multi_key_prefix(&[b"", b""]), b"\x00\x00\x00\x00");

        assert_eq!(multi_key_prefix(&[b"a"]), b"\x00\x01a");
        assert_eq!(multi_key_prefix(&[b"a", b"ab"]), b"\x00\x01a\x00\x02ab");
        assert_eq!(
            multi_key_prefix(&[b"a", b"ab", b"abc"]),
            b"\x00\x01a\x00\x02ab\x00\x03abc"
        );
    }

    #[test]
    fn prefix_safe() {
        let mut storage = MockStorage::new();

        // we use a block scope here to release the &mut before we use it in the next storage
        let mut foo = PrefixedStorage::new(b"foo", &mut storage);
        foo.set(b"bar", b"gotcha");
        assert_eq!(Some(b"gotcha".to_vec()), foo.get(b"bar"));

        // try readonly correctly
        let rfoo = ReadonlyPrefixedStorage::new(b"foo", &storage);
        assert_eq!(Some(b"gotcha".to_vec()), rfoo.get(b"bar"));

        // no collisions with other prefixes
        let fo = ReadonlyPrefixedStorage::new(b"fo", &storage);
        assert_eq!(None, fo.get(b"obar"));

        // Note: explicit scoping is not required, but you must not refer to `foo` anytime after you
        // initialize a different PrefixedStorage. Uncomment this to see errors:
        //        assert_eq!(Some(b"gotcha".to_vec()), foo.get(b"bar"));
    }

    #[test]
    fn multi_level() {
        let mut storage = MockStorage::new();

        // set with nested
        let mut foo = PrefixedStorage::new(b"foo", &mut storage);
        let mut bar = PrefixedStorage::new(b"bar", &mut foo);
        bar.set(b"baz", b"winner");

        // we can nest them the same encoding with one operation
        let loader = ReadonlyPrefixedStorage::multilevel(&[b"foo", b"bar"], &storage);
        assert_eq!(Some(b"winner".to_vec()), loader.get(b"baz"));

        // set with multilevel
        let mut foobar = PrefixedStorage::multilevel(&[b"foo", b"bar"], &mut storage);
        foobar.set(b"second", b"time");

        let a = ReadonlyPrefixedStorage::new(b"foo", &storage);
        let b = ReadonlyPrefixedStorage::new(b"bar", &a);
        assert_eq!(Some(b"time".to_vec()), b.get(b"second"));
    }
}
