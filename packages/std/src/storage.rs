use std::collections::BTreeMap;
#[cfg(feature = "iterator")]
use std::iter;
#[cfg(feature = "iterator")]
use std::ops::{Bound, RangeBounds};

#[cfg(feature = "iterator")]
use crate::iterator::{Order, KV};
use crate::traits::{ReadonlyStorage, Storage};

#[derive(Default)]
pub struct MemoryStorage {
    data: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        MemoryStorage::default()
    }
}

impl ReadonlyStorage for MemoryStorage {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.data.get(key).cloned()
    }

    #[cfg(feature = "iterator")]
    /// range allows iteration over a set of keys, either forwards or backwards
    /// uses standard rust range notation, and eg db.range(b"foo"..b"bar") also works reverse
    fn range<'a>(
        &'a self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> StdResult<Box<dyn Iterator<Item = KV> + 'a>> {
        let bounds = range_bounds(start, end);

        // BTreeMap.range panics if range is start > end.
        // However, this cases represent just empty range and we treat it as such.
        match (bounds.start_bound(), bounds.end_bound()) {
            (Bound::Included(start), Bound::Excluded(end)) if start > end => {
                return Ok(Box::new(iter::empty()));
            }
            _ => {}
        }

        let iter = self.data.range(bounds);
        Ok(match order {
            Order::Ascending => Box::new(iter.map(clone_item)),
            Order::Descending => Box::new(iter.rev().map(clone_item)),
        })
    }
}

#[cfg(feature = "iterator")]
fn range_bounds(start: Option<&[u8]>, end: Option<&[u8]>) -> impl RangeBounds<Vec<u8>> {
    (
        start.map_or(Bound::Unbounded, |x| Bound::Included(x.to_vec())),
        end.map_or(Bound::Unbounded, |x| Bound::Excluded(x.to_vec())),
    )
}

#[cfg(feature = "iterator")]
/// The BTreeMap specific key-value pair reference type, as returned by BTreeMap<Vec<u8>, T>::range.
/// This is internal as it can change any time if the map implementation is swapped out.
type BTreeMapPairRef<'a, T = Vec<u8>> = (&'a Vec<u8>, &'a T);

#[cfg(feature = "iterator")]
fn clone_item<T: Clone>(item_ref: BTreeMapPairRef<T>) -> KV<T> {
    let (key, value) = item_ref;
    (key.clone(), value.clone())
}

impl Storage for MemoryStorage {
    fn set(&mut self, key: &[u8], value: &[u8]) {
        self.data.insert(key.to_vec(), value.to_vec());
    }

