use core::ops::{Bound, RangeBounds};

use crate::Binary;

pub trait ToByteVec {
    fn to_byte_vec(&self) -> Vec<u8>;
}
impl ToByteVec for Vec<u8> {
    fn to_byte_vec(&self) -> Vec<u8> {
        self.clone()
    }
}
impl ToByteVec for [u8] {
    fn to_byte_vec(&self) -> Vec<u8> {
        self.to_vec()
    }
}
impl<const N: usize> ToByteVec for [u8; N] {
    fn to_byte_vec(&self) -> Vec<u8> {
        self.to_vec()
    }
}
impl ToByteVec for Binary {
    fn to_byte_vec(&self) -> Vec<u8> {
        self.to_vec()
    }
}

/// Converts any range to start and end bounds for ranging through storage.
/// The start bound is inclusive, the end bound is exclusive.
pub fn range_to_bounds<'a, R, B>(range: &R) -> (Option<Vec<u8>>, Option<Vec<u8>>)
where
    R: RangeBounds<&'a B>,
    B: ToByteVec + 'a + ?Sized,
{
    let start = match range.start_bound() {
        Bound::Included(start) => Some(start.to_byte_vec()),
        Bound::Excluded(start) => Some(key_after(start.to_byte_vec())),
        Bound::Unbounded => None,
    };
    let end = match range.end_bound() {
        Bound::Included(end) => Some(key_after(end.to_byte_vec())),
        Bound::Excluded(end) => Some(end.to_byte_vec()),
        Bound::Unbounded => None,
    };
    (start, end)
}

/// Returns the key after the given key.
///
/// Reuses the given vector.
fn key_after(mut key: Vec<u8>) -> Vec<u8> {
    key.push(0);
    key
}

#[cfg(test)]
mod tests {
    use crate::{testing::MockStorage, Order, Storage};

    use super::*;

    #[test]
    fn range_to_bounds_works() {
        let mut storage = MockStorage::new();

        let keys: &[&[u8]] = &[
            &[1, 2, 3],
            &[1, 2, 4],
            &[1, 2, 5],
            &[1, 2, 6],
            &[1, 2, 7],
            &[1, 2, 7, 0],
            &[1, 2, 7, 1],
            &[1, 2, 7, 2],
            &[1, 2, 8],
            &[1, 2, 8, 0],
            &[1, 2, 8, 1],
        ];
        // map every key to its index
        for (i, &key) in keys.iter().enumerate() {
            storage.set(key, &[i as u8]);
        }

        // check the range between any two keys inside the storage
        for (idx0, &key0) in keys.iter().enumerate() {
            for (idx1, &key1) in keys.iter().enumerate() {
                // key0..key1 should have idx0..idx1 as values
                assert_range(&storage, key0..key1, (idx0..idx1).map(|idx| idx as u8));

                // key0..=key1 should have idx0..=idx1 as values
                assert_range(&storage, key0..=key1, (idx0..=idx1).map(|idx| idx as u8));
            }

            // key0.. should have idx0.. as values
            assert_range(&storage, key0.., (idx0..keys.len()).map(|idx| idx as u8));
            // ..key0 should have 0..idx0 as values
            assert_range(&storage, ..key0, (0..idx0).map(|idx| idx as u8));
            // ..=key0 should have 0..=idx0 as values
            assert_range(&storage, ..=key0, (0..=idx0).map(|idx| idx as u8));
        }

        // 0..not_in_storage should have range from start to last key before not_in_storage
        let zero: &[u8] = &[0u8];
        let not_in_storage = &[1u8, 2, 7, 3];
        assert_range(&storage, zero..not_in_storage, 0u8..=7);
        assert_range(&storage, zero..=not_in_storage, 0u8..=7);

        // 0..after_last_key should have full range
        let after_last_key: &[u8] = &[1u8, 2, 8, 2];
        assert_range(&storage, zero..after_last_key, 0u8..keys.len() as u8);
        assert_range(&storage, zero..=after_last_key, 0u8..keys.len() as u8);

        // full range
        assert_range(&storage, .., 0u8..keys.len() as u8);

        fn assert_range<'a>(
            storage: &MockStorage,
            range: impl RangeBounds<&'a [u8]>,
            expected_values: impl Iterator<Item = u8> + DoubleEndedIterator + Clone,
        ) {
            let (s, e) = range_to_bounds(&range);
            // ascending
            let values = storage
                .range_values(s.as_deref(), e.as_deref(), Order::Ascending)
                .collect::<Vec<_>>();
            assert_eq!(
                values,
                expected_values.clone().map(|v| vec![v]).collect::<Vec<_>>()
            );
            // descending
            let values = storage
                .range_values(s.as_deref(), e.as_deref(), Order::Descending)
                .collect::<Vec<_>>();
            assert_eq!(
                values,
                expected_values.rev().map(|v| vec![v]).collect::<Vec<_>>()
            );
        }
    }
}
