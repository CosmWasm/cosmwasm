use std::{cmp, ops};

/// A trait for calculating the
/// [integer square root](https://en.wikipedia.org/wiki/Integer_square_root).
pub trait Isqrt {
    /// The [integer square root](https://en.wikipedia.org/wiki/Integer_square_root).
    fn isqrt(self) -> Self;
}

impl<I> Isqrt for I
where
    I: Unsigned
        + ops::Add<I, Output = I>
        + ops::Div<I, Output = I>
        + ops::Shr<u8, Output = I>
        + cmp::PartialOrd
        + Copy
        + From<u8>,
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

/// Marker trait for types that represent unsigned integers.
pub trait Unsigned {}
impl Unsigned for u8 {}
impl Unsigned for u16 {}
impl Unsigned for u32 {}
impl Unsigned for u64 {}
impl Unsigned for u128 {}

#[test]
fn isqrts() {
    // Let's check correctness.
    assert_eq!(0u8.isqrt(), 0);
    assert_eq!(1u8.isqrt(), 1);
    assert_eq!(24u8.isqrt(), 4);
    assert_eq!(25u8.isqrt(), 5);
    assert_eq!(26u8.isqrt(), 5);
    assert_eq!(36u8.isqrt(), 6);

    // Let's also check different types.
    assert_eq!(26u8.isqrt(), 5);
    assert_eq!(26u16.isqrt(), 5);
    assert_eq!(26u32.isqrt(), 5);
    assert_eq!(26u64.isqrt(), 5);
    assert_eq!(26u128.isqrt(), 5);
}
