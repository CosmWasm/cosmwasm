use serde::{de::DeserializeOwned, ser::Serialize};
use std::marker::PhantomData;

use cosmwasm_std::{to_vec, ReadonlyStorage, StdError, StdResult, Storage};
#[cfg(feature = "iterator")]
use cosmwasm_std::{Order, KV};

use crate::length_prefixed::{
    decode_length, length_prefixed_with_key, namespaces_with_key, nested_namespaces_with_key,
};
#[cfg(feature = "iterator")]
use crate::namespace_helpers::range_with_prefix;
#[cfg(feature = "iterator")]
use crate::type_helpers::deserialize_kv;
use crate::type_helpers::{may_deserialize, must_deserialize};

pub(crate) const PREFIX_PK: &[u8] = b"_pk";

/// An alias of Bucket::new for less verbose usage
pub fn bucket<'a, 'b, S, T>(storage: &'a mut S, namespace: &'b [u8]) -> Bucket<'a, 'b, S, T>
where
    S: Storage,
    T: Serialize + DeserializeOwned,
{
    Bucket::new(storage, namespace)
}

/// An alias of ReadonlyBucket::new for less verbose usage
pub fn bucket_read<'a, 'b, S, T>(
    storage: &'a S,
    namespace: &'b [u8],
) -> ReadonlyBucket<'a, 'b, S, T>
where
    S: ReadonlyStorage,
    T: Serialize + DeserializeOwned,
{
    ReadonlyBucket::new(storage, namespace)
}

/// Bucket stores all data under a series of length-prefixed steps: (namespace, "_pk").
/// After this is created (each step length-prefixed), we just append the bucket key at the end.
///
/// The reason for the "_pk" at the end is to allow easy extensibility with IndexedBuckets, which
/// can also store indexes under the same namespace
pub struct Bucket<'a, 'b, S, T>
where
    S: Storage,
    T: Serialize + DeserializeOwned,
{
    pub(crate) storage: &'a mut S,
    pub namespace: &'b [u8],
    // see https://doc.rust-lang.org/std/marker/struct.PhantomData.html#unused-type-parameters for why this is needed
    data: PhantomData<T>,
}

impl<'a, 'b, S, T> Bucket<'a, 'b, S, T>
where
    S: Storage,
    T: Serialize + DeserializeOwned,
{
    /// After some reflection, I removed multilevel, as what I really wanted was a composite primary key.
    /// For eg. allowances, the multilevel design would make it ("bucket", owner, "_pk", spender)
    /// and then maybe ("bucket", owner, "expires", expires_idx, pk) as a secondary index. This is NOT what we want.
    ///
    /// What we want is ("bucket", "_pk", owner, spender) for the first key. And
    /// ("bucket", "expires", expires_idx, pk) as the secondary index. This is not done with "multi-level"
    /// but by supporting CompositeKeys. Looking something like:
    ///
    /// `Bucket::new(storage, "bucket).save(&(owner, spender).pk(), &allowance)`
    ///
    /// Need to figure out the most ergonomic approach for the composite keys, but it should
    ///  live outside the Bucket, and just convert into a normal `&[u8]` for this array.
    pub fn new(storage: &'a mut S, namespace: &'b [u8]) -> Self {
        Bucket {
            storage,
            namespace,
            data: PhantomData,
        }
    }

    /// This provides the raw storage key that we use to access a given "bucket key".
    /// Calling this with `key = b""` will give us the pk prefix for range queries
    pub fn build_primary_key(&self, key: &[u8]) -> Vec<u8> {
        namespaces_with_key(&[&self.namespace, PREFIX_PK], key)
    }

    /// This provides the raw storage key that we use to access a secondary index
    /// Calling this with `key = b""` will give us the index prefix for range queries
    pub fn build_secondary_key(&self, path: &[&[u8]], key: &[u8]) -> Vec<u8> {
        nested_namespaces_with_key(&[self.namespace], path, key)
    }

    /// save will serialize the model and store, returns an error on serialization issues
    pub fn save(&mut self, key: &[u8], data: &T) -> StdResult<()> {
        let key = self.build_primary_key(key);
        self.storage.set(&key, &to_vec(data)?);
        Ok(())
    }

    pub fn remove(&mut self, key: &[u8]) {
        let key = self.build_primary_key(key);
        self.storage.remove(&key);
    }

    /// load will return an error if no data is set at the given key, or on parse error
    pub fn load(&self, key: &[u8]) -> StdResult<T> {
        let key = self.build_primary_key(key);
        let value = self.storage.get(&key);
        must_deserialize(&value)
    }

    /// may_load will parse the data stored at the key if present, returns Ok(None) if no data there.
    /// returns an error on issues parsing
    pub fn may_load(&self, key: &[u8]) -> StdResult<Option<T>> {
        let key = self.build_primary_key(key);
        let value = self.storage.get(&key);
        may_deserialize(&value)
    }

    #[cfg(feature = "iterator")]
    pub fn range<'c>(
        &'c self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = StdResult<KV<T>>> + 'c> {
        let namespace = self.build_primary_key(b"");
        let mapped =
            range_with_prefix(self.storage, &namespace, start, end, order).map(deserialize_kv::<T>);
        Box::new(mapped)
    }

    // TODO: test this, just an idea now, need more work on Pks
    // Also, how much do we trim from the keys? Leave just the last part of the PK, right?

    /// This lets us grab all items under the beginning of a composite key.
    /// If we store under `Pk2(owner, spender)`, then we pass `prefixes: &[owner]` here
    /// To list all spenders under the owner
    #[cfg(feature = "iterator")]
    pub fn range_prefixed<'c>(
        &'c self,
        prefixes: &[&[u8]],
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = StdResult<KV<T>>> + 'c> {
        let namespace = nested_namespaces_with_key(&[&self.namespace, PREFIX_PK], prefixes, b"");
        let mapped =
            range_with_prefix(self.storage, &namespace, start, end, order).map(deserialize_kv::<T>);
        Box::new(mapped)
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
        let output = action(input)?;
        self.save(key, &output)?;
        Ok(output)
    }
}

