use serde::{Deserialize, Serialize};

use crate::errors::CheckedMultiplyFractionalError;
use crate::Uint512;

/// A fraction `p`/`q` with integers `p` and `q`.
///
/// `p` is called the numerator and `q` is called the denominator.
pub trait Fractional<T>: Sized {
    /// Returns the numerator `p`
    fn numerator(&self) -> T;
    /// Returns the denominator `q`
    fn denominator(&self) -> T;

    /// Returns the multiplicative inverse `q/p` for fraction `p/q`.
    ///
    /// If `p` is zero, None is returned.
    fn inv(&self) -> Option<Self>;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Fraction<T>(T, T);

impl<T> Fraction<T> {
    pub fn new(numerator: T, denominator: T) -> Self {
        Self(numerator, denominator)
    }
}

impl<T: Clone> Fractional<T> for Fraction<T> {
    fn numerator(&self) -> T {
        self.0.clone()
    }

    fn denominator(&self) -> T {
        self.1.clone()
    }

    fn inv(&self) -> Option<Self> {
        unimplemented!()
    }
}

pub trait FractionMath {
    fn checked_mul_floored<F: Fractional<T>, T: Into<Uint512>>(
        self,
        rhs: F,
    ) -> Result<Self, CheckedMultiplyFractionalError>
    where
        Self: Sized;

    fn mul_floored<F: Fractional<T>, T: Into<Uint512>>(self, rhs: F) -> Self;

    fn checked_mul_ceil<F: Fractional<T> + Clone, T: Into<Uint512>>(
        self,
        rhs: F,
    ) -> Result<Self, CheckedMultiplyFractionalError>
    where
        Self: Sized;

    fn mul_ceil<F: Fractional<T> + Clone, T: Into<Uint512>>(self, rhs: F) -> Self;
}

#[macro_export]
macro_rules! fraction_math {
    ($name:ident) => {
        impl FractionMath for $name {
            fn checked_mul_floored<F: Fractional<T>, T: Into<Uint512>>(
                self,
                rhs: F,
            ) -> Result<Self, CheckedMultiplyFractionalError> {
                let res = Uint512::from(self)
                    .checked_mul(rhs.numerator().into())?
                    .checked_div(rhs.denominator().into())?;
                Ok(res.try_into()?)
            }

            fn mul_floored<F: Fractional<T>, T: Into<Uint512>>(self, rhs: F) -> Self {
                self.checked_mul_floored(rhs).unwrap()
            }

            fn mul_ceil<F: Fractional<T> + Clone, T: Into<Uint512>>(self, rhs: F) -> Self {
                self.checked_mul_ceil(rhs).unwrap()
            }

            fn checked_mul_ceil<F: Fractional<T> + Clone, T: Into<Uint512>>(
                self,
                rhs: F,
            ) -> Result<Self, CheckedMultiplyFractionalError> {
                let floor_result = self.checked_mul_floored(rhs.clone())?;
                let numerator = rhs.numerator().into();
                let denominator = rhs.denominator().into();
                if !numerator.checked_rem(denominator)?.is_zero() {
                    let ceil_result = Uint512::one().checked_add(floor_result.into())?;
                    Ok(ceil_result.try_into()?)
                } else {
                    Ok(floor_result)
                }
            }
        }
    };
}
