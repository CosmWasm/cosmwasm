#[cfg(feature = "iterator")]
use std::cmp::Ordering;
use std::collections::BTreeMap;
#[cfg(feature = "iterator")]
use std::iter;
#[cfg(feature = "iterator")]
use std::iter::Peekable;
#[cfg(feature = "iterator")]
use std::ops::{Bound, RangeBounds};

use cosmwasm_std::{Api, Extern, Querier, ReadonlyStorage, StdResult, Storage};
#[cfg(feature = "iterator")]
use cosmwasm_std::{Order, KV};

#[cfg(feature = "iterator")]
/// The BTreeMap specific key-value pair reference type, as returned by BTreeMap<Vec<u8>, T>::range.
/// This is internal as it can change any time if the map implementation is swapped out.
type BTreeMapPairRef<'a, T = Vec<u8>> = (&'a Vec<u8>, &'a T);

pub struct StorageTransaction<'a, S: ReadonlyStorage> {
    /// read-only access to backing storage
    storage: &'a S,
    /// these are local changes not flushed to backing storage
    local_state: BTreeMap<Vec<u8>, Delta>,
    /// a log of local changes not yet flushed to backing storage
    rep_log: RepLog,
}

impl<'a, S: ReadonlyStorage> StorageTransaction<'a, S> {
    pub fn new(storage: &'a S) -> Self {
        StorageTransaction {
            storage,
            local_state: BTreeMap::new(),
            rep_log: RepLog::new(),
        }
    }

    /// prepares this transaction to be committed to storage
    pub fn prepare(self) -> RepLog {
        self.rep_log
    }

    /// rollback will consume the checkpoint and drop all changes (no really needed, going out of scope does the same, but nice for clarity)
    pub fn rollback(self) {}
}

impl<'a, S: ReadonlyStorage> ReadonlyStorage for StorageTransaction<'a, S> {
    fn get(&self, key: &[u8]) -> StdResult<Option<Vec<u8>>> {
        match self.local_state.get(key) {
            Some(val) => Ok(match val {
                Delta::Set { value } => Some(value.clone()),
                Delta::Delete {} => None,
            }),
            None => self.storage.get(key),
        }
    }

    #[cfg(feature = "iterator")]
    /// range allows iteration over a set of keys, either forwards or backwards
    /// uses standard rust range notation, and eg db.range(b"foo"..b"bar") also works reverse
    fn range<'b>(
        &'b self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> StdResult<Box<dyn Iterator<Item = StdResult<KV>> + 'b>> {
        let bounds = range_bounds(start, end);

        // BTreeMap.range panics if range is start > end.
        // However, this cases represent just empty range and we treat it as such.
        let local: Box<dyn Iterator<Item = BTreeMapPairRef<Delta>>> =
            match (bounds.start_bound(), bounds.end_bound()) {
                (Bound::Included(start), Bound::Excluded(end)) if start > end => {
                    Box::new(iter::empty())
                }
                _ => {
                    let local_raw = self.local_state.range(bounds);
                    match order {
                        Order::Ascending => Box::new(local_raw),
                        Order::Descending => Box::new(local_raw.rev()),
                    }
                }
            };

        let base = self.storage.range(start, end, order)?;
        let merged = MergeOverlay::new(local, base, order);
        Ok(Box::new(merged))
    }
}

impl<'a, S: ReadonlyStorage> Storage for StorageTransaction<'a, S> {
    fn set(&mut self, key: &[u8], value: &[u8]) -> StdResult<()> {
        let op = Op::Set {
            key: key.to_vec(),
            value: value.to_vec(),
        };
        self.local_state.insert(key.to_vec(), op.to_delta());
        self.rep_log.append(op);
        Ok(())
    }

    fn remove(&mut self, key: &[u8]) -> StdResult<()> {
        let op = Op::Delete { key: key.to_vec() };
        self.local_state.insert(key.to_vec(), op.to_delta());
        self.rep_log.append(op);
        Ok(())
    }
}

pub struct RepLog {
    /// this is a list of changes to be written to backing storage upon commit
    ops_log: Vec<Op>,
}

impl RepLog {
    fn new() -> Self {
        RepLog { ops_log: vec![] }
    }

    /// appends an op to the list of changes to be applied upon commit
    fn append(&mut self, op: Op) {
        self.ops_log.push(op);
    }

    /// applies the stored list of `Op`s to the provided `Storage`
    pub fn commit<S: Storage>(self, storage: &mut S) -> StdResult<()> {
        for op in self.ops_log {
            op.apply(storage)?;
        }
        Ok(())
    }
}

