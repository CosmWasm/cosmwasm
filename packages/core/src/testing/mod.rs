mod assertions;

pub use assertions::assert_approx_eq_impl;
#[cfg(any(test, feature = "testing"))]
pub use assertions::assert_hash_works_impl;
