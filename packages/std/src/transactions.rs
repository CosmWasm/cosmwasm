#[cfg(feature = "iterator")]
use std::cmp::Ordering;
#[cfg(feature = "iterator")]
use std::iter::Peekable;

use crate::errors::Result;
use crate::storage::MemoryStorage;
use crate::traits::{Api, Extern, ReadonlyStorage, Storage};
#[cfg(feature = "iterator")]
use crate::traits::{Order, Pair};

pub struct StorageTransaction<'a, S: ReadonlyStorage> {
    /// read-only access to backing storage
    storage: &'a S,
    /// these are local changes not flushed to backing storage
    local_state: MemoryStorage,
    /// a log of local changes not yet flushed to backing storage
    rep_log: RepLog,
}

impl<'a, S: ReadonlyStorage> StorageTransaction<'a, S> {
    pub fn new(storage: &'a S) -> Self {
        StorageTransaction {
            storage,
            local_state: MemoryStorage::new(),
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
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        match self.local_state.get(key) {
            Some(val) => Some(val),
            None => self.storage.get(key),
        }
    }

    #[cfg(feature = "iterator")]
    /// range allows iteration over a set of keys, either forwards or backwards
    /// uses standard rust range notation, and eg db.range(b"foo"..b"bar") also works reverse
    fn range(
        &self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = Pair>> {
        let local = self.local_state.range(start, end, order);
        let base = self.storage.range(start, end, order);
        let merged = MergeOrdered::new(local, base, order);
        Box::new(merged)
    }
}

impl<'a, S: ReadonlyStorage> Storage for StorageTransaction<'a, S> {
    fn set(&mut self, key: &[u8], value: &[u8]) {
        let op = Op::Set {
            key: key.to_vec(),
            value: value.to_vec(),
        };
        op.apply(&mut self.local_state);
        self.rep_log.append(op);
    }

    fn remove(&mut self, key: &[u8]) {
        // TODO: handle overlay better
        let op = Op::Delete { key: key.to_vec() };
        op.apply(&mut self.local_state);
        self.rep_log.append(op);
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
    pub fn commit<S: Storage>(self, storage: &mut S) {
        for op in self.ops_log {
            op.apply(storage);
        }
    }
}

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
    pub fn apply<S: Storage>(&self, storage: &mut S) {
        match self {
            Op::Set { key, value } => storage.set(&key, &value),
            Op::Delete { key } => storage.remove(&key),
        }
    }
}

#[cfg(feature = "iterator")]
struct MergeOrdered<L, R>
where
    L: Iterator<Item = R::Item>,
    R: Iterator,
{
    left: Peekable<L>,
    right: Peekable<R>,
    order: Order,
}

#[cfg(feature = "iterator")]
impl<L, R> MergeOrdered<L, R>
where
    L: Iterator<Item = R::Item>,
    R: Iterator,
{
    fn new(left: L, right: R, order: Order) -> Self {
        MergeOrdered {
            left: left.peekable(),
            right: right.peekable(),
            order,
        }
    }
}

#[cfg(feature = "iterator")]
impl<L, R> Iterator for MergeOrdered<L, R>
where
    L: Iterator<Item = R::Item>,
    R: Iterator,
    L::Item: Ord,
{
    type Item = L::Item;

    fn next(&mut self) -> Option<L::Item> {
        match (self.left.peek(), self.right.peek()) {
            (Some(l), Some(r)) => {
                let order = match self.order {
                    Order::Ascending => l.cmp(r),
                    Order::Descending => r.cmp(l),
                };
                match order {
                    Ordering::Less => self.left.next(),
                    Ordering::Equal => self.left.next(),
                    Ordering::Greater => self.right.next(),
                }
            }
            (Some(_), None) => self.left.next(),
            (None, Some(_)) => self.right.next(),
            (None, None) => None,
        }
    }
}

pub fn transactional<S: Storage, T>(
    storage: &mut S,
    tx: &dyn Fn(&mut StorageTransaction<S>) -> Result<T>,
) -> Result<T> {
    let mut stx = StorageTransaction::new(storage);
    let res = tx(&mut stx)?;
    stx.prepare().commit(storage);
    Ok(res)
}

pub fn transactional_deps<S: Storage, A: Api, T>(
    deps: &mut Extern<S, A>,
    tx: &dyn Fn(&mut Extern<StorageTransaction<S>, A>) -> Result<T>,
) -> Result<T> {
    let c = StorageTransaction::new(&deps.storage);
    let mut stx_deps = Extern {
        storage: c,
        api: deps.api,
    };
    let res = tx(&mut stx_deps);
    if res.is_ok() {
        stx_deps.storage.prepare().commit(&mut deps.storage);
    } else {
        stx_deps.storage.rollback();
    }
    res
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::errors::Unauthorized;

    #[test]
    fn delete_local() {
        let mut base = MemoryStorage::new();
        let mut check = StorageTransaction::new(&base);
        check.set(b"foo", b"bar");
        check.set(b"food", b"bank");
        check.remove(b"foo");

        assert_eq!(None, check.get(b"foo"));
        assert_eq!(Some(b"bank".to_vec()), check.get(b"food"));

        // now commit to base and query there
        check.prepare().commit(&mut base);
        assert_eq!(None, base.get(b"foo"));
        assert_eq!(Some(b"bank".to_vec()), base.get(b"food"));
    }

    #[test]
    #[ignore]
    fn delete_from_base() {
        let mut base = MemoryStorage::new();
        base.set(b"foo", b"bar");
        let mut check = StorageTransaction::new(&base);
        check.set(b"food", b"bank");
        check.remove(b"foo");

        assert_eq!(None, check.get(b"foo"));
        assert_eq!(Some(b"bank".to_vec()), check.get(b"food"));

        // now commit to base and query there
        check.prepare().commit(&mut base);
        assert_eq!(None, base.get(b"foo"));
        assert_eq!(Some(b"bank".to_vec()), base.get(b"food"));
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn storage_transaction_iterator_empty_base() {
        let base = MemoryStorage::new();
        let mut check = StorageTransaction::new(&base);
        check.set(b"foo", b"bar");
        crate::storage::iterator_test_suite(&mut check);
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn storage_transaction_iterator_with_base_data() {
        let mut base = MemoryStorage::new();
        base.set(b"foo", b"bar");
        let mut check = StorageTransaction::new(&base);
        crate::storage::iterator_test_suite(&mut check);
    }

    // TODO: check iterators working with some deleted data
    #[test]
    #[ignore]
    #[cfg(feature = "iterator")]
    fn storage_transaction_iterator_removed_items_from_base() {
        let mut base = MemoryStorage::new();
        base.set(b"foo", b"bar");
        base.set(b"food", b"bank");
        let mut check = StorageTransaction::new(&base);
        check.remove(b"food");
        crate::storage::iterator_test_suite(&mut check);
    }

    #[test]
    fn commit_writes_through() {
        let mut base = MemoryStorage::new();
        base.set(b"foo", b"bar");

        let mut check = StorageTransaction::new(&base);
        assert_eq!(check.get(b"foo"), Some(b"bar".to_vec()));
        check.set(b"subtx", b"works");
        check.prepare().commit(&mut base);

        assert_eq!(base.get(b"subtx"), Some(b"works".to_vec()));
    }

    #[test]
    fn storage_remains_readable() {
        let mut base = MemoryStorage::new();
        base.set(b"foo", b"bar");

        let mut stxn1 = StorageTransaction::new(&base);

        assert_eq!(stxn1.get(b"foo"), Some(b"bar".to_vec()));

        stxn1.set(b"subtx", b"works");
        assert_eq!(stxn1.get(b"subtx"), Some(b"works".to_vec()));

        // Can still read from base, txn is not yet committed
        assert_eq!(base.get(b"subtx"), None);

        stxn1.prepare().commit(&mut base);
        assert_eq!(base.get(b"subtx"), Some(b"works".to_vec()));
    }

    #[test]
    fn rollback_has_no_effect() {
        let mut base = MemoryStorage::new();
        base.set(b"foo", b"bar");

        let mut check = StorageTransaction::new(&base);
        assert_eq!(check.get(b"foo"), Some(b"bar".to_vec()));
        check.set(b"subtx", b"works");
        check.rollback();

        assert_eq!(base.get(b"subtx"), None);
    }

    #[test]
    fn ignore_same_as_rollback() {
        let mut base = MemoryStorage::new();
        base.set(b"foo", b"bar");

        let mut check = StorageTransaction::new(&base);
        assert_eq!(check.get(b"foo"), Some(b"bar".to_vec()));
        check.set(b"subtx", b"works");

        assert_eq!(base.get(b"subtx"), None);
    }

    #[test]
    fn transactional_works() {
        let mut base = MemoryStorage::new();
        base.set(b"foo", b"bar");

        // writes on success
        let res: Result<i32> = transactional(&mut base, &|store| {
            // ensure we can read from the backing store
            assert_eq!(store.get(b"foo"), Some(b"bar".to_vec()));
            // we write in the Ok case
            store.set(b"good", b"one");
            Ok(5)
        });
        assert_eq!(5, res.unwrap());
        assert_eq!(base.get(b"good"), Some(b"one".to_vec()));

        // rejects on error
        let res: Result<i32> = transactional(&mut base, &|store| {
            // ensure we can read from the backing store
            assert_eq!(store.get(b"foo"), Some(b"bar".to_vec()));
            assert_eq!(store.get(b"good"), Some(b"one".to_vec()));
            // we write in the Error case
            store.set(b"bad", b"value");
            Unauthorized.fail()
        });
        assert!(res.is_err());
        assert_eq!(base.get(b"bad"), None);
    }
}
