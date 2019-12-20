// This is demo code that is not used by other modules,
// but serves as a proof of concept.
// This can be migrated to other modules when it reaches final implementation
#![allow(dead_code)]

use crate::traits::{ReadonlyStorage, Storage};

//#[derive(Clone)]
pub struct ReadonlyPrefixedStorage<'a, T: ReadonlyStorage> {
    prefix: Vec<u8>,
    storage: &'a T,
}

impl<'a, T: ReadonlyStorage> ReadonlyPrefixedStorage<'a, T> {
    fn new(prefix: &[u8], storage: &'a T) -> Self {
        ReadonlyPrefixedStorage {
            prefix: length_prefix(prefix),
            storage,
        }
    }

    fn multilevel(prefixes: &[&[u8]], storage: &'a T) -> Self {
        ReadonlyPrefixedStorage {
            prefix: multi_length_prefix(prefixes),
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

//#[derive(Clone)]
pub struct PrefixedStorage<'a, T: Storage> {
    prefix: Vec<u8>,
    storage: &'a mut T,
}

impl<'a, T: Storage> PrefixedStorage<'a, T> {
    fn new(prefix: &[u8], storage: &'a mut T) -> Self {
        PrefixedStorage {
            prefix: length_prefix(prefix),
            storage,
        }
    }

    fn multilevel(prefixes: &[&[u8]], storage: &'a mut T) -> Self {
        PrefixedStorage {
            prefix: multi_length_prefix(prefixes),
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

// prepend length and store this
fn length_prefix(prefix: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(prefix.len() + 1);
    if prefix.len() > 255 {
        panic!("only supports prefixes up to length 255")
    }
    v.push(prefix.len() as u8);
    v.extend_from_slice(prefix);
    v
}

// prepend length and store this
fn multi_length_prefix(prefixes: &[&[u8]]) -> Vec<u8> {
    let mut size = prefixes.len();
    for &p in prefixes {
        size += p.len();
    }

    let mut v = Vec::with_capacity(size);
    for &p in prefixes {
        if p.len() > 255 {
            panic!("only supports prefixes up to length 255")
        }
        v.push(p.len() as u8);
        v.extend_from_slice(p);
    }
    v
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::mock::MockStorage;

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
        let mut bar = PrefixedStorage::new(b"bar",&mut foo);
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
