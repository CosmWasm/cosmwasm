use std::collections::BTreeMap;
#[cfg(feature = "iterator")]
use std::ops::{Bound, RangeBounds};

use crate::errors::Result;
#[cfg(feature = "iterator")]
use crate::traits::{KVRef, Order, KV};
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
    ) -> Box<dyn Iterator<Item = KV> + 'a> {
        let bounds = range_bounds(start, end);
        let iter = self.data.range(bounds);
        match order {
            Order::Ascending => Box::new(IterVec { iter }),
            Order::Descending => Box::new(IterVec { iter: iter.rev() }),
        }
    }
}

#[cfg(feature = "iterator")]
pub(crate) fn range_bounds(start: Option<&[u8]>, end: Option<&[u8]>) -> impl RangeBounds<Vec<u8>> {
    (
        start.map_or(Bound::Unbounded, |x| Bound::Included(x.to_vec())),
        end.map_or(Bound::Unbounded, |x| Bound::Excluded(x.to_vec())),
    )
}

#[cfg(feature = "iterator")]
struct IterVec<'a, T: Iterator<Item = KVRef<'a>>> {
    iter: T,
}

#[cfg(feature = "iterator")]
impl<'a, T: Iterator<Item = KVRef<'a>>> Iterator for IterVec<'a, T> {
    type Item = KV;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.iter.next();
        match n {
            Some((k, v)) => Some((k.clone(), v.clone())),
            None => None,
        }
    }
}

impl Storage for MemoryStorage {
    fn set(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self.data.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn remove(&mut self, key: &[u8]) -> Result<()> {
        self.data.remove(key);
        Ok(())
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
    assert_eq!(store.range(None, None, Order::Ascending).count(), 1);

    // setup - add some data, and delete part of it as well
    store.set(b"ant", b"hill").expect("error setting value");
    store.set(b"ze", b"bra").expect("error setting value");

    // noise that should be ignored
    store.set(b"bye", b"bye").expect("error setting value");
    store.remove(b"bye").expect("error removing key");

    // open ended range
    {
        let iter = store.range(None, None, Order::Ascending);
        assert_eq!(3, iter.count());
        let mut iter = store.range(None, None, Order::Ascending);
        let first = iter.next().unwrap();
        assert_eq!((b"ant".to_vec(), b"hill".to_vec()), first);
        let mut iter = store.range(None, None, Order::Descending);
        let last = iter.next().unwrap();
        assert_eq!((b"ze".to_vec(), b"bra".to_vec()), last);
    }

    // closed range
    {
        let iter = store.range(Some(b"f"), Some(b"n"), Order::Ascending);
        assert_eq!(1, iter.count());
        let mut iter = store.range(Some(b"f"), Some(b"n"), Order::Ascending);
        let first = iter.next().unwrap();
        assert_eq!((b"foo".to_vec(), b"bar".to_vec()), first);
    }

    // closed range reverse
    {
        let iter = store.range(Some(b"air"), Some(b"loop"), Order::Descending);
        assert_eq!(2, iter.count());
        let mut iter = store.range(Some(b"air"), Some(b"loop"), Order::Descending);
        let first = iter.next().unwrap();
        assert_eq!((b"foo".to_vec(), b"bar".to_vec()), first);
        let second = iter.next().unwrap();
        assert_eq!((b"ant".to_vec(), b"hill".to_vec()), second);
    }

    // end open iterator
    {
        let iter = store.range(Some(b"f"), None, Order::Ascending);
        assert_eq!(2, iter.count());
        let mut iter = store.range(Some(b"f"), None, Order::Ascending);
        let first = iter.next().unwrap();
        assert_eq!((b"foo".to_vec(), b"bar".to_vec()), first);
    }

    // end open iterator reverse
    {
        let iter = store.range(Some(b"f"), None, Order::Descending);
        assert_eq!(2, iter.count());
        let mut iter = store.range(Some(b"f"), None, Order::Descending);
        let first = iter.next().unwrap();
        assert_eq!((b"ze".to_vec(), b"bra".to_vec()), first);
    }

    // start open iterator
    {
        let iter = store.range(None, Some(b"f"), Order::Ascending);
        assert_eq!(1, iter.count());
        let mut iter = store.range(None, Some(b"f"), Order::Ascending);
        let first = iter.next().unwrap();
        assert_eq!((b"ant".to_vec(), b"hill".to_vec()), first);
    }

    // start open iterator
    {
        let iter = store.range(None, Some(b"no"), Order::Descending);
        assert_eq!(2, iter.count());
        let mut iter = store.range(None, Some(b"no"), Order::Descending);
        let first = iter.next().unwrap();
        assert_eq!((b"foo".to_vec(), b"bar".to_vec()), first);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_and_set() {
        let mut store = MemoryStorage::new();
        assert_eq!(None, store.get(b"foo"));
        store.set(b"foo", b"bar").unwrap();
        assert_eq!(Some(b"bar".to_vec()), store.get(b"foo"));
        assert_eq!(None, store.get(b"food"));
    }

    #[test]
    fn delete() {
        let mut store = MemoryStorage::new();
        store.set(b"foo", b"bar").unwrap();
        store.set(b"food", b"bank").unwrap();
        store.remove(b"foo").unwrap();

        assert_eq!(None, store.get(b"foo"));
        assert_eq!(Some(b"bank".to_vec()), store.get(b"food"));
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn iterator() {
        let mut store = MemoryStorage::new();
        store.set(b"foo", b"bar").expect("error setting value");
        iterator_test_suite(&mut store);
    }
}
