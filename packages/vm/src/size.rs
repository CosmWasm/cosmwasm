use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Size(pub(crate) usize);

impl Size {
    /// Creates a size of `n`
    pub const fn new(n: usize) -> Self {
        Size(n)
    }

    /// Creates a size of `n` kilo
    pub const fn kilo(n: usize) -> Self {
        Size(n * 1000)
    }

    /// Creates a size of `n` kibi
    pub const fn kibi(n: usize) -> Self {
        Size(n * 1024)
    }

    /// Creates a size of `n` mega
    pub const fn mega(n: usize) -> Self {
        Size(n * 1000 * 1000)
    }

    /// Creates a size of `n` mebi
    pub const fn mebi(n: usize) -> Self {
        Size(n * 1024 * 1024)
    }

    /// Creates a size of `n` giga
    pub const fn giga(n: usize) -> Self {
        Size(n * 1000 * 1000 * 1000)
    }

    /// Creates a size of `n` gibi
    pub const fn gibi(n: usize) -> Self {
        Size(n * 1024 * 1024 * 1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_work() {
        assert_eq!(Size::new(0).0, Size(0).0);
        assert_eq!(Size::new(3).0, Size(3).0);

        assert_eq!(Size::kilo(0).0, 0);
        assert_eq!(Size::kilo(3).0, 3000);

        assert_eq!(Size::kibi(0).0, 0);
        assert_eq!(Size::kibi(3).0, 3072);

        assert_eq!(Size::mega(0).0, 0);
        assert_eq!(Size::mega(3).0, 3000000);

        assert_eq!(Size::mebi(0).0, 0);
        assert_eq!(Size::mebi(3).0, 3145728);

        assert_eq!(Size::giga(0).0, 0);
        assert_eq!(Size::giga(3).0, 3000000000);

        assert_eq!(Size::gibi(0).0, 0);
        assert_eq!(Size::gibi(3).0, 3221225472);
    }

    #[test]
    fn implements_debug() {
        assert_eq!(format!("{:?}", Size::new(0)), "Size(0)");
        assert_eq!(format!("{:?}", Size::new(123)), "Size(123)");
        assert_eq!(format!("{:?}", Size::kibi(2)), "Size(2048)");
        assert_eq!(format!("{:?}", Size::mebi(1)), "Size(1048576)");
    }
}
