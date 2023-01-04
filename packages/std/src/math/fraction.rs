/// A fraction `p`/`q` with integers `p` and `q`.
///
/// `p` is called the numerator and `q` is called the denominator.
pub trait Fraction<T>: Sized {
    /// Returns the numerator `p`
    fn numerator(&self) -> T;
    /// Returns the denominator `q`
    fn denominator(&self) -> T;

    /// Returns the multiplicative inverse `q/p` for fraction `p/q`.
    ///
    /// If `p` is zero, None is returned.
    fn inv(&self) -> Option<Self>;
}

impl<T: Clone> Fraction<T> for (T, T) {
    fn numerator(&self) -> T {
        self.0.clone()
    }

    fn denominator(&self) -> T {
        self.1.clone()
    }

    fn inv(&self) -> Option<Self> {
        Some((self.1.clone(), self.0.clone()))
    }
}

#[macro_export]
macro_rules! impl_mul_fraction {
    ($Uint:ident) => {
        impl $Uint {
            pub fn checked_mul_floored<F: Fraction<T>, T: Into<$Uint>>(
                self,
                rhs: F,
            ) -> Result<Self, CheckedMultiplyFractionError> {
                let divisor = rhs.denominator().into();
                let res = self
                    .full_mul(rhs.numerator().into())
                    .checked_div(divisor.into())?;
                Ok(res.try_into()?)
            }

            pub fn mul_floored<F: Fraction<T>, T: Into<$Uint>>(self, rhs: F) -> Self {
                self.checked_mul_floored(rhs).unwrap()
            }

            pub fn checked_mul_ceil<F: Fraction<T> + Clone, T: Into<$Uint>>(
                self,
                rhs: F,
            ) -> Result<Self, CheckedMultiplyFractionError> {
                let floor_result = self.checked_mul_floored(rhs.clone())?;
                let numerator = rhs.numerator().into();
                let denominator = rhs.denominator().into();
                if !numerator.checked_rem(denominator)?.is_zero() {
                    Ok($Uint::one().checked_add(floor_result)?)
                } else {
                    Ok(floor_result)
                }
            }

            pub fn mul_ceil<F: Fraction<T> + Clone, T: Into<$Uint>>(self, rhs: F) -> Self {
                self.checked_mul_ceil(rhs).unwrap()
            }
        }
    };
}