/// Op is the user operation, which can be stored in the RepLog.
/// Currently Set or Delete.
enum Op {
    /// represents the `Set` operation for setting a key-value pair in storage
    Set {
        key: Vec<u8>,
        value: Vec<u8>,
    },
    Delete {
        key: Vec<u8>,
    },
}

impl Op {
    /// applies this `Op` to the provided storage
    pub fn apply<S: Storage>(&self, storage: &mut S) -> StdResult<()> {
        match self {
            Op::Set { key, value } => storage.set(&key, &value),
            Op::Delete { key } => storage.remove(&key),
        }
    }

    /// converts the Op to a delta, which can be stored in a local cache
    pub fn to_delta(&self) -> Delta {
        match self {
            Op::Set { value, .. } => Delta::Set {
                value: value.clone(),
            },
            Op::Delete { .. } => Delta::Delete {},
        }
    }
}

/// Delta is the changes, stored in the local transaction cache.
/// This is either Set{value} or Delete{}. Note that this is the "value"
/// part of a BTree, so the Key (from the Op) is stored separately.
enum Delta {
    Set { value: Vec<u8> },
    Delete {},
}

#[cfg(feature = "iterator")]
struct MergeOverlay<'a, L, R>
where
    L: Iterator<Item = BTreeMapPairRef<'a, Delta>>,
    R: Iterator<Item = StdResult<KV>>,
{
    left: Peekable<L>,
    right: Peekable<R>,
    order: Order,
}

#[cfg(feature = "iterator")]
impl<'a, L, R> MergeOverlay<'a, L, R>
where
    L: Iterator<Item = BTreeMapPairRef<'a, Delta>>,
    R: Iterator<Item = StdResult<KV>>,
{
    fn new(left: L, right: R, order: Order) -> Self {
        MergeOverlay {
            left: left.peekable(),
            right: right.peekable(),
            order,
        }
    }

    fn pick_match(&mut self, lkey: Vec<u8>, rkey: Vec<u8>) -> Option<StdResult<KV>> {
        // compare keys - result is such that Ordering::Less => return left side
        let order = match self.order {
            Order::Ascending => lkey.cmp(&rkey),
            Order::Descending => rkey.cmp(&lkey),
        };

        // left must be translated and filtered before return, not so with right
        match order {
            Ordering::Less => self.take_left(),
            Ordering::Equal => {
                //
                let _ = self.right.next();
                self.take_left()
            }
            Ordering::Greater => self.right.next(),
        }
    }

    /// take_left must only be called when we know self.left.next() will return Some
    fn take_left(&mut self) -> Option<StdResult<KV>> {
        let (lkey, lval) = self.left.next().unwrap();
        match lval {
            Delta::Set { value } => Some(Ok((lkey.clone(), value.clone()))),
            Delta::Delete {} => self.next(),
        }
    }
}

#[cfg(feature = "iterator")]
impl<'a, L, R> Iterator for MergeOverlay<'a, L, R>
where
    L: Iterator<Item = BTreeMapPairRef<'a, Delta>>,
    R: Iterator<Item = StdResult<KV>>,
{
    type Item = StdResult<KV>;

    fn next(&mut self) -> Option<Self::Item> {
        let (left, right) = (self.left.peek(), self.right.peek());
        match (left, right) {
            (Some(litem), Some(ritem)) => {
                let (lkey, _) = litem;
                let (rkey, _) = ritem.as_ref().expect("error items not yet supported");

                // we just use cloned keys to avoid double mutable references
                // (we must release the return value from peek, before beginning to call next or other mut methods
                let (l, r) = (lkey.to_vec(), rkey.to_vec());
                self.pick_match(l, r)
            }
            (Some(_), None) => self.take_left(),
            (None, Some(_)) => self.right.next(),
            (None, None) => None,
        }
    }
}

pub fn transactional<S, C, T>(storage: &mut S, callback: C) -> StdResult<T>
where
    S: Storage,
    C: FnOnce(&mut StorageTransaction<S>) -> StdResult<T>,
{
    let mut stx = StorageTransaction::new(storage);
    let res = callback(&mut stx)?;
    stx.prepare().commit(storage)?;
    Ok(res)
}

