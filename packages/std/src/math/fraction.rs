use serde::{Deserialize, Serialize};

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
