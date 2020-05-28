use serde::{de::DeserializeOwned, ser::Serialize};
use std::marker::PhantomData;

use cosmwasm_std::{to_vec, ReadonlyStorage, StdResult, Storage};
#[cfg(feature = "iterator")]
use cosmwasm_std::{Order, KV};

use crate::length_prefixed::{to_length_prefixed, to_length_prefixed_nested};
#[cfg(feature = "iterator")]
use crate::namespace_helpers::range_with_prefix;
use crate::namespace_helpers::{get_with_prefix, set_with_prefix};
#[cfg(feature = "iterator")]
use crate::type_helpers::deserialize_kv;
use crate::type_helpers::{may_deserialize, must_deserialize};

pub fn bucket<'a, S: Storage, T>(namespace: &[u8], storage: &'a mut S) -> Bucket<'a, S, T>
where
    T: Serialize + DeserializeOwned,
{
    Bucket::new(namespace, storage)
}

pub fn bucket_read<'a, S: ReadonlyStorage, T>(
    namespace: &[u8],
    storage: &'a S,
) -> ReadonlyBucket<'a, S, T>
where
    T: Serialize + DeserializeOwned,
{
    ReadonlyBucket::new(namespace, storage)
}

pub struct Bucket<'a, S: Storage, T>
where
    T: Serialize + DeserializeOwned,
{
    storage: &'a mut S,
    // see https://doc.rust-lang.org/std/marker/struct.PhantomData.html#unused-type-parameters for why this is needed
    data: PhantomData<&'a T>,
    prefix: Vec<u8>,
}

impl<'a, S: Storage, T> Bucket<'a, S, T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn new(namespace: &[u8], storage: &'a mut S) -> Self {
        Bucket {
            prefix: to_length_prefixed(namespace),
            storage,
            data: PhantomData,
        }
    }

    pub fn multilevel(namespaces: &[&[u8]], storage: &'a mut S) -> Self {
        Bucket {
            prefix: to_length_prefixed_nested(namespaces),
            storage,
            data: PhantomData,
        }
    }

    /// save will serialize the model and store, returns an error on serialization issues
    pub fn save(&mut self, key: &[u8], data: &T) -> StdResult<()> {
        set_with_prefix(self.storage, &self.prefix, key, &to_vec(data)?)
    }

    /// load will return an error if no data is set at the given key, or on parse error
    pub fn load(&self, key: &[u8]) -> StdResult<T> {
        let value = get_with_prefix(self.storage, &self.prefix, key);
        must_deserialize(&value)
    }

    /// may_load will parse the data stored at the key if present, returns Ok(None) if no data there.
    /// returns an error on issues parsing
    pub fn may_load(&self, key: &[u8]) -> StdResult<Option<T>> {
        let value = get_with_prefix(self.storage, &self.prefix, key);
        may_deserialize(&value)
    }

    #[cfg(feature = "iterator")]
    pub fn range<'b>(
        &'b self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> StdResult<Box<dyn Iterator<Item = StdResult<KV<T>>> + 'b>> {
        let mapped = range_with_prefix(self.storage, &self.prefix, start, end, order)?
            .map(deserialize_kv::<T>);
        Ok(Box::new(mapped))
    }

    /// update will load the data, perform the specified action, and store the result
    /// in the database. This is shorthand for some common sequences, which may be useful.
    /// Note that this only updates *pre-existing* values. If you want to modify possibly
    /// non-existent values, please use `may_update`
    ///
    /// This is the least stable of the APIs, and definitely needs some usage
    pub fn update<A>(&mut self, key: &[u8], action: A) -> StdResult<T>
    where
        A: FnOnce(Option<T>) -> StdResult<T>,
    {
        let input = self.may_load(key)?;
        let output = action(input)?;
        self.save(key, &output)?;
        Ok(output)
    }
}

pub struct ReadonlyBucket<'a, S: ReadonlyStorage, T>
where
    T: Serialize + DeserializeOwned,
{
    storage: &'a S,
    // see https://doc.rust-lang.org/std/marker/struct.PhantomData.html#unused-type-parameters for why this is needed
    data: PhantomData<&'a T>,
    prefix: Vec<u8>,
}

