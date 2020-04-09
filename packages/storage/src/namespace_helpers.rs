#[cfg(feature = "iterator")]
use cosmwasm_std::{Order, KV};
use cosmwasm_std::{ReadonlyStorage, Result, Storage};

pub(crate) fn get_with_prefix<S: ReadonlyStorage>(
    storage: &S,
    namespace: &[u8],
    key: &[u8],
) -> Result<Option<Vec<u8>>> {
    storage.get(&concat(namespace, key))
}

pub(crate) fn set_with_prefix<S: Storage>(
    storage: &mut S,
    namespace: &[u8],
    key: &[u8],
    value: &[u8],
) -> Result<()> {
    storage.set(&concat(namespace, key), value)
}

pub(crate) fn remove_with_prefix<S: Storage>(
    storage: &mut S,
    namespace: &[u8],
    key: &[u8],
) -> Result<()> {
    storage.remove(&concat(namespace, key))
}

#[inline]
fn concat(namespace: &[u8], key: &[u8]) -> Vec<u8> {
    let mut k = namespace.to_vec();
    k.extend_from_slice(key);
    k
}

#[cfg(feature = "iterator")]
pub(crate) fn range_with_prefix<'a, S: ReadonlyStorage>(
    storage: &'a S,
    namespace: &[u8],
    start: Option<&[u8]>,
    end: Option<&[u8]>,
    order: Order,
) -> Box<dyn Iterator<Item = KV> + 'a> {
    // prepare start, end with prefix
    let start = match start {
        Some(s) => concat(namespace, s),
        None => namespace.to_vec(),
    };
    let end = match end {
        Some(e) => concat(namespace, e),
        // end is updating last byte by one
        None => namespace_upper_bound(namespace),
    };

    // get iterator from storage
    let base_iterator = storage.range(Some(&start), Some(&end), order);

    // make a copy for the closure to handle lifetimes safely
    let prefix = namespace.to_vec();
    let mapped = base_iterator.map(move |(k, v)| (trim(&prefix, &k), v));
    Box::new(mapped)
}

// Calculates the raw key prefix for a given namespace
// as documented in https://github.com/webmaster128/key-namespacing#length-prefixed-keys
pub(crate) fn key_prefix(namespace: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(namespace.len() + 2);
    extend_with_prefix(&mut out, namespace);
    out
}

// Calculates the raw key prefix for a given nested namespace
// as documented in https://github.com/webmaster128/key-namespacing#nesting
pub(crate) fn key_prefix_nested(namespaces: &[&[u8]]) -> Vec<u8> {
    let mut size = namespaces.len();
    for &namespace in namespaces {
        size += namespace.len() + 2;
    }

    let mut out = Vec::with_capacity(size);
    for &namespace in namespaces {
        extend_with_prefix(&mut out, namespace);
    }
    out
}

// extend_with_prefix is only for internal use to unify key_prefix and key_prefix_nested efficiently
// as documented in https://github.com/webmaster128/key-namespacing#nesting
fn extend_with_prefix(out: &mut Vec<u8>, namespace: &[u8]) {
    out.extend_from_slice(&key_len(namespace));
    out.extend_from_slice(namespace);
}

// returns the length as a 2 byte big endian encoded integer
fn key_len(prefix: &[u8]) -> [u8; 2] {
    if prefix.len() > 0xFFFF {
        panic!("only supports namespaces up to length 0xFFFF")
    }
    let length_bytes = (prefix.len() as u64).to_be_bytes();
    [length_bytes[6], length_bytes[7]]
}

#[cfg(feature = "iterator")]
#[inline]
fn trim(namespace: &[u8], key: &[u8]) -> Vec<u8> {
    key[namespace.len()..].to_vec()
}

