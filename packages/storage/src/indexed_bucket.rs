// this module requires iterator to be useful at all
#![cfg(feature = "iterator")]

use cosmwasm_std::{from_slice, to_vec, Binary, Order, StdError, StdResult, Storage, KV};
use serde::de::DeserializeOwned;
// use serde::Serialize;
use serde::{Deserialize, Serialize};

use crate::namespace_helpers::{
    get_with_prefix, range_with_prefix, remove_with_prefix, set_with_prefix,
};
use crate::type_helpers::{deserialize_kv, may_deserialize, must_deserialize};
use crate::{to_length_prefixed, to_length_prefixed_nested};
use serde::export::PhantomData;

/// reserved name, no index may register
const PREFIX_PK: &[u8] = b"_pk";
/// MARKER is stored in the multi-index as value, but we only look at the key (which is pk)
const MARKER: &[u8] = b"1";

/// IndexedBucket works like a bucket but has a secondary index
/// This is a WIP.
/// Step 1 - allow exactly 1 secondary index, no multi-prefix on primary key
/// Step 2 - allow multiple named secondary indexes, no multi-prefix on primary key
/// Step 3 - allow unique indexes - They store {pk: Vec<u8>, value: T} so we don't need to re-lookup
/// Step 4 - allow multiple named secondary indexes, clean composite key support
///
/// Current Status: 2
pub struct IndexedBucket<'a, 'b, 'x, S, T>
where
    S: Storage + 'x,
    T: Serialize + DeserializeOwned + Clone + 'x,
{
    core: Core<'a, 'b, S, T>,
    indexes: Vec<Box<dyn Index<S, T> + 'x>>,
}

// 2 main variants:
//  * store (namespace, index_name, idx_value, key) -> b"1" - allows many and references pk
//  * store (namespace, index_name, idx_value) -> {key, value} - allows one and copies pk and data
//  // this would be the primary key - we abstract that too???
//  * store (namespace, index_name, pk) -> value - allows one with data
pub trait Index<S, T>
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

/// we pull out Core from IndexedBucket, so we can get a mutable reference to storage
/// while holding an immutable iterator over indexers
pub struct Core<'a, 'b, S, T>
where
    S: Storage,
    T: Serialize + DeserializeOwned + Clone,
{
    storage: &'a mut S,
    namespace: &'b [u8],
    _phantom: PhantomData<T>,
}

impl<'a, 'b, S, T> Core<'a, 'b, S, T>
where
    S: Storage,
    T: Serialize + DeserializeOwned + Clone,
{
    pub fn set_pk(&mut self, key: &[u8], updated: &T) -> StdResult<()> {
        set_with_prefix(self.storage, &self.prefix_pk(), key, &to_vec(updated)?);
        Ok(())
    }

    pub fn remove_pk(&mut self, key: &[u8]) {
        remove_with_prefix(self.storage, &self.prefix_pk(), key)
    }

    /// load will return an error if no data is set at the given key, or on parse error
    pub fn load(&self, key: &[u8]) -> StdResult<T> {
        let value = get_with_prefix(self.storage, &self.prefix_pk(), key);
        must_deserialize(&value)
    }

    /// may_load will parse the data stored at the key if present, returns Ok(None) if no data there.
    /// returns an error on issues parsing
    pub fn may_load(&self, key: &[u8]) -> StdResult<Option<T>> {
        let value = get_with_prefix(self.storage, &self.prefix_pk(), key);
        may_deserialize(&value)
    }

    fn index_space(&self, index_name: &str, idx: &[u8]) -> Vec<u8> {
        let mut index_space = self.prefix_idx(index_name);
        let mut key_prefix = to_length_prefixed(idx);
        index_space.append(&mut key_prefix);
        index_space
    }

    fn prefix_idx(&self, index_name: &str) -> Vec<u8> {
        to_length_prefixed_nested(&[self.namespace, index_name.as_bytes()])
    }

    fn prefix_pk(&self) -> Vec<u8> {
        to_length_prefixed_nested(&[self.namespace, PREFIX_PK])
    }

    /// iterates over the items in pk order
    pub fn range<'c>(
        &'c self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = StdResult<KV<T>>> + 'c> {
        let mapped = range_with_prefix(self.storage, &self.prefix_pk(), start, end, order)
            .map(deserialize_kv::<T>);
        Box::new(mapped)
    }
}

