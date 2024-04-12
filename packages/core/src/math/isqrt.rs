use core::{cmp, ops};

use crate::{Uint128, Uint256, Uint512, Uint64};

/// A trait for calculating the
/// [integer square root](https://en.wikipedia.org/wiki/Integer_square_root).
pub trait Isqrt {
    /// The [integer square root](https://en.wikipedia.org/wiki/Integer_square_root).
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn isqrt(self) -> Self;
}

impl<I> Isqrt for I
where
    I: Unsigned
        + ops::Add<I, Output = I>
        + ops::Div<I, Output = I>
        + ops::Shl<u32, Output = I>
        + ops::Shr<u32, Output = I>
        + cmp::PartialOrd
        + Copy,
{
    /// Algorithm adapted from
    /// [Wikipedia](https://en.wikipedia.org/wiki/Integer_square_root#Example_implementation_in_C).
    fn isqrt(self) -> Self {
        // sqrt(0) = 0, sqrt(1) = 1
        if self <= Self::ONE {
            return self;
        }

        let mut x0 = Self::ONE << ((self.log_2() / 2) + 1);

        if x0 > Self::ZERO {
            let mut x1 = (x0 + self / x0) >> 1;

            while x1 < x0 {
                x0 = x1;
                x1 = (x0 + self / x0) >> 1;
            }

            return x0;
        }
        self
    }
}

/// Marker trait for types that represent unsigned integers.
pub trait Unsigned {
    const ZERO: Self;
    const ONE: Self;

    fn log_2(self) -> u32;
}

macro_rules! impl_unsigned {
    ($type:ty, $zero:expr, $one:expr) => {
        impl Unsigned for $type {
            const ZERO: Self = $zero;
            const ONE: Self = $one;

            fn log_2(self) -> u32 {
                self.ilog2()
            }
        }
    };
}
impl_unsigned!(u8, 0, 1);
impl_unsigned!(u16, 0, 1);
impl_unsigned!(u32, 0, 1);
impl_unsigned!(u64, 0, 1);
impl_unsigned!(u128, 0, 1);
impl_unsigned!(usize, 0, 1);
impl_unsigned!(Uint64, Self::zero(), Self::one());
impl_unsigned!(Uint128, Self::zero(), Self::one());
impl_unsigned!(Uint256, Self::zero(), Self::one());
impl_unsigned!(Uint512, Self::zero(), Self::one());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isqrt_primitives() {
        // Let's check correctness.
        assert_eq!(super::Isqrt::isqrt(0u8), 0);
        assert_eq!(super::Isqrt::isqrt(1u8), 1);
        assert_eq!(super::Isqrt::isqrt(24u8), 4);
        assert_eq!(super::Isqrt::isqrt(25u8), 5);
        assert_eq!(super::Isqrt::isqrt(26u8), 5);
        assert_eq!(super::Isqrt::isqrt(36u8), 6);

        // Let's also check different types.
        assert_eq!(super::Isqrt::isqrt(26u8), 5);
        assert_eq!(super::Isqrt::isqrt(26u16), 5);
        assert_eq!(super::Isqrt::isqrt(26u32), 5);
        assert_eq!(super::Isqrt::isqrt(26u64), 5);
        assert_eq!(super::Isqrt::isqrt(26u128), 5);
    }

    #[test]
    fn isqrt_uint64() {
        assert_eq!(Uint64::new(24).isqrt(), Uint64::new(4));
    }

    #[test]
    fn isqrt_uint128() {
        assert_eq!(Uint128::new(24).isqrt(), Uint128::new(4));
    }

    #[test]
    fn isqrt_uint256() {
        assert_eq!(Uint256::from(24u32).isqrt(), Uint256::from(4u32));
        assert_eq!(
            (Uint256::from(u128::MAX) * Uint256::from(u128::MAX)).isqrt(),
            Uint256::try_from("340282366920938463463374607431768211455").unwrap()
        );
    }

    #[test]
    fn isqrt_uint512() {
        assert_eq!(Uint512::from(24u32).isqrt(), Uint512::from(4u32));
        assert_eq!(
            (Uint512::from(Uint256::MAX) * Uint512::from(Uint256::MAX)).isqrt(),
            Uint512::try_from(
                "115792089237316195423570985008687907853269984665640564039457584007913129639935"
            )
            .unwrap()
        );
    }
}
