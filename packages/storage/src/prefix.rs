use cosmwasm_std::{ReadonlyStorage, Storage};

use crate::namespace_helpers::{
    get_with_prefix, key_prefix, key_prefix_nested, remove_with_prefix, set_with_prefix,
};

// prefixed_read is a helper function for less verbose usage
pub fn prefixed_read<'a, T: ReadonlyStorage>(
    prefix: &[u8],
    storage: &'a T,
) -> ReadonlyPrefixedStorage<'a, T> {
    ReadonlyPrefixedStorage::new(prefix, storage)
}

// prefixed_rw is a helper function for less verbose usage
pub fn prefixed<'a, T: Storage>(prefix: &[u8], storage: &'a mut T) -> PrefixedStorage<'a, T> {
    PrefixedStorage::new(prefix, storage)
}

pub struct ReadonlyPrefixedStorage<'a, T: ReadonlyStorage> {
    prefix: Vec<u8>,
    storage: &'a T,
}

impl<'a, T: ReadonlyStorage> ReadonlyPrefixedStorage<'a, T> {
    pub fn new(namespace: &[u8], storage: &'a T) -> Self {
        ReadonlyPrefixedStorage {
            prefix: key_prefix(namespace),
            storage,
        }
    }

    // Nested namespaces as documented in
    // https://github.com/webmaster128/key-namespacing#nesting
    pub fn multilevel(namespaces: &[&[u8]], storage: &'a T) -> Self {
        ReadonlyPrefixedStorage {
            prefix: key_prefix_nested(namespaces),
            storage,
        }
    }
}

impl<'a, T: ReadonlyStorage> ReadonlyStorage for ReadonlyPrefixedStorage<'a, T> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        get_with_prefix(self.storage, &self.prefix, key)
    }
}

pub struct PrefixedStorage<'a, T: Storage> {
    prefix: Vec<u8>,
    storage: &'a mut T,
}

impl<'a, T: Storage> PrefixedStorage<'a, T> {
    pub fn new(namespace: &[u8], storage: &'a mut T) -> Self {
        PrefixedStorage {
            prefix: key_prefix(namespace),
            storage,
        }
    }

    // Nested namespaces as documented in
    // https://github.com/webmaster128/key-namespacing#nesting
    pub fn multilevel(namespaces: &[&[u8]], storage: &'a mut T) -> Self {
        PrefixedStorage {
            prefix: key_prefix_nested(namespaces),
            storage,
        }
    }
}

impl<'a, T: Storage> ReadonlyStorage for PrefixedStorage<'a, T> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        get_with_prefix(self.storage, &self.prefix, key)
    }
}

impl<'a, T: Storage> Storage for PrefixedStorage<'a, T> {
    fn set(&mut self, key: &[u8], value: &[u8]) {
        set_with_prefix(self.storage, &self.prefix, key, value)
    }

    fn remove(&mut self, key: &[u8]) {
        remove_with_prefix(self.storage, &self.prefix, key)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::MockStorage;

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