impl<'a, 'b, 'x, S, T> IndexedBucket<'a, 'b, 'x, S, T>
where
    S: Storage,
    T: Serialize + DeserializeOwned + Clone,
{
    pub fn new(storage: &'a mut S, namespace: &'b [u8]) -> Self {
        IndexedBucket {
            core: Core {
                storage,
                namespace,
                _phantom: Default::default(),
            },
            indexes: vec![],
        }
    }

    /// Usage:
    /// let mut bucket = IndexedBucket::new(&mut storeage, b"foobar")
    ///                     .with_unique_index("name", |x| x.name.clone())?
    ///                     .with_index("age", by_age)?;
    pub fn with_index<U: Into<String>>(
        mut self,
        name: U,
        indexer: fn(&T) -> Vec<u8>,
    ) -> StdResult<Self> {
        let name = name.into();
        self.can_add_index(&name)?;

        let index = MultiIndex::new(indexer, name);
        self.indexes.push(Box::new(index));
        Ok(self)
    }

    /// Usage:
    /// let mut bucket = IndexedBucket::new(&mut storeage, b"foobar")
    ///                     .with_unique_index("name", |x| x.name.clone())?
    ///                     .with_index("age", by_age)?;
    pub fn with_unique_index<U: Into<String>>(
        mut self,
        name: U,
        indexer: fn(&T) -> Vec<u8>,
    ) -> StdResult<Self> {
        let name = name.into();
        self.can_add_index(&name)?;

        let index = UniqueIndex::new(indexer, name);
        self.indexes.push(Box::new(index));
        Ok(self)
    }

    fn can_add_index(&self, name: &str) -> StdResult<()> {
        if name.as_bytes() == PREFIX_PK {
            return Err(StdError::generic_err(
                "Index _pk is reserved for the primary key",
            ));
        }
        let dup = self.get_index(name);
        match dup {
            Some(_) => Err(StdError::generic_err(format!(
                "Attempt to write index {} 2 times",
                name
            ))),
            None => Ok(()),
        }
    }

    fn get_index(&self, name: &str) -> Option<&Box<dyn Index<S, T> + 'x>> {
        for existing in self.indexes.iter() {
            if existing.name() == name {
                return Some(existing);
            }
        }
        None
    }

    /// save will serialize the model and store, returns an error on serialization issues.
    /// this must load the old value to update the indexes properly
    /// if you loaded the old value earlier in the same function, use replace to avoid needless db reads
    pub fn save(&mut self, key: &[u8], data: &T) -> StdResult<()> {
        let old_data = self.may_load(key)?;
        self.replace(key, Some(data), old_data.as_ref())
    }

    pub fn remove(&mut self, key: &[u8]) -> StdResult<()> {
        let old_data = self.may_load(key)?;
        self.replace(key, None, old_data.as_ref())
    }

    /// replace writes data to key. old_data must be the current stored value (from a previous load)
    /// and is used to properly update the index. This is used by save, replace, and update
    /// and can be called directly if you want to optimize
    pub fn replace(&mut self, key: &[u8], data: Option<&T>, old_data: Option<&T>) -> StdResult<()> {
        if let Some(old) = old_data {
            // Note: this didn't work as we cannot mutably borrow self (remove_from_index) inside the iterator
            for index in self.indexes.iter() {
                index.remove(&mut self.core, key, old)?;
            }
        }
        if let Some(updated) = data {
            for index in self.indexes.iter() {
                index.insert(&mut self.core, key, updated)?;
            }
            self.core.set_pk(key, updated)?;
        } else {
            self.core.remove_pk(key);
        }
        Ok(())
    }

    /// Loads the data, perform the specified action, and store the result
    /// in the database. This is shorthand for some common sequences, which may be useful.
    ///
    /// If the data exists, `action(Some(value))` is called. Otherwise `action(None)` is called.
    pub fn update<A, E>(&mut self, key: &[u8], action: A) -> Result<T, E>
    where
        A: FnOnce(Option<T>) -> Result<T, E>,
        E: From<StdError>,
    {
        let input = self.may_load(key)?;
        let old_val = input.clone();
        let output = action(input)?;
        self.replace(key, Some(&output), old_val.as_ref())?;
        Ok(output)
    }

    // Everything else, that doesn't touch indexers, is just pass-through from self.core,
    // thus can be used from while iterating over indexes

    /// load will return an error if no data is set at the given key, or on parse error
    pub fn load(&self, key: &[u8]) -> StdResult<T> {
        self.core.load(key)
    }

    /// may_load will parse the data stored at the key if present, returns Ok(None) if no data there.
    /// returns an error on issues parsing
    pub fn may_load(&self, key: &[u8]) -> StdResult<Option<T>> {
        self.core.may_load(key)
    }

    /// iterates over the items in pk order
    pub fn range<'c>(
        &'c self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = StdResult<KV<T>>> + 'c> {
        self.core.range(start, end, order)
    }

    /// returns all pks that where stored under this secondary index, always Ascending
    /// this is mainly an internal function, but can be used direcly if you just want to list ids cheaply
    pub fn pks_by_index<'c>(
        &'c self,
        index_name: &str,
        idx: &[u8],
    ) -> StdResult<Box<dyn Iterator<Item = Vec<u8>> + 'c>> {
        let index = self
            .get_index(index_name)
            .ok_or_else(|| StdError::not_found(index_name))?;
        Ok(index.pks_by_index(&self.core, idx))
    }

    /// returns all items that match this secondary index, always by pk Ascending
    pub fn items_by_index<'c>(
        &'c self,
        index_name: &str,
        idx: &[u8],
    ) -> StdResult<Box<dyn Iterator<Item = StdResult<KV<T>>> + 'c>> {
        let index = self
            .get_index(index_name)
            .ok_or_else(|| StdError::not_found(index_name))?;
        Ok(index.items_by_index(&self.core, idx))
    }
}

