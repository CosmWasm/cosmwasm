use crate::traits::{Api, Querier, Storage};

/// Holds all external dependencies of the contract.
/// Designed to allow easy dependency injection at runtime.
/// This cannot be copied or cloned since it would behave differently
/// for mock storages and a bridge storage in the VM.
pub struct Deps<S: Storage, A: Api, Q: Querier> {
    pub storage: S,
    pub api: A,
    pub querier: Q,
}

pub struct DepsMut<'a, Q: Querier> {
    pub storage: &'a mut dyn Storage,
    pub api: &'a dyn Api,
    pub querier: &'a Q,
}

pub struct DepsRef<'a, Q: Querier> {
    pub storage: &'a dyn Storage,
    pub api: &'a dyn Api,
    // querier trait includes a lot of generic functions
    // eg. fn custom_query<C: CustomQuery, U: DeserializeOwned>
    // TODO: we must move those elsewhere before we can remove this parameter
    pub querier: &'a Q,
}

impl<S: Storage, A: Api, Q: Querier> Deps<S, A, Q> {
    pub fn as_ref(&'_ self) -> DepsRef<'_, Q> {
        DepsRef {
            storage: &self.storage,
            api: &self.api,
            querier: &self.querier,
        }
    }

    pub fn as_mut(&'_ mut self) -> DepsMut<'_, Q> {
        DepsMut {
            storage: &mut self.storage,
            api: &self.api,
            querier: &self.querier,
        }
    }
}

// impl<'a> DepsMut<'a> {
//     pub fn as_ref(&'_ self) -> DepsRef<'_> {
//         DepsRef {
//             storage: &self.storage,
//             api: self.api,
//             querier: self.querier,
//         }
//     }
// }
