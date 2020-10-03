// this module requires iterator to be useful at all
#![cfg(feature = "iterator")]

use cosmwasm_std::{from_slice, to_vec, Binary, Order, StdResult, Storage, KV};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use crate::indexed_bucket::Core;
use crate::length_prefixed::{namespaces_with_key, to_length_prefixed_nested};
use crate::namespace_helpers::range_with_prefix;

/// MARKER is stored in the multi-index as value, but we only look at the key (which is pk)
const MARKER: &[u8] = b"1";

// 2 main variants:
//  * store (namespace, index_name, idx_value, key) -> b"1" - allows many and references pk
//  * store (namespace, index_name, idx_value) -> {key, value} - allows one and copies pk and data
//  // this would be the primary key - we abstract that too???
//  * store (namespace, index_name, pk) -> value - allows one with data
pub(crate) trait Index<S, T>
where
    S: Storage,
    T: Serialize + DeserializeOwned + Clone,
{
    // TODO: do we make this any Vec<u8> ?
    fn name(&self) -> String;
    fn index(&self, data: &T) -> Vec<u8>;

    fn insert(&self, core: &mut Core<S, T>, pk: &[u8], data: &T) -> StdResult<()>;
    fn remove(&self, core: &mut Core<S, T>, pk: &[u8], old_data: &T) -> StdResult<()>;

    // these should be implemented by all
    fn pks_by_index<'c>(
        &self,
        core: &'c Core<S, T>,
        idx: &[u8],
    ) -> Box<dyn Iterator<Item = Vec<u8>> + 'c>;

    /// returns all items that match this secondary index, always by pk Ascending
    fn items_by_index<'c>(
        &self,
        core: &'c Core<S, T>,
        idx: &[u8],
    ) -> Box<dyn Iterator<Item = StdResult<KV<T>>> + 'c>;

    // TODO: range over secondary index values? (eg. all results with 30 < age < 40)
}

pub(crate) struct MultiIndex<'a, S, T>
where
    S: Storage,
    T: Serialize + DeserializeOwned + Clone,
{
    idx_fn: fn(&T) -> Vec<u8>,
    _name: &'a str,
    _phantom: PhantomData<S>,
}

impl<'a, S, T> MultiIndex<'a, S, T>
where
    S: Storage,
    T: Serialize + DeserializeOwned + Clone,
{
    pub fn new(idx_fn: fn(&T) -> Vec<u8>, name: &'a str) -> Self {
        MultiIndex {
            idx_fn,
            _name: name,
            _phantom: Default::default(),
        }
    }
}

impl<'a, S, T> Index<S, T> for MultiIndex<'a, S, T>
where
    S: Storage,
    T: Serialize + DeserializeOwned + Clone,
{
    fn name(&self) -> String {
        self._name.to_string()
    }

    fn index(&self, data: &T) -> Vec<u8> {
        (self.idx_fn)(data)
    }

    fn insert(&self, core: &mut Core<S, T>, pk: &[u8], data: &T) -> StdResult<()> {
        let idx = self.index(data);
        let key = namespaces_with_key(&[core.namespace, self._name.as_bytes(), &idx], pk);
        core.storage.set(&key, MARKER);
        Ok(())
    }

    fn remove(&self, core: &mut Core<S, T>, pk: &[u8], old_data: &T) -> StdResult<()> {
        let idx = self.index(old_data);
        let key = namespaces_with_key(&[core.namespace, self._name.as_bytes(), &idx], pk);
        core.storage.remove(&key);
        Ok(())
    }

    fn pks_by_index<'c>(
        &self,
        core: &'c Core<S, T>,
        idx: &[u8],
    ) -> Box<dyn Iterator<Item = Vec<u8>> + 'c> {
        let namespace = to_length_prefixed_nested(&[core.namespace, self._name.as_bytes(), idx]);
        let mapped = range_with_prefix(core.storage, &namespace, None, None, Order::Ascending)
            .map(|(k, _)| k);
        Box::new(mapped)
    }

    /// returns all items that match this secondary index, always by pk Ascending
    fn items_by_index<'c>(
        &self,
        core: &'c Core<S, T>,
        idx: &[u8],
    ) -> Box<dyn Iterator<Item = StdResult<KV<T>>> + 'c> {
        let mapped = self.pks_by_index(core, idx).map(move |pk| {
            let v = core.load(&pk)?;
            Ok((pk, v))
        });
        Box::new(mapped)
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct UniqueRef<T: Clone> {
    pk: Binary,
    value: T,
}

pub(crate) struct UniqueIndex<'a, S, T>
where
    S: Storage,
    T: Serialize + DeserializeOwned + Clone,
{
    idx_fn: fn(&T) -> Vec<u8>,
    _name: &'a str,
    _phantom: PhantomData<S>,
}

impl<'a, S, T> UniqueIndex<'a, S, T>
where
    S: Storage,
    T: Serialize + DeserializeOwned + Clone,
{
    pub fn new(idx_fn: fn(&T) -> Vec<u8>, name: &'a str) -> Self {
        UniqueIndex {
            idx_fn,
            _name: name,
            _phantom: Default::default(),
        }
    }
}

impl<'a, S, T> Index<S, T> for UniqueIndex<'a, S, T>
where
    S: Storage,
    T: Serialize + DeserializeOwned + Clone,
{
    fn name(&self) -> String {
        self._name.to_string()
    }

    fn index(&self, data: &T) -> Vec<u8> {
        (self.idx_fn)(data)
    }

    // we store (namespace, index_name, idx_value) -> { pk, value }
    fn insert(&self, core: &mut Core<S, T>, pk: &[u8], data: &T) -> StdResult<()> {
        let idx = self.index(data);
        let key = namespaces_with_key(&[core.namespace, self._name.as_bytes()], &idx);
        let reference = UniqueRef::<T> {
            pk: pk.into(),
            value: data.clone(),
        };
        core.storage.set(&key, &to_vec(&reference)?);
        Ok(())
    }

    // we store (namespace, index_name, idx_value) -> { pk, value }
    fn remove(&self, core: &mut Core<S, T>, _pk: &[u8], old_data: &T) -> StdResult<()> {
        let idx = self.index(old_data);
        let key = namespaces_with_key(&[core.namespace, self._name.as_bytes()], &idx);
        core.storage.remove(&key);
        Ok(())
    }

    // there is exactly 0 or 1 here...
    fn pks_by_index<'c>(
        &self,
        core: &'c Core<S, T>,
        idx: &[u8],
    ) -> Box<dyn Iterator<Item = Vec<u8>> + 'c> {
        // TODO: update types to return StdResult<Vec<u8>> ?
        // should never really happen, but I dislike unwrap
        let mapped = self.items_by_index(core, idx).map(|res| res.unwrap().0);
        Box::new(mapped)
    }

    /// returns all items that match this secondary index, always by pk Ascending
    fn items_by_index<'c>(
        &self,
        core: &'c Core<S, T>,
        idx: &[u8],
    ) -> Box<dyn Iterator<Item = StdResult<KV<T>>> + 'c> {
        let key = namespaces_with_key(&[core.namespace, self._name.as_bytes()], idx);
        let data = match core.storage.get(&key) {
            Some(bin) => vec![bin],
            None => vec![],
        };
        let mapped = data.into_iter().map(|bin| {
            let parsed: UniqueRef<T> = from_slice(&bin)?;
            Ok((parsed.pk.into_vec(), parsed.value))
        });
        Box::new(mapped)
    }
}
