/// A fraction `p`/`q` with integers `p` and `q`.
///
/// `p` is called the nominator and `q` is called the denominator.
pub trait Fraction<T> {
    /// Returns the nominator `p`
    fn nominator(&self) -> T;
    /// Returns the denominator `q`
    fn denominator(&self) -> T;
}