//------- indexers ------//

pub struct MultiIndex<S, T>
where
    S: Storage,
    T: Serialize + DeserializeOwned + Clone,
{
    idx_fn: fn(&T) -> Vec<u8>,
    _name: String,
    _phantom: PhantomData<S>,
}

impl<S, T> MultiIndex<S, T>
where
    S: Storage,
    T: Serialize + DeserializeOwned + Clone,
{
    pub fn new<U: Into<String>>(idx_fn: fn(&T) -> Vec<u8>, name: U) -> Self {
        MultiIndex {
            idx_fn,
            _name: name.into(),
            _phantom: Default::default(),
        }
    }
}

impl<S, T> Index<S, T> for MultiIndex<S, T>
where
    S: Storage,
    T: Serialize + DeserializeOwned + Clone,
{
    fn name(&self) -> String {
        self._name.clone()
    }

    fn index(&self, data: &T) -> Vec<u8> {
        (self.idx_fn)(data)
    }

    fn insert(&self, core: &mut Core<S, T>, pk: &[u8], data: &T) -> StdResult<()> {
        let idx = self.index(data);
        set_with_prefix(
            core.storage,
            &core.index_space(&self._name, &idx),
            pk,
            MARKER,
        );
        Ok(())
    }

    fn remove(&self, core: &mut Core<S, T>, pk: &[u8], old_data: &T) -> StdResult<()> {
        let idx = self.index(old_data);
        remove_with_prefix(core.storage, &core.index_space(&self._name, &idx), pk);
        Ok(())
    }

    fn pks_by_index<'c>(
        &self,
        core: &'c Core<S, T>,
        idx: &[u8],
    ) -> Box<dyn Iterator<Item = Vec<u8>> + 'c> {
        let start = core.index_space(&self._name, idx);
        let mapped =
            range_with_prefix(core.storage, &start, None, None, Order::Ascending).map(|(k, _)| k);
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

