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

    /// An trait that ensures other traits are implemented for our number types
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

    impl AllImpl<'_> for Uint64 {}
    impl AllImpl<'_> for Uint128 {}
    impl AllImpl<'_> for Uint256 {}
    impl AllImpl<'_> for Uint512 {}

    // TODO: When all implementations are done, extra trait can be removed and
    // unified with AllImpl
    trait AllImplDecimal<'a>:
        Add
        // + Add<&'a Self>
        // + AddAssign
        // + AddAssign<&'a Self>
        + Sub
        // + Sub<&'a Self>
        // + SubAssign
        // + SubAssign<&'a Self>
        + Mul
        // + Mul<&'a Self>
        // + MulAssign
        // + MulAssign<&'a Self>
        // + Div
        // + Div<&'a Self>
        // + DivAssign
        // + DivAssign<&'a Self>
        // + Rem
        // + Rem<&'a Self>
        // + RemAssign
        // + RemAssign<&'a Self>
        + Sized
        + Copy
    where
        Self: 'a,
    {
    }

    impl AllImplDecimal<'_> for Decimal {}
    impl AllImplDecimal<'_> for Decimal256 {}
}
