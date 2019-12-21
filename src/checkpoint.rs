use crate::traits::{Storage, ReadonlyStorage};
use crate::errors::Result;
use crate::mock::MockStorage;

pub struct Checkpoint<'a, S: Storage> {
    storage: &'a mut S,
    local_state: MockStorage,
    rep_log: Vec<Op>,
}

enum Op {
    Set{key: Vec<u8>, value: Vec<u8>},
}

impl<'a, S: Storage> Checkpoint<'a, S> {
    pub fn new(storage: &'a mut S) -> Self {
        Checkpoint{
            storage,
            local_state: MockStorage::new(),
            rep_log: vec![],
        }
    }

    /// commit will consume the checkpoint and write all changes to the underlying store
    pub fn commit(self) {
        for op in self.rep_log.iter() {
            match op {
                Op::Set{key, value} => self.storage.set(&key, &value),
            }
        }
    }

    /// rollback will consume the checkpoint and drop all changes (no really needed, going out of scope does the same, but nice for clarity)
    pub fn rollback(self) {}
}

impl<'a, S: Storage> ReadonlyStorage for Checkpoint<'a, S> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        match self.local_state.get(key) {
            Some(val) => Some(val),
            None => self.storage.get(key),
        }
    }
}

impl<'a, S: Storage> Storage for Checkpoint<'a, S> {
    fn set(&mut self, key: &[u8], value: &[u8]) {
        self.local_state.set(key, value);
        self.rep_log.push(Op::Set{key: key.to_vec(), value: value.to_vec()})
    }
}


pub fn checkpoint<S: Storage, T>(storage: &mut S, tx: &dyn Fn(&mut Checkpoint<S>) -> Result<T>) -> Result<T>  {
    let mut c = Checkpoint::new(storage);
    let res = tx(&mut c)?;
    c.commit();
    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock::MockStorage;

    #[test]
    fn checkpoint_writes_through() {
        let mut base = MockStorage::new();
        base.set(b"foo", b"bar");

        let mut check = Checkpoint::new(&mut base);
        assert_eq!(Some(b"bar".to_vec()), check.get(b"foo"));
        check.set(b"subtx", b"works");
        check.commit();

        assert_eq!(Some(b"works".to_vec()), base.get(b"subtx"));
    }
}