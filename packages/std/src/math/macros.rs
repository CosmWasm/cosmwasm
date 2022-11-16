#![macro_use]

macro_rules! impl_mul_arithmetic {
    ($ty:ty) => {
        impl $ty {
            pub fn mul_ceiled<Rhs: Mul<Self>>(self, rhs: impl crate::Fraction<Rhs>) -> Self {
                todo!()
            }

            pub fn mul_floored<Rhs: Mul<Self>>(self, rhs: impl crate::Fraction<Rhs>) -> Self {
                todo!()
            }

            pub fn checked_mul_ceiled<Rhs: Mul<Self>>(
                self,
                rhs: impl crate::Fraction<Rhs>,
            ) -> Result<Self, crate::errors::CheckedMultiplyCeiledError> {
                todo!()
            }

            pub fn checked_mul_floored(
                self,
                rhs: impl crate::Fraction<Uint128>,
            ) -> Result<Self, crate::errors::CheckedMultiplyFlooredError> {
                let numerator = rhs.numerator();
                let denominator = rhs.denominator();
                if denominator.u128() == 0 {
                    return Err(crate::errors::CheckedMultiplyFlooredError::DivideByZero);
                }
                match (self.full_mul(numerator) / Uint256::from(denominator)).try_into() {
                    Ok(ratio) => Ok(ratio),
                    Err(_) => Err(crate::errors::CheckedMultiplyFlooredError::Overflow),
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{Decimal, Uint128};

    #[test]
    fn foo() {
        let lhs = Uint128::new(5);
        let rhs = Decimal::percent(3);

        assert_eq!(lhs.checked_mul_floored(rhs).unwrap(), Uint128::from(1u128));
    }
}
