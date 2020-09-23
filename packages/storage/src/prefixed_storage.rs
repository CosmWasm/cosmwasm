#[cfg(feature = "iterator")]
use cosmwasm_std::{Order, KV};
use cosmwasm_std::{ReadonlyStorage, Storage};

use crate::length_prefixed::{to_length_prefixed, to_length_prefixed_nested};
#[cfg(feature = "iterator")]
use crate::namespace_helpers::range_with_prefix;
use crate::namespace_helpers::{get_with_prefix, remove_with_prefix, set_with_prefix};

/// An alias of PrefixedStorage::new for less verbose usage
pub fn prefixed<'a, S>(storage: &'a mut S, namespace: &[u8]) -> PrefixedStorage<'a, S>
where
    S: Storage,
{
    PrefixedStorage::new(storage, namespace)
}

/// An alias of ReadonlyPrefixedStorage::new for less verbose usage
pub fn prefixed_read<'a, S>(storage: &'a S, namespace: &[u8]) -> ReadonlyPrefixedStorage<'a, S>
where
    S: ReadonlyStorage,
{
    ReadonlyPrefixedStorage::new(storage, namespace)
}

pub struct ReadonlyPrefixedStorage<'a, S>
where
    S: ReadonlyStorage,
{
    storage: &'a S,
    prefix: Vec<u8>,
}

impl<'a, S: ReadonlyStorage> ReadonlyPrefixedStorage<'a, S>
where
    S: ReadonlyStorage,
{
    pub fn new(storage: &'a S, namespace: &[u8]) -> Self {
        ReadonlyPrefixedStorage {
            prefix: to_length_prefixed(namespace),
            storage,
        }
    }

    // Nested namespaces as documented in
    // https://github.com/webmaster128/key-namespacing#nesting
    pub fn multilevel(storage: &'a S, namespaces: &[&[u8]]) -> Self {
        ReadonlyPrefixedStorage {
            prefix: to_length_prefixed_nested(namespaces),
            storage,
        }
    }
}

impl<'a, S> ReadonlyStorage for ReadonlyPrefixedStorage<'a, S>
where
    S: ReadonlyStorage,
{
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        get_with_prefix(self.storage, &self.prefix, key)
    }

    #[cfg(feature = "iterator")]
    /// range allows iteration over a set of keys, either forwards or backwards
    fn range<'b>(
        &'b self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = KV> + 'b> {
        range_with_prefix(self.storage, &self.prefix, start, end, order)
    }
}

pub struct PrefixedStorage<'a, S>
where
    S: Storage,
{
    storage: &'a mut S,
    prefix: Vec<u8>,
}

impl<'a, S> PrefixedStorage<'a, S>
where
    S: Storage,
{
    pub fn new(storage: &'a mut S, namespace: &[u8]) -> Self {
        PrefixedStorage {
            storage,
            prefix: to_length_prefixed(namespace),
        }
    }

    // Nested namespaces as documented in
    // https://github.com/webmaster128/key-namespacing#nesting
    pub fn multilevel(storage: &'a mut S, namespaces: &[&[u8]]) -> Self {
        PrefixedStorage {
            storage,
            prefix: to_length_prefixed_nested(namespaces),
        }
    }
}

impl<'a, S> ReadonlyStorage for PrefixedStorage<'a, S>
where
    S: Storage,
{
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        get_with_prefix(self.storage, &self.prefix, key)
    }

    #[cfg(feature = "iterator")]
    /// range allows iteration over a set of keys, either forwards or backwards
    /// uses standard rust range notation, and eg db.range(b"foo"..b"bar") also works reverse
    fn range<'b>(
        &'b self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = KV> + 'b> {
        range_with_prefix(self.storage, &self.prefix, start, end, order)
    }
}

impl<'a, S> Storage for PrefixedStorage<'a, S>
where
    S: Storage,
{
    fn set(&mut self, key: &[u8], value: &[u8]) {
        set_with_prefix(self.storage, &self.prefix, key, value);
    }

    fn remove(&mut self, key: &[u8]) {
        remove_with_prefix(self.storage, &self.prefix, key);
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
        let mut foo = PrefixedStorage::new(&mut storage, b"foo");
        foo.set(b"bar", b"gotcha");
        assert_eq!(foo.get(b"bar"), Some(b"gotcha".to_vec()));

        // try readonly correctly
        let rfoo = ReadonlyPrefixedStorage::new(&storage, b"foo");
        assert_eq!(rfoo.get(b"bar"), Some(b"gotcha".to_vec()));

        // no collisions with other prefixes
        let fo = ReadonlyPrefixedStorage::new(&storage, b"fo");
        assert_eq!(fo.get(b"obar"), None);

        // Note: explicit scoping is not required, but you must not refer to `foo` anytime after you
        // initialize a different PrefixedStorage. Uncomment this to see errors:
        //        assert_eq!(Some(b"gotcha".to_vec()), foo.get(b"bar"));
    }

    #[test]
    fn multi_level() {
        let mut storage = MockStorage::new();

        // set with nested
        let mut foo = PrefixedStorage::new(&mut storage, b"foo");
        let mut bar = PrefixedStorage::new(&mut foo, b"bar");
        bar.set(b"baz", b"winner");

        // we can nest them the same encoding with one operation
        let loader = ReadonlyPrefixedStorage::multilevel(&storage, &[b"foo", b"bar"]);
        assert_eq!(loader.get(b"baz"), Some(b"winner".to_vec()));

        // set with multilevel
        let mut foobar = PrefixedStorage::multilevel(&mut storage, &[b"foo", b"bar"]);
        foobar.set(b"second", b"time");

        let a = ReadonlyPrefixedStorage::new(&storage, b"foo");
        let b = ReadonlyPrefixedStorage::new(&a, b"bar");
        assert_eq!(b.get(b"second"), Some(b"time".to_vec()));
    }
}