    fn remove(&mut self, key: &[u8]) {
        self.data.remove(key);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(feature = "iterator")]
    // iterator_test_suite takes a storage, adds data and runs iterator tests
    // the storage must previously have exactly one key: "foo" = "bar"
    // (this allows us to test StorageTransaction and other wrapped storage better)
    fn iterator_test_suite<S: Storage>(store: &mut S) {
        // ensure we had previously set "foo" = "bar"
        assert_eq!(store.get(b"foo"), Some(b"bar".to_vec()));
        assert_eq!(
            store.range(None, None, Order::Ascending).unwrap().count(),
            1
        );

        // setup - add some data, and delete part of it as well
        store.set(b"ant", b"hill").expect("error setting value");
        store.set(b"ze", b"bra").expect("error setting value");

        // noise that should be ignored
        store.set(b"bye", b"bye").expect("error setting value");
        store.remove(b"bye").expect("error removing key");

        // unbounded
        {
            let iter = store.range(None, None, Order::Ascending).unwrap();
            let elements: Vec<KV> = iter.collect();
            assert_eq!(
                elements,
                vec![
                    (b"ant".to_vec(), b"hill".to_vec()),
                    (b"foo".to_vec(), b"bar".to_vec()),
                    (b"ze".to_vec(), b"bra".to_vec()),
                ]
            );
        }

        // unbounded (descending)
        {
            let iter = store.range(None, None, Order::Descending).unwrap();
            let elements: Vec<KV> = iter.collect();
            assert_eq!(
                elements,
                vec![
                    (b"ze".to_vec(), b"bra".to_vec()),
                    (b"foo".to_vec(), b"bar".to_vec()),
                    (b"ant".to_vec(), b"hill".to_vec()),
                ]
            );
        }

        // bounded
        {
            let iter = store
                .range(Some(b"f"), Some(b"n"), Order::Ascending)
                .unwrap();
            let elements: Vec<KV> = iter.collect();
            assert_eq!(elements, vec![(b"foo".to_vec(), b"bar".to_vec())]);
        }

        // bounded (descending)
        {
            let iter = store
                .range(Some(b"air"), Some(b"loop"), Order::Descending)
                .unwrap();
            let elements: Vec<KV> = iter.collect();
            assert_eq!(
                elements,
                vec![
                    (b"foo".to_vec(), b"bar".to_vec()),
                    (b"ant".to_vec(), b"hill".to_vec()),
                ]
            );
        }

        // bounded empty [a, a)
        {
            let iter = store
                .range(Some(b"foo"), Some(b"foo"), Order::Ascending)
                .unwrap();
            let elements: Vec<KV> = iter.collect();
            assert_eq!(elements, vec![]);
        }

        // bounded empty [a, a) (descending)
        {
            let iter = store
                .range(Some(b"foo"), Some(b"foo"), Order::Descending)
                .unwrap();
            let elements: Vec<KV> = iter.collect();
            assert_eq!(elements, vec![]);
        }

        // bounded empty [a, b) with b < a
        {
            let iter = store
                .range(Some(b"z"), Some(b"a"), Order::Ascending)
                .unwrap();
            let elements: Vec<KV> = iter.collect();
            assert_eq!(elements, vec![]);
        }

        // bounded empty [a, b) with b < a (descending)
        {
            let iter = store
                .range(Some(b"z"), Some(b"a"), Order::Descending)
                .unwrap();
            let elements: Vec<KV> = iter.collect();
            assert_eq!(elements, vec![]);
        }

        // right unbounded
        {
            let iter = store.range(Some(b"f"), None, Order::Ascending).unwrap();
            let elements: Vec<KV> = iter.collect();
            assert_eq!(
                elements,
                vec![
                    (b"foo".to_vec(), b"bar".to_vec()),
                    (b"ze".to_vec(), b"bra".to_vec()),
                ]
            );
        }

        // right unbounded (descending)
        {
            let iter = store.range(Some(b"f"), None, Order::Descending).unwrap();
            let elements: Vec<KV> = iter.collect();
            assert_eq!(
                elements,
                vec![
                    (b"ze".to_vec(), b"bra".to_vec()),
                    (b"foo".to_vec(), b"bar".to_vec()),
                ]
            );
        }

        // left unbounded
        {
            let iter = store.range(None, Some(b"f"), Order::Ascending).unwrap();
            let elements: Vec<KV> = iter.collect();
            assert_eq!(elements, vec![(b"ant".to_vec(), b"hill".to_vec()),]);
        }

        // left unbounded (descending)
        {
            let iter = store.range(None, Some(b"no"), Order::Descending).unwrap();
            let elements: Vec<KV> = iter.collect();
            assert_eq!(
                elements,
                vec![
                    (b"foo".to_vec(), b"bar".to_vec()),
                    (b"ant".to_vec(), b"hill".to_vec()),
                ]
            );
        }
    }

    #[test]
    fn get_and_set() {
        let mut store = MemoryStorage::new();
        assert_eq!(store.get(b"foo"), None);
        store.set(b"foo", b"bar");
        assert_eq!(store.get(b"foo"), Some(b"bar".to_vec()));
        assert_eq!(store.get(b"food"), None);
    }

    #[test]
    fn delete() {
        let mut store = MemoryStorage::new();
        store.set(b"foo", b"bar");
        store.set(b"food", b"bank");
        store.remove(b"foo");

        assert_eq!(store.get(b"foo"), None);
        assert_eq!(store.get(b"food"), Some(b"bank".to_vec()));
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn iterator() {
        let mut store = MemoryStorage::new();
        store.set(b"foo", b"bar").expect("error setting value");
        iterator_test_suite(&mut store);
    }
}
