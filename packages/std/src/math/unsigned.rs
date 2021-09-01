use crate::{Uint128, Uint256, Uint512, Uint64};

/// Marker trait for types that represent unsigned integers.
pub trait Unsigned {}
impl Unsigned for u8 {}
impl Unsigned for u16 {}
impl Unsigned for u32 {}
impl Unsigned for u64 {}
impl Unsigned for u128 {}
impl Unsigned for Uint64 {}
impl Unsigned for Uint128 {}
impl Unsigned for Uint256 {}
impl Unsigned for Uint512 {}
impl Unsigned for usize {}
