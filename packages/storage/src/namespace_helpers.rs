use crate::cosmwasm_std::Storage;
#[cfg(feature = "iterator")]
use crate::cosmwasm_std::{Order, Record};

pub(crate) fn get_with_prefix(
    storage: &dyn Storage,
    namespace: &[u8],
    key: &[u8],
) -> Option<Vec<u8>> {
    storage.get(&concat(namespace, key))
}

pub(crate) fn set_with_prefix(
    storage: &mut dyn Storage,
    namespace: &[u8],
    key: &[u8],
    value: &[u8],
) {
    storage.set(&concat(namespace, key), value);
}

pub(crate) fn remove_with_prefix(storage: &mut dyn Storage, namespace: &[u8], key: &[u8]) {
    storage.remove(&concat(namespace, key));
}

#[inline]
fn concat(namespace: &[u8], key: &[u8]) -> Vec<u8> {
    let mut k = namespace.to_vec();
    k.extend_from_slice(key);
    k
}

#[cfg(feature = "iterator")]
pub(crate) fn range_with_prefix<'a>(
    storage: &'a dyn Storage,
    namespace: &[u8],
    start: Option<&[u8]>,
    end: Option<&[u8]>,
    order: Order,
) -> Box<dyn Iterator<Item = Record> + 'a> {
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
            copy[i] += 1;
            break;
        }
    }
    copy
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::length_prefixed::to_length_prefixed;
    use crate::cosmwasm_std::testing::MockStorage;

    #[test]
    fn prefix_get_set() {
        let mut storage = MockStorage::new();
        let prefix = to_length_prefixed(b"foo");

        set_with_prefix(&mut storage, &prefix, b"bar", b"gotcha");
        let rfoo = get_with_prefix(&storage, &prefix, b"bar");
        assert_eq!(rfoo, Some(b"gotcha".to_vec()));

        // no collisions with other prefixes
        let other_prefix = to_length_prefixed(b"fo");
        let collision = get_with_prefix(&storage, &other_prefix, b"obar");
        assert_eq!(collision, None);
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn range_works() {
        let mut storage = MockStorage::new();
        let prefix = to_length_prefixed(b"foo");
        let other_prefix = to_length_prefixed(b"food");

        // set some values in this range
        set_with_prefix(&mut storage, &prefix, b"bar", b"none");
        set_with_prefix(&mut storage, &prefix, b"snowy", b"day");

        // set some values outside this range
        set_with_prefix(&mut storage, &other_prefix, b"moon", b"buggy");

        // ensure we get proper result from prefixed_range iterator
        let mut iter = range_with_prefix(&storage, &prefix, None, None, Order::Descending);
        let first = iter.next().unwrap();
        assert_eq!(first, (b"snowy".to_vec(), b"day".to_vec()));
        let second = iter.next().unwrap();
        assert_eq!(second, (b"bar".to_vec(), b"none".to_vec()));
        assert!(iter.next().is_none());

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
    fn range_with_prefix_wrapover() {
        let mut storage = MockStorage::new();
        // if we don't properly wrap over there will be issues here (note 255+1 is used to calculate end)
        let prefix = to_length_prefixed(b"f\xff\xff");
        let other_prefix = to_length_prefixed(b"f\xff\x44");

        // set some values in this range
        set_with_prefix(&mut storage, &prefix, b"bar", b"none");
        set_with_prefix(&mut storage, &prefix, b"snowy", b"day");

        // set some values outside this range
        set_with_prefix(&mut storage, &other_prefix, b"moon", b"buggy");

        // ensure we get proper result from prefixed_range iterator
        let iter = range_with_prefix(&storage, &prefix, None, None, Order::Descending);
        let elements: Vec<Record> = iter.collect();
        assert_eq!(
            elements,
            vec![
                (b"snowy".to_vec(), b"day".to_vec()),
                (b"bar".to_vec(), b"none".to_vec()),
            ]
        );
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn range_with_start_end_set() {
        let mut storage = MockStorage::new();
        // if we don't properly wrap over there will be issues here (note 255+1 is used to calculate end)
        let prefix = to_length_prefixed(b"f\xff\xff");
        let other_prefix = to_length_prefixed(b"f\xff\x44");

        // set some values in this range
        set_with_prefix(&mut storage, &prefix, b"bar", b"none");
        set_with_prefix(&mut storage, &prefix, b"snowy", b"day");

        // set some values outside this range
        set_with_prefix(&mut storage, &other_prefix, b"moon", b"buggy");

        // make sure start and end are applied properly
        let res: Vec<Record> =
            range_with_prefix(&storage, &prefix, Some(b"b"), Some(b"c"), Order::Ascending)
                .collect();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], (b"bar".to_vec(), b"none".to_vec()));

        // make sure start and end are applied properly
        let res_count = range_with_prefix(
            &storage,
            &prefix,
            Some(b"bas"),
            Some(b"sno"),
            Order::Ascending,
        )
        .count();
        assert_eq!(res_count, 0);

        let res: Vec<Record> =
            range_with_prefix(&storage, &prefix, Some(b"ant"), None, Order::Ascending).collect();
        assert_eq!(res.len(), 2);
        assert_eq!(res[0], (b"bar".to_vec(), b"none".to_vec()));
        assert_eq!(res[1], (b"snowy".to_vec(), b"day".to_vec()));
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn namespace_upper_bound_works() {
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
