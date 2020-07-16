use std::collections::BTreeMap;
#[cfg(feature = "iterator")]
use std::ops::{Bound, RangeBounds};

#[cfg(feature = "iterator")]
use cosmwasm_std::{Order, KV};

#[cfg(feature = "iterator")]
use crate::traits::StorageIterator;
use crate::{FfiResult, GasInfo, Storage};

#[cfg(feature = "iterator")]
const GAS_COST_LAST_ITERATION: u64 = 37;

#[cfg(feature = "iterator")]
const GAS_COST_RANGE: u64 = 11;

/// A storage iterator for testing only. This type uses a Rust iterator
/// as a data source, which does not provide a gas value for the last iteration.
#[cfg(feature = "iterator")]
pub struct MockIterator<'a> {
    source: Box<dyn Iterator<Item = (KV, u64)> + 'a>,
}

#[cfg(feature = "iterator")]
impl MockIterator<'_> {
    pub fn empty() -> Self {
        MockIterator {
            source: Box::new(std::iter::empty()),
        }
    }
}

#[cfg(feature = "iterator")]
impl StorageIterator for MockIterator<'_> {
    fn next(&mut self) -> FfiResult<Option<KV>> {
        let item = match self.source.next() {
            Some((kv, gas_used)) => (Some(kv), GasInfo::with_externally_used(gas_used)),
            None => (None, GasInfo::with_externally_used(GAS_COST_LAST_ITERATION)),
        };
        Ok(item)
    }
}

#[derive(Default, Debug)]
pub struct MockStorage {
    data: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl MockStorage {
    pub fn new() -> Self {
        MockStorage::default()
    }
}

impl Storage for MockStorage {
    fn get(&self, key: &[u8]) -> FfiResult<Option<Vec<u8>>> {
        let gas_info = GasInfo::with_externally_used(key.len() as u64);
        Ok((self.data.get(key).cloned(), gas_info))
    }

    #[cfg(feature = "iterator")]
    /// range allows iteration over a set of keys, either forwards or backwards
    /// uses standard rust range notation, and eg db.range(b"foo"..b"bar") also works reverse
    fn range<'a>(
        &'a self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> FfiResult<Box<dyn StorageIterator + 'a>> {
        let gas_info = GasInfo::with_externally_used(GAS_COST_RANGE);
        let bounds = range_bounds(start, end);

        // BTreeMap.range panics if range is start > end.
        // However, this cases represent just empty range and we treat it as such.
        match (bounds.start_bound(), bounds.end_bound()) {
            (Bound::Included(start), Bound::Excluded(end)) if start > end => {
                return Ok((Box::new(MockIterator::empty()), gas_info));
            }
            _ => {}
        }

        let original_iter = self.data.range(bounds);
        let iter: Box<dyn Iterator<Item = (KV, u64)>> = match order {
            Order::Ascending => Box::new(original_iter.map(clone_item).map(|item| {
                let gas_cost = (item.0.len() + item.1.len()) as u64;
                (item, gas_cost)
            })),
            Order::Descending => Box::new(original_iter.rev().map(clone_item).map(|item| {
                let gas_cost = (item.0.len() + item.1.len()) as u64;
                (item, gas_cost)
            })),
        };
        Ok((Box::new(MockIterator { source: iter }), gas_info))
    }

    fn set(&mut self, key: &[u8], value: &[u8]) -> FfiResult<()> {
        self.data.insert(key.to_vec(), value.to_vec());
        let gas_info = GasInfo::with_externally_used((key.len() + value.len()) as u64);
        Ok(((), gas_info))
    }

    fn remove(&mut self, key: &[u8]) -> FfiResult<()> {
        self.data.remove(key);
        let gas_info = GasInfo::with_externally_used(key.len() as u64);
        Ok(((), gas_info))
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_and_set() {
        let mut store = MockStorage::new();
        assert_eq!(None, store.get(b"foo").unwrap().0);
        store.set(b"foo", b"bar").unwrap();
        assert_eq!(Some(b"bar".to_vec()), store.get(b"foo").unwrap().0);
        assert_eq!(None, store.get(b"food").unwrap().0);
    }

    #[test]
    fn delete() {
        let mut store = MockStorage::new();
        store.set(b"foo", b"bar").unwrap();
        store.set(b"food", b"bank").unwrap();
        store.remove(b"foo").unwrap();

        assert_eq!(None, store.get(b"foo").unwrap().0);
        assert_eq!(Some(b"bank".to_vec()), store.get(b"food").unwrap().0);
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn iterator() {
        let mut store = MockStorage::new();
        store.set(b"foo", b"bar").expect("error setting value");

        // ensure we had previously set "foo" = "bar"
        assert_eq!(store.get(b"foo").unwrap().0, Some(b"bar".to_vec()));
        assert_eq!(
            store
                .range(None, None, Order::Ascending)
                .unwrap()
                .0
                .elements()
                .unwrap()
                .len(),
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
            let iter = store.range(None, None, Order::Ascending).unwrap().0;
            let elements = iter.elements().unwrap();
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
            let iter = store.range(None, None, Order::Descending).unwrap().0;
            let elements = iter.elements().unwrap();
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
                .unwrap()
                .0;
            let elements = iter.elements().unwrap();
            assert_eq!(elements, vec![(b"foo".to_vec(), b"bar".to_vec())]);
        }

        // bounded (descending)
        {
            let iter = store
                .range(Some(b"air"), Some(b"loop"), Order::Descending)
                .unwrap()
                .0;
            let elements = iter.elements().unwrap();
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
                .unwrap()
                .0;
            let elements = iter.elements().unwrap();
            assert_eq!(elements, vec![]);
        }

        // bounded empty [a, a) (descending)
        {
            let iter = store
                .range(Some(b"foo"), Some(b"foo"), Order::Descending)
                .unwrap()
                .0;
            let elements = iter.elements().unwrap();
            assert_eq!(elements, vec![]);
        }

        // bounded empty [a, b) with b < a
        {
            let iter = store
                .range(Some(b"z"), Some(b"a"), Order::Ascending)
                .unwrap()
                .0;
            let elements = iter.elements().unwrap();
            assert_eq!(elements, vec![]);
        }

        // bounded empty [a, b) with b < a (descending)
        {
            let iter = store
                .range(Some(b"z"), Some(b"a"), Order::Descending)
                .unwrap()
                .0;
            let elements = iter.elements().unwrap();
            assert_eq!(elements, vec![]);
        }

        // right unbounded
        {
            let iter = store.range(Some(b"f"), None, Order::Ascending).unwrap().0;
            let elements = iter.elements().unwrap();
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
            let iter = store.range(Some(b"f"), None, Order::Descending).unwrap().0;
            let elements = iter.elements().unwrap();
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
            let iter = store.range(None, Some(b"f"), Order::Ascending).unwrap().0;
            let elements = iter.elements().unwrap();
            assert_eq!(elements, vec![(b"ant".to_vec(), b"hill".to_vec()),]);
        }

        // left unbounded (descending)
        {
            let iter = store.range(None, Some(b"no"), Order::Descending).unwrap().0;
            let elements = iter.elements().unwrap();
            assert_eq!(
                elements,
                vec![
                    (b"foo".to_vec(), b"bar".to_vec()),
                    (b"ant".to_vec(), b"hill".to_vec()),
                ]
            );
        }
    }
}
