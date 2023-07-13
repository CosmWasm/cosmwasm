//! Module with common routines used internally by the library in unit tests.

use core::hash::{Hash, Hasher};

/// Tests that type `T` implements `Hash` trait correctly.
///
/// `foo` and `bar` must be different objects.
///
/// Some object pairs may produce the same hash causing test failure.  In those
/// cases try different objects.  The test uses stable hasher so once working
/// pair is identified, the test’s going to continue passing.
pub(crate) fn check_hash_impl<T: Clone + Hash>(foo: T, bar: T) {
    let foo_copy = foo.clone();

    fn hash<T: Hash>(value: &T) -> u64 {
        let mut hasher = crc32fast::Hasher::default();
        value.hash(&mut hasher);
        hasher.finish()
    }

    let foo_hash = hash(&foo);
    assert_eq!(foo_hash, hash(&foo_copy));
    assert_ne!(foo_hash, hash(&bar));
}
