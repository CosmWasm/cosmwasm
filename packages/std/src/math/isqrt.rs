use std::{cmp, ops};

/// A trait for calculating the
/// [integer square root](https://en.wikipedia.org/wiki/Integer_square_root).
pub trait Isqrt {
    /// The [integer square root](https://en.wikipedia.org/wiki/Integer_square_root).
    fn isqrt(self) -> Self;
}

impl<
        I: ops::Add<I, Output = I>
            + ops::Div<I, Output = I>
            + ops::Shr<u8, Output = I>
            + cmp::PartialOrd
            + Copy
            + From<u8>,
    > Isqrt for I
{
    /// Algorithm adapted from
    /// [Wikipedia](https://en.wikipedia.org/wiki/Integer_square_root#Example_implementation_in_C).
    fn isqrt(self) -> Self {
        let mut x0 = self >> 1;

        if x0 > 0.into() {
            let mut x1 = (x0 + self / x0) >> 1;

            while x1 < x0 {
                x0 = x1;
                x1 = (x0 + self / x0) >> 1;
            }

            return x0;
        } else if x0 < 0.into() {
            panic!("attempt to calculate the integer square root of a negative number");
        }
        self
    }
}

#[test]
fn uint128_sqrts() {
    // Let's check correctness.
    assert_eq!(0.isqrt(), 0);
    assert_eq!(1.isqrt(), 1);
    assert_eq!(24.isqrt(), 4);
    assert_eq!(25.isqrt(), 5);
    assert_eq!(26.isqrt(), 5);
    assert_eq!(36.isqrt(), 6);

    // Let's also check different types.
    assert_eq!(26u8.isqrt(), 5);
    assert_eq!(26u128.isqrt(), 5);
}