/// Returns a new vec of same length and last byte incremented by one
/// If last bytes are 255, we handle overflow up the chain.
/// If all bytes are 255, this returns wrong data - but that is never possible as a namespace
#[cfg(feature = "iterator")]
fn namespace_upper_bound(input: &[u8]) -> Vec<u8> {
    let mut copy = input.to_vec();
    // zero out all trailing 255, increment first that is not such
    for i in (0..input.len()).rev() {
        if copy[i] == 255 {
            copy[i] = 0;
        } else {
            copy[i] = copy[i] + 1;
            break;
        }
    }
    copy
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::MockStorage;

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
    fn key_prefix_nested_works() {
        assert_eq!(key_prefix_nested(&[]), b"");
        assert_eq!(key_prefix_nested(&[b""]), b"\x00\x00");
        assert_eq!(key_prefix_nested(&[b"", b""]), b"\x00\x00\x00\x00");

        assert_eq!(key_prefix_nested(&[b"a"]), b"\x00\x01a");
        assert_eq!(key_prefix_nested(&[b"a", b"ab"]), b"\x00\x01a\x00\x02ab");
        assert_eq!(
            key_prefix_nested(&[b"a", b"ab", b"abc"]),
            b"\x00\x01a\x00\x02ab\x00\x03abc"
        );
    }

    #[test]
    fn prefix_get_set() {
        let mut storage = MockStorage::new();
        let prefix = key_prefix(b"foo");

        set_with_prefix(&mut storage, &prefix, b"bar", b"gotcha").unwrap();
        let rfoo = get_with_prefix(&storage, &prefix, b"bar").unwrap();
        assert_eq!(Some(b"gotcha".to_vec()), rfoo);

        // no collisions with other prefixes
        let other_prefix = key_prefix(b"fo");
        let collision = get_with_prefix(&storage, &other_prefix, b"obar").unwrap();
        assert_eq!(None, collision);
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn test_range() {
        let mut storage = MockStorage::new();
        let prefix = key_prefix(b"foo");
        let other_prefix = key_prefix(b"food");

        // set some values in this range
        set_with_prefix(&mut storage, &prefix, b"bar", b"none").unwrap();
        set_with_prefix(&mut storage, &prefix, b"snowy", b"day").unwrap();

        // set some values outside this range
        set_with_prefix(&mut storage, &other_prefix, b"moon", b"buggy").unwrap();

        // ensure we get proper result from prefixed_range iterator
        let mut iter = range_with_prefix(&storage, &prefix, None, None, Order::Descending);
        let first = iter.next().unwrap();
        assert_eq!(first, (b"snowy".to_vec(), b"day".to_vec()));
        let second = iter.next().unwrap();
        assert_eq!(second, (b"bar".to_vec(), b"none".to_vec()));
        assert_eq!(iter.next(), None);

        // ensure we get raw result from base range
        let iter = storage.range(None, None, Order::Ascending);
        assert_eq!(3, iter.count());

        // foo comes first
        let mut iter = storage.range(None, None, Order::Ascending);
        let first = iter.next().unwrap();
        let expected_key = concat(&prefix, b"bar");
        assert_eq!(first, (expected_key, b"none".to_vec()));
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn test_range_with_prefix_wrapover() {
        let mut storage = MockStorage::new();
        // if we don't properly wrap over there will be issues here (note 255+1 is used to calculate end)
        let prefix = key_prefix(b"f\xff\xff");
        let other_prefix = key_prefix(b"f\xff\x44");

        // set some values in this range
        set_with_prefix(&mut storage, &prefix, b"bar", b"none").unwrap();
        set_with_prefix(&mut storage, &prefix, b"snowy", b"day").unwrap();

        // set some values outside this range
        set_with_prefix(&mut storage, &other_prefix, b"moon", b"buggy").unwrap();

        // ensure we get proper result from prefixed_range iterator
        let mut iter = range_with_prefix(&storage, &prefix, None, None, Order::Descending);
        let first = iter.next().unwrap();
        assert_eq!(first, (b"snowy".to_vec(), b"day".to_vec()));
        let second = iter.next().unwrap();
        assert_eq!(second, (b"bar".to_vec(), b"none".to_vec()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn test_range_with_start_end_set() {
        let mut storage = MockStorage::new();
        // if we don't properly wrap over there will be issues here (note 255+1 is used to calculate end)
        let prefix = key_prefix(b"f\xff\xff");
        let other_prefix = key_prefix(b"f\xff\x44");

        // set some values in this range
        set_with_prefix(&mut storage, &prefix, b"bar", b"none").unwrap();
        set_with_prefix(&mut storage, &prefix, b"snowy", b"day").unwrap();

        // set some values outside this range
        set_with_prefix(&mut storage, &other_prefix, b"moon", b"buggy").unwrap();

        // make sure start and end are applied properly
        let res: Vec<KV> =
            range_with_prefix(&storage, &prefix, Some(b"b"), Some(b"c"), Order::Ascending)
                .collect();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], (b"bar".to_vec(), b"none".to_vec()));

        // make sure start and end are applied properly
        let res: Vec<KV> = range_with_prefix(
            &storage,
            &prefix,
            Some(b"bas"),
            Some(b"sno"),
            Order::Ascending,
        )
        .collect();
        assert_eq!(res.len(), 0);

        let res: Vec<KV> =
            range_with_prefix(&storage, &prefix, Some(b"ant"), None, Order::Ascending).collect();
        assert_eq!(res.len(), 2);
        assert_eq!(res[0], (b"bar".to_vec(), b"none".to_vec()));
        assert_eq!(res[1], (b"snowy".to_vec(), b"day".to_vec()));
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn test_namespace_upper_bound() {
        assert_eq!(namespace_upper_bound(b"bob"), b"boc".to_vec());
        assert_eq!(namespace_upper_bound(b"fo\xfe"), b"fo\xff".to_vec());
        assert_eq!(namespace_upper_bound(b"fo\xff"), b"fp\x00".to_vec());
        // multiple \xff roll over
        assert_eq!(
            namespace_upper_bound(b"fo\xff\xff\xff"),
            b"fp\x00\x00\x00".to_vec()
        );
        // \xff not at the end are ignored
        assert_eq!(namespace_upper_bound(b"\xffabc"), b"\xffabd".to_vec());
    }
}
