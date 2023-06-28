use serde::{de::DeserializeOwned, ser::Serialize};
use std::marker::PhantomData;

use crate::cosmwasm_std::{to_vec, StdError, StdResult, Storage};
#[cfg(feature = "iterator")]
use crate::cosmwasm_std::{Order, Record};

use crate::length_prefixed::{to_length_prefixed, to_length_prefixed_nested};
#[cfg(feature = "iterator")]
use crate::namespace_helpers::range_with_prefix;
use crate::namespace_helpers::{get_with_prefix, remove_with_prefix, set_with_prefix};
#[cfg(feature = "iterator")]
use crate::type_helpers::deserialize_kv;
use crate::type_helpers::{may_deserialize, must_deserialize};

/// An alias of Bucket::new for less verbose usage
pub fn bucket<'a, T>(storage: &'a mut dyn Storage, namespace: &[u8]) -> Bucket<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    Bucket::new(storage, namespace)
}

/// An alias of ReadonlyBucket::new for less verbose usage
pub fn bucket_read<'a, T>(storage: &'a dyn Storage, namespace: &[u8]) -> ReadonlyBucket<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    ReadonlyBucket::new(storage, namespace)
}

pub struct Bucket<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    storage: &'a mut dyn Storage,
    prefix: Vec<u8>,
    // see https://doc.rust-lang.org/std/marker/struct.PhantomData.html#unused-type-parameters for why this is needed
    data: PhantomData<T>,
}

impl<'a, T> Bucket<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn new(storage: &'a mut dyn Storage, namespace: &[u8]) -> Self {
        Bucket {
            storage,
            prefix: to_length_prefixed(namespace),
            data: PhantomData,
        }
    }

    pub fn multilevel(storage: &'a mut dyn Storage, namespaces: &[&[u8]]) -> Self {
        Bucket {
            storage,
            prefix: to_length_prefixed_nested(namespaces),
            data: PhantomData,
        }
    }

    /// save will serialize the model and store, returns an error on serialization issues
    pub fn save(&mut self, key: &[u8], data: &T) -> StdResult<()> {
        set_with_prefix(self.storage, &self.prefix, key, &to_vec(data)?);
        Ok(())
    }

    pub fn remove(&mut self, key: &[u8]) {
        remove_with_prefix(self.storage, &self.prefix, key)
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
    ) -> Box<dyn Iterator<Item = StdResult<Record<T>>> + 'b> {
        let mapped = range_with_prefix(self.storage, &self.prefix, start, end, order)
            .map(deserialize_kv::<T>);
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

pub struct ReadonlyBucket<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    storage: &'a dyn Storage,
    prefix: Vec<u8>,
    // see https://doc.rust-lang.org/std/marker/struct.PhantomData.html#unused-type-parameters for why this is needed
    data: PhantomData<T>,
}

impl<'a, T> ReadonlyBucket<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn new(storage: &'a dyn Storage, namespace: &[u8]) -> Self {
        ReadonlyBucket {
            storage,
            prefix: to_length_prefixed(namespace),
            data: PhantomData,
        }
    }

    pub fn multilevel(storage: &'a dyn Storage, namespaces: &[&[u8]]) -> Self {
        ReadonlyBucket {
            storage,
            prefix: to_length_prefixed_nested(namespaces),
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
    ) -> Box<dyn Iterator<Item = StdResult<Record<T>>> + 'b> {
        let mapped = range_with_prefix(self.storage, &self.prefix, start, end, order)
            .map(deserialize_kv::<T>);
        Box::new(mapped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secret_cosmwasm_std::testing::MockStorage;
    use secret_cosmwasm_std::StdError;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Data {
        pub name: String,
        pub age: i32,
    }

    #[test]
    fn store_and_load() {
        let mut store = MockStorage::new();
        let mut bucket = bucket::<Data>(&mut store, b"data");

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
        let mut bucket = bucket::<Data>(&mut store, b"data");

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
        let mut bucket = bucket::<Data>(&mut store, b"data");

        // save data
        let data = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        bucket.save(b"maria", &data).unwrap();

        let reader = bucket_read::<Data>(&store, b"data");

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
        let mut bucket1 = bucket::<Data>(&mut store, b"data");

        // save data
        let data = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        bucket1.save(b"maria", &data).unwrap();

        let mut bucket2 = bucket::<Data>(&mut store, b"dat");

        // save data (dat, amaria) vs (data, maria)
        let data2 = Data {
            name: "Amen".to_string(),
            age: 67,
        };
        bucket2.save(b"amaria", &data2).unwrap();

        // load one
        let reader = bucket_read::<Data>(&store, b"data");
        let loaded = reader.load(b"maria").unwrap();
        assert_eq!(data, loaded);
        // no cross load
        assert_eq!(None, reader.may_load(b"amaria").unwrap());

        // load the other
        let reader2 = bucket_read::<Data>(&store, b"dat");
        let loaded2 = reader2.load(b"amaria").unwrap();
        assert_eq!(data2, loaded2);
        // no cross load
        assert_eq!(None, reader2.may_load(b"maria").unwrap());
    }

    #[test]
    fn update_success() {
        let mut store = MockStorage::new();
        let mut bucket = bucket::<Data>(&mut store, b"data");

        // initial data
        let init = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        bucket.save(b"maria", &init).unwrap();

        // it's my birthday
        let birthday = |mayd: Option<Data>| -> StdResult<Data> {
            let mut d = mayd.ok_or_else(|| StdError::not_found("Data"))?;
            d.age += 1;
            Ok(d)
        };
        let output = bucket.update(b"maria", birthday).unwrap();
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
        let mut bucket = bucket::<Data>(&mut store, b"data");
        let init = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        bucket.save(b"maria", &init).unwrap();

        // show we can capture data from the closure
        let mut old_age = 0i32;
        bucket
            .update(b"maria", |mayd: Option<Data>| -> StdResult<_> {
                let mut d = mayd.ok_or_else(|| StdError::not_found("Data"))?;
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
        let mut bucket = bucket::<Data>(&mut store, b"data");

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
        let mut bucket = bucket::<Data>(&mut store, b"data");

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
                Err(MyError::NotFound)
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
        let mut bucket = bucket::<Data>(&mut store, b"data");

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
        let mut bucket = bucket::<Data>(&mut store, b"data");

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

        let res_data: StdResult<Vec<Record<Data>>> =
            bucket.range(None, None, Order::Ascending).collect();
        let data = res_data.unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0], (b"jose".to_vec(), jose.clone()));
        assert_eq!(data[1], (b"maria".to_vec(), maria.clone()));

        // also works for readonly
        let read_bucket = bucket_read::<Data>(&store, b"data");
        let res_data: StdResult<Vec<Record<Data>>> =
            read_bucket.range(None, None, Order::Ascending).collect();
        let data = res_data.unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0], (b"jose".to_vec(), jose));
        assert_eq!(data[1], (b"maria".to_vec(), maria));
    }
}
