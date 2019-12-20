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
        ReadonlyPrefixedStorage{
            prefix: length_prefix(prefix),
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
        PrefixedStorage{
            prefix: length_prefix(prefix),
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock::MockStorage;

    #[test]
    pub fn prefix_safe() {
        let mut storage = MockStorage::new();

        // we use a block scope here to release the &mut before we use it in the next storage
        {
            let mut foo = PrefixedStorage::new(b"foo", &mut storage);
            foo.set(b"bar", b"gotcha");
            assert_eq!(Some(b"gotcha".to_vec()), foo.get(b"bar"));
        }

        // try readonly correctly
        {
            let rfoo = ReadonlyPrefixedStorage::new(b"foo", &storage);
            assert_eq!(Some(b"gotcha".to_vec()), rfoo.get(b"bar"));
        }

        // no collisions with other prefixes
        {
            let fo = ReadonlyPrefixedStorage::new(b"fo", &storage);
            assert_eq!(None, fo.get(b"obar"));
        }
    }
}