pub trait PrimaryKey {
    type Output;

    fn pk(&self) -> Vec<u8>;
    fn parse(data: &[u8]) -> Self::Output;

    // convert a PK into an owned variant (Vec<u8> rather than &[u8])
    // this can be done brute force, but please override for a cheaper version
    // FIXME: better name for this function - to_owned() sounded good, but uses Cloned. Other ideas?
    fn to_output(&self) -> Self::Output {
        Self::parse(&self.pk())
    }

    fn from_kv<T>(kv: (Vec<u8>, T)) -> (Self::Output, T)
    where
        Self: std::marker::Sized,
    {
        let (k, v) = kv;
        (Self::parse(&k), v)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pk2<'a>(pub &'a [u8], pub &'a [u8]);

impl<'a> PrimaryKey for Pk2<'a> {
    type Output = (Vec<u8>, Vec<u8>);

    fn pk(&self) -> Vec<u8> {
        length_prefixed_with_key(self.0, self.1)
    }

    fn to_output(&self) -> Self::Output {
        (self.0.to_vec(), self.1.to_vec())
    }

    fn parse(pk: &[u8]) -> Self::Output {
        let l = decode_length(&pk[..2]);
        let first = pk[2..l + 2].to_vec();
        let second = pk[l + 2..].to_vec();
        (first, second)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pk3<'a>(&'a [u8], &'a [u8], &'a [u8]);

impl<'a> PrimaryKey for Pk3<'a> {
    type Output = (Vec<u8>, Vec<u8>, Vec<u8>);

    fn pk(&self) -> Vec<u8> {
        namespaces_with_key(&[self.0, self.1], self.2)
    }

    fn to_output(&self) -> Self::Output {
        (self.0.to_vec(), self.1.to_vec(), self.2.to_vec())
    }

    fn parse(pk: &[u8]) -> Self::Output {
        let l = decode_length(&pk[..2]);
        let l2 = decode_length(&pk[l + 2..l + 4]);
        let first = pk[2..l + 2].to_vec();
        let second = pk[l + 4..l + l2 + 4].to_vec();
        let third = pk[l + l2 + 4..].to_vec();
        (first, second, third)
    }
}

pub struct ReadonlyBucket<'a, 'b, S, T>
where
    S: ReadonlyStorage,
    T: Serialize + DeserializeOwned,
{
    pub(crate) storage: &'a S,
    pub(crate) namespace: &'b [u8],
    // see https://doc.rust-lang.org/std/marker/struct.PhantomData.html#unused-type-parameters for why this is needed
    data: PhantomData<T>,
}

impl<'a, 'b, S, T> ReadonlyBucket<'a, 'b, S, T>
where
    S: ReadonlyStorage,
    T: Serialize + DeserializeOwned,
{
    pub fn new(storage: &'a S, namespace: &'b [u8]) -> Self {
        ReadonlyBucket {
            storage,
            namespace,
            data: PhantomData,
        }
    }

    /// This provides the raw storage key that we use to access a given "bucket key".
    /// Calling this with `key = b""` will give us the pk prefix for range queries
    pub fn build_primary_key(&self, key: &[u8]) -> Vec<u8> {
        namespaces_with_key(&[&self.namespace, PREFIX_PK], key)
    }

    /// This provides the raw storage key that we use to access a secondary index
    /// Calling this with `key = b""` will give us the index prefix for range queries
    pub fn build_secondary_key(&self, path: &[&[u8]], key: &[u8]) -> Vec<u8> {
        nested_namespaces_with_key(&[self.namespace], path, key)
    }

    /// load will return an error if no data is set at the given key, or on parse error
    pub fn load(&self, key: &[u8]) -> StdResult<T> {
        let key = self.build_primary_key(key);
        let value = self.storage.get(&key);
        must_deserialize(&value)
    }

    /// may_load will parse the data stored at the key if present, returns Ok(None) if no data there.
    /// returns an error on issues parsing
    pub fn may_load(&self, key: &[u8]) -> StdResult<Option<T>> {
        let key = self.build_primary_key(key);
        let value = self.storage.get(&key);
        may_deserialize(&value)
    }

    #[cfg(feature = "iterator")]
    pub fn range<'c>(
        &'c self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = StdResult<KV<T>>> + 'c> {
        let namespace = self.build_primary_key(b"");
        let mapped =
            range_with_prefix(self.storage, &namespace, start, end, order).map(deserialize_kv::<T>);
        Box::new(mapped)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::MockStorage;
    use cosmwasm_std::StdError;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Data {
        pub name: String,
        pub age: i32,
    }

    #[test]
    fn store_and_load() {
        let mut store = MockStorage::new();
        let mut bucket = bucket::<_, Data>(&mut store, b"data");

        // save data
        let data = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        bucket.save(b"maria", &data).unwrap();

        // load it properly
        let loaded = bucket.load(b"maria").unwrap();
        assert_eq!(data, loaded);
    }

    #[test]
    fn remove_works() {
        let mut store = MockStorage::new();
        let mut bucket = bucket::<_, Data>(&mut store, b"data");

        // save data
        let data = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        bucket.save(b"maria", &data).unwrap();
        assert_eq!(data, bucket.load(b"maria").unwrap());

        // deleting random key does nothing
        bucket.remove(b"foobar");
        assert_eq!(data, bucket.load(b"maria").unwrap());

        // deleting maria removes the data
        bucket.remove(b"maria");
        assert_eq!(None, bucket.may_load(b"maria").unwrap());
    }

    #[test]
    fn readonly_works() {
        let mut store = MockStorage::new();
        let mut bucket = bucket::<_, Data>(&mut store, b"data");

        // save data
        let data = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        bucket.save(b"maria", &data).unwrap();

        let reader = bucket_read::<_, Data>(&mut store, b"data");

        // check empty data handling
        assert!(reader.load(b"john").is_err());
        assert_eq!(reader.may_load(b"john").unwrap(), None);

        // load it properly
        let loaded = reader.load(b"maria").unwrap();
        assert_eq!(data, loaded);
    }

    #[test]
    fn buckets_isolated() {
        let mut store = MockStorage::new();
        let mut bucket1 = bucket::<_, Data>(&mut store, b"data");

        // save data
        let data = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        bucket1.save(b"maria", &data).unwrap();

        let mut bucket2 = bucket::<_, Data>(&mut store, b"dat");

        // save data (dat, amaria) vs (data, maria)
        let data2 = Data {
            name: "Amen".to_string(),
            age: 67,
        };
        bucket2.save(b"amaria", &data2).unwrap();

        // load one
        let reader = bucket_read::<_, Data>(&store, b"data");
        let loaded = reader.load(b"maria").unwrap();
        assert_eq!(data, loaded);
        // no cross load
        assert_eq!(None, reader.may_load(b"amaria").unwrap());

        // load the other
        let reader2 = bucket_read::<_, Data>(&store, b"dat");
        let loaded2 = reader2.load(b"amaria").unwrap();
        assert_eq!(data2, loaded2);
        // no cross load
        assert_eq!(None, reader2.may_load(b"maria").unwrap());
    }

    #[test]
    fn update_success() {
        let mut store = MockStorage::new();
        let mut bucket = bucket::<_, Data>(&mut store, b"data");

        // initial data
        let init = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        bucket.save(b"maria", &init).unwrap();

        // it's my birthday
        let birthday = |mayd: Option<Data>| -> StdResult<Data> {
            let mut d = mayd.ok_or(StdError::not_found("Data"))?;
            d.age += 1;
            Ok(d)
        };
        let output = bucket.update(b"maria", &birthday).unwrap();
        let expected = Data {
            name: "Maria".to_string(),
            age: 43,
        };
        assert_eq!(output, expected);

        // load it properly
        let loaded = bucket.load(b"maria").unwrap();
        assert_eq!(loaded, expected);
    }

    #[test]
    fn update_can_change_variable_from_outer_scope() {
        let mut store = MockStorage::new();
        let mut bucket = bucket::<_, Data>(&mut store, b"data");
        let init = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        bucket.save(b"maria", &init).unwrap();

        // show we can capture data from the closure
        let mut old_age = 0i32;
        bucket
            .update(b"maria", |mayd: Option<Data>| -> StdResult<_> {
                let mut d = mayd.ok_or(StdError::not_found("Data"))?;
                old_age = d.age;
                d.age += 1;
                Ok(d)
            })
            .unwrap();
        assert_eq!(old_age, 42);
    }

    #[test]
    fn update_fails_on_error() {
        let mut store = MockStorage::new();
        let mut bucket = bucket::<_, Data>(&mut store, b"data");

        // initial data
        let init = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        bucket.save(b"maria", &init).unwrap();

        // it's my birthday
        let output = bucket.update(b"maria", |_d| {
            Err(StdError::generic_err("cuz i feel like it"))
        });
        assert!(output.is_err());

        // load it properly
        let loaded = bucket.load(b"maria").unwrap();
        assert_eq!(loaded, init);
    }

    #[test]
    fn update_supports_custom_error_types() {
        #[derive(Debug)]
        enum MyError {
            Std,
            NotFound,
        }

        impl From<StdError> for MyError {
            fn from(_original: StdError) -> MyError {
                MyError::Std
            }
        }

        let mut store = MockStorage::new();
        let mut bucket = bucket::<_, Data>(&mut store, b"data");

        // initial data
        let init = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        bucket.save(b"maria", &init).unwrap();

        // it's my birthday
        let res = bucket.update(b"bob", |data| {
            if let Some(mut data) = data {
                if data.age < 0 {
                    // Uses Into to convert StdError to MyError
                    return Err(StdError::generic_err("Current age is negative").into());
                }
                if data.age > 10 {
                    to_vec(&data)?; // Uses From to convert StdError to MyError
                }
                data.age += 1;
                Ok(data)
            } else {
                return Err(MyError::NotFound);
            }
        });
        match res.unwrap_err() {
            MyError::NotFound { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn update_handles_on_no_data() {
        let mut store = MockStorage::new();
        let mut bucket = bucket::<_, Data>(&mut store, b"data");

        let init_value = Data {
            name: "Maria".to_string(),
            age: 42,
        };

        // it's my birthday
        let output = bucket
            .update(b"maria", |d| match d {
                Some(_) => Err(StdError::generic_err("Ensure this was empty")),
                None => Ok(init_value.clone()),
            })
            .unwrap();
        assert_eq!(output, init_value);

        // nothing stored
        let loaded = bucket.load(b"maria").unwrap();
        assert_eq!(loaded, init_value);
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn range_over_data() {
        let mut store = MockStorage::new();
        let mut bucket = bucket::<_, Data>(&mut store, b"data");

        let jose = Data {
            name: "Jose".to_string(),
            age: 42,
        };
        let maria = Data {
            name: "Maria".to_string(),
            age: 27,
        };

        bucket.save(b"maria", &maria).unwrap();
        bucket.save(b"jose", &jose).unwrap();

        let res_data: StdResult<Vec<KV<Data>>> =
            bucket.range(None, None, Order::Ascending).collect();
        let data = res_data.unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0], (b"jose".to_vec(), jose.clone()));
        assert_eq!(data[1], (b"maria".to_vec(), maria.clone()));

        // also works for readonly
        let read_bucket = bucket_read::<_, Data>(&store, b"data");
        let res_data: StdResult<Vec<KV<Data>>> =
            read_bucket.range(None, None, Order::Ascending).collect();
        let data = res_data.unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0], (b"jose".to_vec(), jose));
        assert_eq!(data[1], (b"maria".to_vec(), maria));
    }

    #[test]
    fn composite_keys() {
        let composite = Pk2(b"its", b"windy");
        let key = composite.pk();
        assert_eq!(10, key.len());
        let parsed = Pk2::parse(&key);
        assert_eq!(parsed, composite.to_output());

        let composite = Pk3(b"winters", b"really", b"windy");
        let key = composite.pk();
        assert_eq!(22, key.len());
        let parsed = Pk3::parse(&key);
        assert_eq!(parsed, composite.to_output());
    }

    #[test]
    fn composite_keys_parsing() {
        // Try from a KV (as if we got a range)
        let john = Data {
            name: "John".to_string(),
            age: 123,
        };
        let composite = Pk3(b"lots", b"of", b"text");

        // demo usage as if we mapped over a range iterator
        let mut it = vec![(composite.pk(), john.clone())]
            .into_iter()
            .map(Pk3::from_kv);
        let (k1, v1) = it.next().unwrap();
        assert_eq!(k1, composite.to_output());
        assert_eq!(v1, john);
        assert!(it.next().is_none());
    }

    #[test]
    #[cfg(features = "iterator")]
    fn bucket_with_composite_pk() {
        let mut store = MockStorage::new();
        let mut bucket = Bucket::<_, 64>::new(&mut store, b"allowance");

        let owner1: &[u8] = b"john";
        let owner2: &[u8] = b"juan";
        let spender1: &[u8] = b"marco";
        let spender2: &[u8] = b"maria";
        let spender3: &[u8] = b"martian";

        // store some data with composite key
        bucket.save(&Pk2(owner1, spender1).pk(), &100).unwrap();
        bucket.save(&Pk2(owner1, spender2).pk(), &250).unwrap();
        bucket.save(&Pk2(owner2, spender1).pk(), &77).unwrap();
        bucket.save(&Pk2(owner2, spender3).pk(), &444).unwrap();

        // query by full key
        assert_eq!(100, bucket.load(&Pk2(owner1, spender1).pk()).unwrap());
        assert_eq!(444, bucket.load(&Pk2(owner2, spender3).pk()).unwrap());

        // range over one owner. since it is prefixed, we only get the remaining part of the pk (spender)
        let spenders: StdResult<Vec<_>> = bucket
            .range_prefixed(&[owner1], None, None, Order::Ascending)
            .collect();
        let spenders = spenders.unwrap();
        assert_eq!(2, spenders.len());
        assert_eq!(spenders[0], (spender1.to_vec(), 100));
        assert_eq!(spenders[1], (spender2.to_vec(), 250));

        // range over all data. use Pk2::from_kv to parse out the composite key (owner, spender)
        let spenders: StdResult<Vec<_>> = bucket
            .range(None, None, Order::Ascending)
            .map(Pk2::from_kv)
            .collect();
        let spenders = spenders.unwrap();
        assert_eq!(4, spenders.len());
        assert_eq!(spenders[0], (Pk2(owner1, spender1).to_output(), 100));
        assert_eq!(spenders[1], (Pk2(owner1, spender2).to_output(), 250));
        assert_eq!(spenders[2], (Pk2(owner2, spender1).to_output(), 77));
        assert_eq!(spenders[3], (Pk2(owner2, spender3).to_output(), 444));
    }
}