pub fn transactional_deps<S, A, Q, C, T>(deps: &mut Extern<S, A, Q>, callback: C) -> StdResult<T>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: FnOnce(&mut Extern<StorageTransaction<S>, A, Q>) -> StdResult<T>,
{
    let c = StorageTransaction::new(&deps.storage);
    let mut stx_deps = Extern {
        storage: c,
        api: deps.api,
        querier: deps.querier.clone(),
    };
    let res = callback(&mut stx_deps);
    if res.is_ok() {
        stx_deps.storage.prepare().commit(&mut deps.storage)?;
    } else {
        stx_deps.storage.rollback();
    }
    res
}

#[cfg(feature = "iterator")]
fn range_bounds(start: Option<&[u8]>, end: Option<&[u8]>) -> impl RangeBounds<Vec<u8>> {
    (
        start.map_or(Bound::Unbounded, |x| Bound::Included(x.to_vec())),
        end.map_or(Bound::Unbounded, |x| Bound::Excluded(x.to_vec())),
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::{unauthorized, MemoryStorage};

    #[cfg(feature = "iterator")]
    // iterator_test_suite takes a storage, adds data and runs iterator tests
    // the storage must previously have exactly one key: "foo" = "bar"
    // (this allows us to test StorageTransaction and other wrapped storage better)
    fn iterator_test_suite<S: Storage>(store: &mut S) {
        // ensure we had previously set "foo" = "bar"
        assert_eq!(store.get(b"foo").unwrap(), Some(b"bar".to_vec()));
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
            let elements: Vec<KV> = iter.filter_map(StdResult::ok).collect();
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
            let elements: Vec<KV> = iter.filter_map(StdResult::ok).collect();
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
            let elements: Vec<KV> = iter.filter_map(StdResult::ok).collect();
            assert_eq!(elements, vec![(b"foo".to_vec(), b"bar".to_vec())]);
        }

        // bounded (descending)
        {
            let iter = store
                .range(Some(b"air"), Some(b"loop"), Order::Descending)
                .unwrap();
            let elements: Vec<KV> = iter.filter_map(StdResult::ok).collect();
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
            let elements: Vec<KV> = iter.filter_map(StdResult::ok).collect();
            assert_eq!(elements, vec![]);
        }

        // bounded empty [a, a) (descending)
        {
            let iter = store
                .range(Some(b"foo"), Some(b"foo"), Order::Descending)
                .unwrap();
            let elements: Vec<KV> = iter.filter_map(StdResult::ok).collect();
            assert_eq!(elements, vec![]);
        }

        // bounded empty [a, b) with b < a
        {
            let iter = store
                .range(Some(b"z"), Some(b"a"), Order::Ascending)
                .unwrap();
            let elements: Vec<KV> = iter.filter_map(StdResult::ok).collect();
            assert_eq!(elements, vec![]);
        }

        // bounded empty [a, b) with b < a (descending)
        {
            let iter = store
                .range(Some(b"z"), Some(b"a"), Order::Descending)
                .unwrap();
            let elements: Vec<KV> = iter.filter_map(StdResult::ok).collect();
            assert_eq!(elements, vec![]);
        }

        // right unbounded
        {
            let iter = store.range(Some(b"f"), None, Order::Ascending).unwrap();
            let elements: Vec<KV> = iter.filter_map(StdResult::ok).collect();
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
            let elements: Vec<KV> = iter.filter_map(StdResult::ok).collect();
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
            let elements: Vec<KV> = iter.filter_map(StdResult::ok).collect();
            assert_eq!(elements, vec![(b"ant".to_vec(), b"hill".to_vec()),]);
        }

        // left unbounded (descending)
        {
            let iter = store.range(None, Some(b"no"), Order::Descending).unwrap();
            let elements: Vec<KV> = iter.filter_map(StdResult::ok).collect();
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
    fn delete_local() {
        let mut base = MemoryStorage::new();
        let mut check = StorageTransaction::new(&base);
        check.set(b"foo", b"bar").unwrap();
        check.set(b"food", b"bank").unwrap();
        check.remove(b"foo").unwrap();

        assert_eq!(None, check.get(b"foo").unwrap());
        assert_eq!(Some(b"bank".to_vec()), check.get(b"food").unwrap());

        // now commit to base and query there
        check.prepare().commit(&mut base).unwrap();
        assert_eq!(None, base.get(b"foo").unwrap());
        assert_eq!(Some(b"bank".to_vec()), base.get(b"food").unwrap());
    }

    #[test]
    fn delete_from_base() {
        let mut base = MemoryStorage::new();
        base.set(b"foo", b"bar").unwrap();
        let mut check = StorageTransaction::new(&base);
        check.set(b"food", b"bank").unwrap();
        check.remove(b"foo").unwrap();

        assert_eq!(None, check.get(b"foo").unwrap());
        assert_eq!(Some(b"bank".to_vec()), check.get(b"food").unwrap());

        // now commit to base and query there
        check.prepare().commit(&mut base).unwrap();
        assert_eq!(None, base.get(b"foo").unwrap());
        assert_eq!(Some(b"bank".to_vec()), base.get(b"food").unwrap());
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn storage_transaction_iterator_empty_base() {
        let base = MemoryStorage::new();
        let mut check = StorageTransaction::new(&base);
        check.set(b"foo", b"bar").expect("error setting value");
        iterator_test_suite(&mut check);
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn storage_transaction_iterator_with_base_data() {
        let mut base = MemoryStorage::new();
        base.set(b"foo", b"bar").expect("error setting value");
        let mut check = StorageTransaction::new(&base);
        iterator_test_suite(&mut check);
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn storage_transaction_iterator_removed_items_from_base() {
        let mut base = MemoryStorage::new();
        base.set(b"foo", b"bar").expect("error setting value");
        base.set(b"food", b"bank").expect("error setting value");
        let mut check = StorageTransaction::new(&base);
        check.remove(b"food").expect("error removing key");
        iterator_test_suite(&mut check);
    }

    #[test]
    fn commit_writes_through() {
        let mut base = MemoryStorage::new();
        base.set(b"foo", b"bar").unwrap();

        let mut check = StorageTransaction::new(&base);
        assert_eq!(check.get(b"foo").unwrap(), Some(b"bar".to_vec()));
        check.set(b"subtx", b"works").unwrap();
        check.prepare().commit(&mut base).unwrap();

        assert_eq!(base.get(b"subtx").unwrap(), Some(b"works".to_vec()));
    }

    #[test]
    fn storage_remains_readable() {
        let mut base = MemoryStorage::new();
        base.set(b"foo", b"bar").unwrap();

        let mut stxn1 = StorageTransaction::new(&base);

        assert_eq!(stxn1.get(b"foo").unwrap(), Some(b"bar".to_vec()));

        stxn1.set(b"subtx", b"works").unwrap();
        assert_eq!(stxn1.get(b"subtx").unwrap(), Some(b"works".to_vec()));

        // Can still read from base, txn is not yet committed
        assert_eq!(base.get(b"subtx").unwrap(), None);

        stxn1.prepare().commit(&mut base).unwrap();
        assert_eq!(base.get(b"subtx").unwrap(), Some(b"works".to_vec()));
    }

    #[test]
    fn rollback_has_no_effect() {
        let mut base = MemoryStorage::new();
        base.set(b"foo", b"bar").unwrap();

        let mut check = StorageTransaction::new(&base);
        assert_eq!(check.get(b"foo").unwrap(), Some(b"bar".to_vec()));
        check.set(b"subtx", b"works").unwrap();
        check.rollback();

        assert_eq!(base.get(b"subtx").unwrap(), None);
    }

    #[test]
    fn ignore_same_as_rollback() {
        let mut base = MemoryStorage::new();
        base.set(b"foo", b"bar").unwrap();

        let mut check = StorageTransaction::new(&base);
        assert_eq!(check.get(b"foo").unwrap(), Some(b"bar".to_vec()));
        check.set(b"subtx", b"works").unwrap();

        assert_eq!(base.get(b"subtx").unwrap(), None);
    }

    #[test]
    fn transactional_works() {
        let mut base = MemoryStorage::new();
        base.set(b"foo", b"bar").unwrap();

        // writes on success
        let res: StdResult<i32> = transactional(&mut base, |store| {
            // ensure we can read from the backing store
            assert_eq!(store.get(b"foo").unwrap(), Some(b"bar".to_vec()));
            // we write in the Ok case
            store.set(b"good", b"one").unwrap();
            Ok(5)
        });
        assert_eq!(5, res.unwrap());
        assert_eq!(base.get(b"good").unwrap(), Some(b"one".to_vec()));

        // rejects on error
        let res: StdResult<i32> = transactional(&mut base, |store| {
            // ensure we can read from the backing store
            assert_eq!(store.get(b"foo").unwrap(), Some(b"bar".to_vec()));
            assert_eq!(store.get(b"good").unwrap(), Some(b"one".to_vec()));
            // we write in the Error case
            store.set(b"bad", b"value").unwrap();
            Err(unauthorized())
        });
        assert!(res.is_err());
        assert_eq!(base.get(b"bad").unwrap(), None);
    }
}
