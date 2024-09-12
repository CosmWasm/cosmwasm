use std::fmt::Debug;

pub struct Hash(pub &'static str);

inventory::collect!(Hash);

#[inline]
pub fn collect_hashes() -> impl Iterator<Item = &'static str> + Debug {
    let mut hashes = inventory::iter::<Hash>
        .into_iter()
        .map(|hash| hash.0)
        .collect::<Vec<_>>();

    hashes.sort();
    hashes.into_iter()
}

#[doc(hidden)]
pub use ::inventory;

pub use ::cosmwasm_vm_derive_impl::*;