impl<'a, S: ReadonlyStorage, T> ReadonlyBucket<'a, S, T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn new(namespace: &[u8], storage: &'a S) -> Self {
        ReadonlyBucket {
            prefix: to_length_prefixed(namespace),
            storage,
            data: PhantomData,
        }
    }

    pub fn multilevel(namespaces: &[&[u8]], storage: &'a S) -> Self {
        ReadonlyBucket {
            prefix: to_length_prefixed_nested(namespaces),
            storage,
            data: PhantomData,
        }
    }

    /// load will return an error if no data is set at the given key, or on parse error
    pub fn load(&self, key: &[u8]) -> StdResult<T> {
        let value = get_with_prefix(self.storage, &self.prefix, key);
        must_deserialize(&value)
    }

    /// may_load will parse the data stored at the key if present, returns Ok(None) if no data there.
    /// returns an error on issues parsing
    pub fn may_load(&self, key: &[u8]) -> StdResult<Option<T>> {
        let value = get_with_prefix(self.storage, &self.prefix, key);
        may_deserialize(&value)
    }

    #[cfg(feature = "iterator")]
    pub fn range<'b>(
        &'b self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> StdResult<Box<dyn Iterator<Item = StdResult<KV<T>>> + 'b>> {
        let mapped = range_with_prefix(self.storage, &self.prefix, start, end, order)?
            .map(deserialize_kv::<T>);
        Ok(Box::new(mapped))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::MockStorage;
    use cosmwasm_std::{generic_err, not_found};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Data {
        pub name: String,
        pub age: i32,
    }

    #[test]
    fn store_and_load() {
        let mut store = MockStorage::new();
        let mut bucket = bucket::<_, Data>(b"data", &mut store);

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
    fn readonly_works() {
        let mut store = MockStorage::new();
        let mut bucket = bucket::<_, Data>(b"data", &mut store);

        // save data
        let data = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        bucket.save(b"maria", &data).unwrap();

        let reader = bucket_read::<_, Data>(b"data", &mut store);

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
        let mut bucket1 = bucket::<_, Data>(b"data", &mut store);

        // save data
        let data = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        bucket1.save(b"maria", &data).unwrap();

        let mut bucket2 = bucket::<_, Data>(b"dat", &mut store);

        // save data (dat, amaria) vs (data, maria)
        let data2 = Data {
            name: "Amen".to_string(),
            age: 67,
        };
        bucket2.save(b"amaria", &data2).unwrap();

        // load one
        let reader = bucket_read::<_, Data>(b"data", &store);
        let loaded = reader.load(b"maria").unwrap();
        assert_eq!(data, loaded);
        // no cross load
        assert_eq!(None, reader.may_load(b"amaria").unwrap());

        // load the other
        let reader2 = bucket_read::<_, Data>(b"dat", &store);
        let loaded2 = reader2.load(b"amaria").unwrap();
        assert_eq!(data2, loaded2);
        // no cross load
        assert_eq!(None, reader2.may_load(b"maria").unwrap());
    }

    #[test]
    fn update_success() {
        let mut store = MockStorage::new();
        let mut bucket = bucket::<_, Data>(b"data", &mut store);

        // initial data
        let init = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        bucket.save(b"maria", &init).unwrap();

        // it's my birthday
        let birthday = |mayd: Option<Data>| -> StdResult<Data> {
            let mut d = mayd.ok_or(not_found("Data"))?;
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
        let mut bucket = bucket::<_, Data>(b"data", &mut store);
        let init = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        bucket.save(b"maria", &init).unwrap();

        // show we can capture data from the closure
        let mut old_age = 0i32;
        bucket
            .update(b"maria", |mayd: Option<Data>| {
                let mut d = mayd.ok_or(not_found("Data"))?;
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
        let mut bucket = bucket::<_, Data>(b"data", &mut store);

        // initial data
        let init = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        bucket.save(b"maria", &init).unwrap();

        // it's my birthday
        let output = bucket.update(b"maria", |_d| Err(generic_err("cuz i feel like it")));
        assert!(output.is_err());

        // load it properly
        let loaded = bucket.load(b"maria").unwrap();
        assert_eq!(loaded, init);
    }

    #[test]
    fn update_handles_on_no_data() {
        let mut store = MockStorage::new();
        let mut bucket = bucket::<_, Data>(b"data", &mut store);

        let init_value = Data {
            name: "Maria".to_string(),
            age: 42,
        };

        // it's my birthday
        let output = bucket
            .update(b"maria", |d| match d {
                Some(_) => Err(generic_err("Ensure this was empty")),
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
        let mut bucket = bucket::<_, Data>(b"data", &mut store);

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

        let res_data: StdResult<Vec<KV<Data>>> = bucket
            .range(None, None, Order::Ascending)
            .unwrap()
            .collect();
        let data = res_data.unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0], (b"jose".to_vec(), jose.clone()));
        assert_eq!(data[1], (b"maria".to_vec(), maria.clone()));

        // also works for readonly
        let read_bucket = bucket_read::<_, Data>(b"data", &store);
        let res_data: StdResult<Vec<KV<Data>>> = read_bucket
            .range(None, None, Order::Ascending)
            .unwrap()
            .collect();
        let data = res_data.unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0], (b"jose".to_vec(), jose));
        assert_eq!(data[1], (b"maria".to_vec(), maria));
    }
}