pub struct UniqueIndex<S, T>
where
    S: Storage,
    T: Serialize + DeserializeOwned + Clone,
{
    idx_fn: fn(&T) -> Vec<u8>,
    _name: String,
    _phantom: PhantomData<S>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct UniqueRef<T: Clone> {
    pk: Binary,
    value: T,
}

impl<S, T> UniqueIndex<S, T>
where
    S: Storage,
    T: Serialize + DeserializeOwned + Clone,
{
    pub fn new<U: Into<String>>(idx_fn: fn(&T) -> Vec<u8>, name: U) -> Self {
        UniqueIndex {
            idx_fn,
            _name: name.into(),
            _phantom: Default::default(),
        }
    }
}

impl<S, T> Index<S, T> for UniqueIndex<S, T>
where
    S: Storage,
    T: Serialize + DeserializeOwned + Clone,
{
    fn name(&self) -> String {
        self._name.clone()
    }

    fn index(&self, data: &T) -> Vec<u8> {
        (self.idx_fn)(data)
    }

    // we store (namespace, index_name, idx_value) -> { pk, value }
    fn insert(&self, core: &mut Core<S, T>, pk: &[u8], data: &T) -> StdResult<()> {
        let idx = self.index(data);
        let reference = UniqueRef::<T> {
            pk: pk.into(),
            value: data.clone(),
        };
        set_with_prefix(
            core.storage,
            &core.prefix_idx(&self._name),
            &idx,
            &to_vec(&reference)?,
        );
        Ok(())
    }

    // we store (namespace, index_name, idx_value) -> { pk, value }
    fn remove(&self, core: &mut Core<S, T>, _pk: &[u8], old_data: &T) -> StdResult<()> {
        let idx = self.index(old_data);
        remove_with_prefix(core.storage, &core.prefix_idx(&self._name), &idx);
        Ok(())
    }

    // there is exactly 0 or 1 here...
    fn pks_by_index<'c>(
        &self,
        core: &'c Core<S, T>,
        idx: &[u8],
    ) -> Box<dyn Iterator<Item = Vec<u8>> + 'c> {
        let data = get_with_prefix(core.storage, &core.prefix_idx(&self._name), &idx);
        let res = match data {
            Some(bin) => vec![bin],
            None => vec![],
        };
        let mapped = res.into_iter().map(|bin| {
            // TODO: update types so we can return error here
            let parsed: UniqueRef<T> = from_slice(&bin).unwrap();
            parsed.pk.into_vec()
        });
        Box::new(mapped)
    }

    /// returns all items that match this secondary index, always by pk Ascending
    fn items_by_index<'c>(
        &self,
        core: &'c Core<S, T>,
        idx: &[u8],
    ) -> Box<dyn Iterator<Item = StdResult<KV<T>>> + 'c> {
        let data = get_with_prefix(core.storage, &core.prefix_idx(&self._name), &idx);
        let res = match data {
            Some(bin) => vec![bin],
            None => vec![],
        };
        let mapped = res.into_iter().map(|bin| {
            let parsed: UniqueRef<T> = from_slice(&bin)?;
            Ok((parsed.pk.into_vec(), parsed.value))
        });
        Box::new(mapped)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use cosmwasm_std::testing::MockStorage;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Data {
        pub name: String,
        pub age: i32,
    }

    fn by_name(data: &Data) -> Vec<u8> {
        data.name.as_bytes().to_vec()
    }

    fn by_age(data: &Data) -> Vec<u8> {
        data.age.to_be_bytes().into()
    }

    #[test]
    fn store_and_load_by_index() {
        let mut store = MockStorage::new();
        let mut bucket = IndexedBucket::new(&mut store, b"data")
            .with_index("name", by_name)
            .unwrap()
            .with_unique_index("age", by_age)
            .unwrap();

        // save data
        let data = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        let pk: &[u8] = b"5627";
        bucket.save(pk, &data).unwrap();

        // load it properly
        let loaded = bucket.load(pk).unwrap();
        assert_eq!(data, loaded);

        // load it by secondary index (we must know how to compute this)
        let marias: StdResult<Vec<_>> = bucket.items_by_index("name", b"Maria").unwrap().collect();
        let marias = marias.unwrap();
        assert_eq!(1, marias.len());
        let (k, v) = &marias[0];
        assert_eq!(pk, k.as_slice());
        assert_eq!(&data, v);

        // other index doesn't match (1 byte after)
        let marias: StdResult<Vec<_>> = bucket.items_by_index("name", b"Marib").unwrap().collect();
        assert_eq!(0, marias.unwrap().len());

        // other index doesn't match (1 byte before)
        let marias: StdResult<Vec<_>> = bucket.items_by_index("name", b"Mari`").unwrap().collect();
        assert_eq!(0, marias.unwrap().len());

        // other index doesn't match (longer)
        let marias: StdResult<Vec<_>> = bucket.items_by_index("name", b"Maria5").unwrap().collect();
        assert_eq!(0, marias.unwrap().len());

        // match on proper age
        let proper = 42u32.to_be_bytes();
        let marias: StdResult<Vec<_>> = bucket.items_by_index("age", &proper).unwrap().collect();
        let marias = marias.unwrap();
        assert_eq!(1, marias.len());

        // no match on wrong age
        let too_old = 43u32.to_be_bytes();
        let marias: StdResult<Vec<_>> = bucket.items_by_index("age", &too_old).unwrap().collect();
        assert_eq!(0, marias.unwrap().len());
    }
}
