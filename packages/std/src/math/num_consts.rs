/// Crate internal trait for all our signed and unsigned number types
#[allow(dead_code)] // only used in tests for now
pub(crate) trait NumConsts {
    const MAX: Self;
    const MIN: Self;
    const ZERO: Self;
    const ONE: Self;
}
