/// Performs a perfect shuffle (in shuffle)
///
/// https://en.wikipedia.org/wiki/Riffle_shuffle_permutation#Perfect_shuffles
/// https://en.wikipedia.org/wiki/In_shuffle
///
/// The number of shuffles required to restore the original order are listed in
/// https://oeis.org/A002326, e.g.:
///
/// ```ignore
/// 2: 2
/// 4: 4
/// 6: 3
/// 8: 6
/// 10: 10
/// 12: 12
/// 14: 4
/// 16: 8
/// 18: 18
/// 20: 6
/// 22: 11
/// 24: 20
/// 26: 18
/// 28: 28
/// 30: 5
/// 32: 10
/// 34: 12
/// 36: 36
/// 38: 12
/// 40: 20
/// 42: 14
/// 44: 12
/// 46: 23
/// 48: 21
/// 50: 8
/// 52: 52
/// 54: 20
/// 56: 18
/// 58: 58
/// 60: 60
/// 62: 6
/// 64: 12
/// 66: 66
/// 68: 22
/// 70: 35
/// 72: 9
/// 74: 20
/// ```
pub fn riffle_shuffle<T: Clone>(input: &[T]) -> Vec<T> {
    assert!(
        input.len() % 2 == 0,
        "Method only defined for even number of elements"
    );
    let mid = input.len() / 2;
    let (left, right) = input.split_at(mid);
    let mut out = Vec::<T>::with_capacity(input.len());
    for i in 0..mid {
        out.push(right[i].clone());
        out.push(left[i].clone());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn riffle_shuffle_works() {
        // Example from https://en.wikipedia.org/wiki/In_shuffle
        let start = [0xA, 0x2, 0x3, 0x4, 0x5, 0x6];
        let round1 = riffle_shuffle(&start);
        assert_eq!(round1, [0x4, 0xA, 0x5, 0x2, 0x6, 0x3]);
        let round2 = riffle_shuffle(&round1);
        assert_eq!(round2, [0x2, 0x4, 0x6, 0xA, 0x3, 0x5]);
        let round3 = riffle_shuffle(&round2);
        assert_eq!(round3, start);

        // For 14 elements, the original order is restored after 4 executions
        // See https://en.wikipedia.org/wiki/In_shuffle#Mathematics and https://oeis.org/A002326
        let original = [12, 33, 76, 576, 0, 44, 1, 14, 78, 99, 871212, -7, 2, -1];
        let mut result = Vec::from(original);
        for _ in 0..4 {
            result = riffle_shuffle(&result);
        }
        assert_eq!(result, original);

        // For 24 elements, the original order is restored after 20 executions
        let original = [
            7, 4, 2, 4656, 23, 45, 23, 1, 12, 76, 576, 0, 12, 1, 14, 78, 99, 12, 1212, 444, 31,
            111, 424, 34,
        ];
        let mut result = Vec::from(original);
        for _ in 0..20 {
            result = riffle_shuffle(&result);
        }
        assert_eq!(result, original);
    }
}
