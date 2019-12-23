use crate::errors::Result;
use crate::mock::MockStorage;
use crate::traits::{Api, Extern, ReadonlyStorage, Storage};

pub struct StorageTransaction<'a, S: Storage> {
    /// a backing storage that is only modified upon commit
    storage: &'a mut S,
    /// these are local changes not flushed to backing storage
    local_state: MockStorage,
    /// this is a list of changes to be written to backing storage upon commit
    rep_log: Vec<Op>,
}

enum Op {
    Set { key: Vec<u8>, value: Vec<u8> },
}

impl<'a, S: Storage> StorageTransaction<'a, S> {
    pub fn new(storage: &'a mut S) -> Self {
        StorageTransaction {
            storage,
            local_state: MockStorage::new(),
            rep_log: vec![],
        }
    }

    /// commit will consume the checkpoint and write all changes to the underlying store
    pub fn commit(self) {
        for op in self.rep_log.iter() {
            match op {
                Op::Set { key, value } => self.storage.set(&key, &value),
            }
        }
    }

    /// rollback will consume the checkpoint and drop all changes (no really needed, going out of scope does the same, but nice for clarity)
    pub fn rollback(self) {}
}

impl<'a, S: Storage> ReadonlyStorage for StorageTransaction<'a, S> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        match self.local_state.get(key) {
            Some(val) => Some(val),
            None => self.storage.get(key),
        }
    }
}

impl<'a, S: Storage> Storage for StorageTransaction<'a, S> {
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
    let mut c = StorageTransaction::new(storage);
    let res = tx(&mut c)?;
    c.commit();
    Ok(res)
}

pub fn transactional_deps<S: Storage, A: Api, T>(
    deps: &mut Extern<S, A>,
    tx: &dyn Fn(&mut Extern<StorageTransaction<S>, A>) -> Result<T>,
) -> Result<T> {
    let c = StorageTransaction::new(&mut deps.storage);
    let mut deps = Extern {
        storage: c,
        api: deps.api,
    };
    let res = tx(&mut deps);
    if res.is_ok() {
        deps.storage.commit();
    } else {
        deps.storage.rollback();
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

        let mut check = StorageTransaction::new(&mut base);
        assert_eq!(check.get(b"foo"), Some(b"bar".to_vec()));
        check.set(b"subtx", b"works");
        check.commit();

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
