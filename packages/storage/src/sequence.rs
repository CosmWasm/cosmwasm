use cosmwasm_std::{ReadonlyStorage, StdResult, Storage};

use crate::singleton::{singleton, Singleton};

/// Sequence creates a custom Singleton to hold an empty sequence
pub fn sequence(key: &[u8]) -> Singleton<u64> {
    Singleton::new(key)
}

/// currval returns the last value returned by nextval. If the sequence has never been used,
/// then it will return 0.
pub fn currval<S: ReadonlyStorage>(store: &S, key: &[u8]) -> StdResult<u64> {
    Ok(singleton(key).may_load(store)?.unwrap_or_default())
}

/// nextval increments the counter by 1 and returns the new value.
/// On the first time it is called (no sequence info in db) it will return 1.
pub fn nextval<S: Storage>(store: &mut S, key: &[u8]) -> StdResult<u64> {
    let seq = singleton(key);
    let val = seq.may_load(store)?.unwrap_or_default() + 1;
    seq.save(store, &val)?;
    Ok(val)
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::MockStorage;

    #[test]
    fn walk_through_sequence() {
        let mut store = MockStorage::new();
        let key: &[u8] = b"seq";

        assert_eq!(currval(&store, key).unwrap(), 0);
        assert_eq!(nextval(&mut store, key).unwrap(), 1);
        assert_eq!(nextval(&mut store, key).unwrap(), 2);
        assert_eq!(nextval(&mut store, key).unwrap(), 3);
        assert_eq!(currval(&store, key).unwrap(), 3);
        assert_eq!(currval(&store, key).unwrap(), 3);
    }

    #[test]
    fn sequences_independent() {
        let mut store = MockStorage::new();

        let key: &[u8] = b"seq";
        assert_eq!(nextval(&mut store, key).unwrap(), 1);
        assert_eq!(nextval(&mut store, key).unwrap(), 2);
        assert_eq!(nextval(&mut store, key).unwrap(), 3);

        let key2: &[u8] = b"seq2";
        assert_eq!(nextval(&mut store, key2).unwrap(), 1);
        assert_eq!(nextval(&mut store, key2).unwrap(), 2);

        assert_eq!(nextval(&mut store, key).unwrap(), 4);
    }

    #[test]
    fn set_sequence() {
        let mut store = MockStorage::new();
        let key: &[u8] = b"food";

        assert_eq!(nextval(&mut store, key).unwrap(), 1);
        assert_eq!(nextval(&mut store, key).unwrap(), 2);

        sequence(key).save(&mut store, &20).unwrap();

        assert_eq!(currval(&store, key).unwrap(), 20);
        assert_eq!(nextval(&mut store, key).unwrap(), 21);
    }
}
