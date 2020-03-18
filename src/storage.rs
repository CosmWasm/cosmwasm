use crate::errors::Result;
use crate::mock::MockStorage;
use crate::traits::{Api, Extern, ReadonlyStorage, Storage};

pub struct StorageTransaction<'a, S: ReadonlyStorage> {
    /// read-only access to backing storage
    storage: &'a S,
    /// these are local changes not flushed to backing storage
    local_state: MockStorage,
    /// this is a list of changes to be written to backing storage upon commit
    rep_log: Vec<Op>,
}

pub struct Commit {
    /// this is a list of changes to be written to backing storage upon commit
    rep_log: Vec<Op>,
}

impl Commit {
    fn new(rep_log: Vec<Op>) -> Self {
        Commit { rep_log }
    }

    pub fn commit<S: Storage>(self, storage: &mut S) {
        for op in self.rep_log {
            op.apply(storage);
        }
    }
}

enum Op {
    Set { key: Vec<u8>, value: Vec<u8> },
}

impl Op {
    pub fn apply<S: Storage>(&self, storage: &mut S) {
        match self {
            Op::Set { key, value } => storage.set(&key, &value),
        }
    }
}

impl<'a, S: ReadonlyStorage> StorageTransaction<'a, S> {
    pub fn new(storage: &'a S) -> Self {
        StorageTransaction {
            storage,
            local_state: MockStorage::new(),
            rep_log: vec![],
        }
    }

    pub fn prepare(self) -> Commit {
        Commit::new(self.rep_log)
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
}

impl<'a, S: ReadonlyStorage> Storage for StorageTransaction<'a, S> {
    fn set(&mut self, key: &[u8], value: &[u8]) {
        self.local_state.set(key, value);
        self.rep_log.push(Op::Set {
            key: key.to_vec(),
            value: value.to_vec(),
        })
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
    use crate::mock::MockStorage;

    #[test]
    fn commit_writes_through() {
        let mut base = MockStorage::new();
        base.set(b"foo", b"bar");

        let mut check = StorageTransaction::new(&base);
        assert_eq!(check.get(b"foo"), Some(b"bar".to_vec()));
        check.set(b"subtx", b"works");
        check.prepare().commit(&mut base);

        assert_eq!(base.get(b"subtx"), Some(b"works".to_vec()));
    }

    #[test]
    fn storage_remains_readable() {
        let mut base = MockStorage::new();
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
        let mut base = MockStorage::new();
        base.set(b"foo", b"bar");

        let mut check = StorageTransaction::new(&mut base);
        assert_eq!(check.get(b"foo"), Some(b"bar".to_vec()));
        check.set(b"subtx", b"works");
        check.rollback();

        assert_eq!(base.get(b"subtx"), None);
    }

    #[test]
    fn ignore_same_as_rollback() {
        let mut base = MockStorage::new();
        base.set(b"foo", b"bar");

        let mut check = StorageTransaction::new(&mut base);
        assert_eq!(check.get(b"foo"), Some(b"bar".to_vec()));
        check.set(b"subtx", b"works");

        assert_eq!(base.get(b"subtx"), None);
    }

    #[test]
    fn transactional_works() {
        let mut base = MockStorage::new();
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
