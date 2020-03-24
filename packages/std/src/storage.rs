use std::collections::BTreeMap;
#[cfg(feature = "iterator")]
use std::ops::Bound;

#[cfg(feature = "iterator")]
use crate::traits::{KVPair, Sort};
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
    fn range(
        &self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Sort,
    ) -> Box<dyn Iterator<Item = KVPair>> {
        let bounds = (
            start.map_or(Bound::Unbounded, |x| Bound::Included(x.to_vec())),
            end.map_or(Bound::Unbounded, |x| Bound::Excluded(x.to_vec())),
        );
        let iter = self.data.range(bounds);

        // We brute force this a bit to deal with lifetimes.... should do this lazy
        let res: Vec<_> = match order {
            Sort::Ascending => iter.map(|(k, v)| (k.clone(), v.clone())).collect(),
            Sort::Descending => iter.rev().map(|(k, v)| (k.clone(), v.clone())).collect(),
        };
        Box::new(res.into_iter())
    }
}

impl Storage for MemoryStorage {
    fn set(&mut self, key: &[u8], value: &[u8]) {
        self.data.insert(key.to_vec(), value.to_vec());
    }
}

#[cfg(test)]
#[cfg(feature = "iterator")]
// iterator_test_suite takes a storage, adds data and runs iterator tests
// the storage must previously have exactly one key: "foo" = "bar"
// (this allows us to test StorageTransaction and other wrapped storage better)
//
// designed to be imported by other modules
pub(crate) fn iterator_test_suite<S: Storage>(store: &mut S) {
    // ensure we had previously set "foo" = "bar"
    assert_eq!(store.get(b"foo"), Some(b"bar".to_vec()));
    assert_eq!(store.range(None, None, Sort::Ascending).count(), 1);

    // setup
    store.set(b"ant", b"hill");
    store.set(b"ze", b"bra");

    // open ended range
    {
        let iter = store.range(None, None, Sort::Ascending);
        assert_eq!(3, iter.count());
        let mut iter = store.range(None, None, Sort::Ascending);
        let first = iter.next().unwrap();
        assert_eq!((b"ant".to_vec(), b"hill".to_vec()), first);
        let mut iter = store.range(None, None, Sort::Descending);
        let last = iter.next().unwrap();
        assert_eq!((b"ze".to_vec(), b"bra".to_vec()), last);
    }

    // closed range
    {
        let iter = store.range(Some(b"f"), Some(b"n"), Sort::Ascending);
        assert_eq!(1, iter.count());
        let mut iter = store.range(Some(b"f"), Some(b"n"), Sort::Ascending);
        let first = iter.next().unwrap();
        assert_eq!((b"foo".to_vec(), b"bar".to_vec()), first);
    }

    // closed range reverse
    {
        let iter = store.range(Some(b"air"), Some(b"loop"), Sort::Descending);
        assert_eq!(2, iter.count());
        let mut iter = store.range(Some(b"air"), Some(b"loop"), Sort::Descending);
        let first = iter.next().unwrap();
        assert_eq!((b"foo".to_vec(), b"bar".to_vec()), first);
        let second = iter.next().unwrap();
        assert_eq!((b"ant".to_vec(), b"hill".to_vec()), second);
    }

    // end open iterator
    {
        let iter = store.range(Some(b"f"), None, Sort::Ascending);
        assert_eq!(2, iter.count());
        let mut iter = store.range(Some(b"f"), None, Sort::Ascending);
        let first = iter.next().unwrap();
        assert_eq!((b"foo".to_vec(), b"bar".to_vec()), first);
    }

    // end open iterator reverse
    {
        let iter = store.range(Some(b"f"), None, Sort::Descending);
        assert_eq!(2, iter.count());
        let mut iter = store.range(Some(b"f"), None, Sort::Descending);
        let first = iter.next().unwrap();
        assert_eq!((b"ze".to_vec(), b"bra".to_vec()), first);
    }

    // start open iterator
    {
        let iter = store.range(None, Some(b"f"), Sort::Ascending);
        assert_eq!(1, iter.count());
        let mut iter = store.range(None, Some(b"f"), Sort::Ascending);
        let first = iter.next().unwrap();
        assert_eq!((b"ant".to_vec(), b"hill".to_vec()), first);
    }

    // start open iterator
    {
        let iter = store.range(None, Some(b"no"), Sort::Descending);
        assert_eq!(2, iter.count());
        let mut iter = store.range(None, Some(b"no"), Sort::Descending);
        let first = iter.next().unwrap();
        assert_eq!((b"foo".to_vec(), b"bar".to_vec()), first);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn memory_storage_get_and_set() {
        let mut store = MemoryStorage::new();
        assert_eq!(None, store.get(b"foo"));
        store.set(b"foo", b"bar");
        assert_eq!(Some(b"bar".to_vec()), store.get(b"foo"));
        assert_eq!(None, store.get(b"food"));
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn memory_storage_iterator() {
        let mut store = MemoryStorage::new();
        store.set(b"foo", b"bar");
        iterator_test_suite(&mut store);
    }
}
