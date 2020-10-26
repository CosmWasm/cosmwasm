use crate::traits::{Api, QuerierTrait, Storage};
use crate::Querier;

/// Holds all external dependencies of the contract.
/// Designed to allow easy dependency injection at runtime.
/// This cannot be copied or cloned since it would behave differently
/// for mock storages and a bridge storage in the VM.
pub struct OwnedDeps<S: Storage, A: Api, Q: QuerierTrait> {
    pub storage: S,
    pub api: A,
    pub querier: Q,
}

pub struct Deps<'a> {
    pub storage: &'a mut dyn Storage,
    pub api: &'a dyn Api,
    pub querier: Querier<'a>,
}

#[derive(Copy, Clone)]
pub struct DepsRef<'a> {
    pub storage: &'a dyn Storage,
    pub api: &'a dyn Api,
    pub querier: Querier<'a>,
}

impl<S: Storage, A: Api, Q: QuerierTrait> OwnedDeps<S, A, Q> {
    pub fn as_ref(&'_ self) -> DepsRef<'_> {
        DepsRef {
            storage: &self.storage,
            api: &self.api,
            querier: Querier::new(&self.querier),
        }
    }

    pub fn as_mut(&'_ mut self) -> Deps<'_> {
        Deps {
            storage: &mut self.storage,
            api: &self.api,
            querier: Querier::new(&self.querier),
        }
    }
}

impl<'a> Deps<'a> {
    pub fn as_ref(&'_ self) -> DepsRef<'_> {
        DepsRef {
            storage: self.storage,
            api: self.api,
            querier: self.querier,
        }
    }
}
