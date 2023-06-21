mod decimal;
mod decimal256;
mod fraction;
mod isqrt;
mod uint128;
mod uint256;
mod uint512;
mod uint64;

pub use decimal::{Decimal, DecimalRangeExceeded};
pub use decimal256::{Decimal256, Decimal256RangeExceeded};
pub use fraction::Fraction;
pub use isqrt::Isqrt;
pub use uint128::Uint128;
pub use uint256::Uint256;
pub use uint512::Uint512;
pub use uint64::Uint64;

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::*;

    /// A trait that ensures other traits are implemented for our number types
    trait AllImpl<'a>:
        Add
        + Add<&'a Self>
        + AddAssign
        + AddAssign<&'a Self>
        + Sub
        + Sub<&'a Self>
        + SubAssign
        + SubAssign<&'a Self>
        + Mul
        + Mul<&'a Self>
        + MulAssign
        + MulAssign<&'a Self>
        + Div
        + Div<&'a Self>
        + DivAssign
        + DivAssign<&'a Self>
        + Rem
        + Rem<&'a Self>
        + RemAssign
        + RemAssign<&'a Self>
        + Sized
        + Copy
    where
        Self: 'a,
    {
    }

    /// A trait that ensures other traits are implemented for our integer types
    trait IntImpl<'a>:
        AllImpl<'a>
        + Shl<u32>
        + Shl<&'a u32>
        + ShlAssign<u32>
        + ShlAssign<&'a u32>
        + Shr<u32>
        + Shr<&'a u32>
        + ShrAssign<u32>
        + ShrAssign<&'a u32>
    {
    }

    impl AllImpl<'_> for Uint64 {}
    impl AllImpl<'_> for Uint128 {}
    impl AllImpl<'_> for Uint256 {}
    impl AllImpl<'_> for Uint512 {}
    impl IntImpl<'_> for Uint64 {}
    impl IntImpl<'_> for Uint128 {}
    impl IntImpl<'_> for Uint256 {}
    impl IntImpl<'_> for Uint512 {}
    impl AllImpl<'_> for Decimal {}
    impl AllImpl<'_> for Decimal256 {}
}
