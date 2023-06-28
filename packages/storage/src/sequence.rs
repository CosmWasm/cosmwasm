use crate::cosmwasm_std::{StdResult, Storage};

use crate::Singleton;

/// Sequence creates a custom Singleton to hold an empty sequence
pub fn sequence<'a>(storage: &'a mut dyn Storage, key: &[u8]) -> Singleton<'a, u64> {
    Singleton::new(storage, key)
}

/// currval returns the last value returned by nextval. If the sequence has never been used,
/// then it will return 0.
pub fn currval(seq: &Singleton<u64>) -> StdResult<u64> {
    Ok(seq.may_load()?.unwrap_or_default())
}

/// nextval increments the counter by 1 and returns the new value.
/// On the first time it is called (no sequence info in db) it will return 1.
pub fn nextval(seq: &mut Singleton<u64>) -> StdResult<u64> {
    let val = currval(seq)? + 1;
    seq.save(&val)?;
    Ok(val)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cosmwasm_std::testing::MockStorage;

    #[test]
    fn walk_through_sequence() {
        let mut store = MockStorage::new();
        let mut seq = sequence(&mut store, b"seq");

        assert_eq!(currval(&seq).unwrap(), 0);
        assert_eq!(nextval(&mut seq).unwrap(), 1);
        assert_eq!(nextval(&mut seq).unwrap(), 2);
        assert_eq!(nextval(&mut seq).unwrap(), 3);
        assert_eq!(currval(&seq).unwrap(), 3);
        assert_eq!(currval(&seq).unwrap(), 3);
    }

    #[test]
    fn sequences_independent() {
        let mut store = MockStorage::new();

        let mut seq = sequence(&mut store, b"seq");
        assert_eq!(nextval(&mut seq).unwrap(), 1);
        assert_eq!(nextval(&mut seq).unwrap(), 2);
        assert_eq!(nextval(&mut seq).unwrap(), 3);

        let mut seq2 = sequence(&mut store, b"seq2");
        assert_eq!(nextval(&mut seq2).unwrap(), 1);
        assert_eq!(nextval(&mut seq2).unwrap(), 2);

        let mut seq3 = sequence(&mut store, b"seq");
        assert_eq!(nextval(&mut seq3).unwrap(), 4);
    }

    #[test]
    fn set_sequence() {
        let mut store = MockStorage::new();
        let mut seq = sequence(&mut store, b"seq");

        assert_eq!(nextval(&mut seq).unwrap(), 1);
        assert_eq!(nextval(&mut seq).unwrap(), 2);

        seq.save(&20).unwrap();

        assert_eq!(currval(&seq).unwrap(), 20);
        assert_eq!(nextval(&mut seq).unwrap(), 21);
    }
}
