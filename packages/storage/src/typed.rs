use serde::{de::DeserializeOwned, ser::Serialize};
use std::marker::PhantomData;

use cosmwasm_std::{to_vec, ReadonlyStorage, Result, Storage};

use crate::type_helpers::{may_deserialize, must_deserialize};

pub fn typed<S: Storage, T>(storage: &mut S) -> TypedStorage<S, T>
where
    T: Serialize + DeserializeOwned,
{
    TypedStorage::new(storage)
}

pub fn typed_read<S: ReadonlyStorage, T>(storage: &S) -> ReadonlyTypedStorage<S, T>
where
    T: Serialize + DeserializeOwned,
{
    ReadonlyTypedStorage::new(storage)
}

pub struct TypedStorage<'a, S: Storage, T>
where
    T: Serialize + DeserializeOwned,
{
    storage: &'a mut S,
    // see https://doc.rust-lang.org/std/marker/struct.PhantomData.html#unused-type-parameters for why this is needed
    data: PhantomData<&'a T>,
}

impl<'a, S: Storage, T> TypedStorage<'a, S, T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn new(storage: &'a mut S) -> Self {
        TypedStorage {
            storage,
            data: PhantomData,
        }
    }

    /// save will serialize the model and store, returns an error on serialization issues
    pub fn save(&mut self, key: &[u8], data: &T) -> Result<()> {
        self.storage.set(key, &to_vec(data)?)
    }

    /// load will return an error if no data is set at the given key, or on parse error
    pub fn load(&self, key: &[u8]) -> Result<T> {
        let value = self.storage.get(key);
        must_deserialize(&value)
    }

    /// may_load will parse the data stored at the key if present, returns Ok(None) if no data there.
    /// returns an error on issues parsing
    pub fn may_load(&self, key: &[u8]) -> Result<Option<T>> {
        let value = self.storage.get(key);
        may_deserialize(&value)
    }

    /// update will load the data, perform the specified action, and store the result
    /// in the database. This is shorthand for some common sequences, which may be useful
    ///
    /// This is the least stable of the APIs, and definitely needs some usage
    pub fn update(&mut self, key: &[u8], action: &dyn Fn(Option<T>) -> Result<T>) -> Result<T> {
        let input = self.may_load(key)?;
        let output = action(input)?;
        self.save(key, &output)?;
        Ok(output)
    }
}

pub struct ReadonlyTypedStorage<'a, S: ReadonlyStorage, T>
where
    T: Serialize + DeserializeOwned,
{
    storage: &'a S,
    // see https://doc.rust-lang.org/std/marker/struct.PhantomData.html#unused-type-parameters for why this is needed
    data: PhantomData<&'a T>,
}

impl<'a, S: ReadonlyStorage, T> ReadonlyTypedStorage<'a, S, T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn new(storage: &'a S) -> Self {
        ReadonlyTypedStorage {
            storage,
            data: PhantomData,
        }
    }

    /// load will return an error if no data is set at the given key, or on parse error
    pub fn load(&self, key: &[u8]) -> Result<T> {
        let value = self.storage.get(key);
        must_deserialize(&value)
    }

    /// may_load will parse the data stored at the key if present, returns Ok(None) if no data there.
    /// returns an error on issues parsing
    pub fn may_load(&self, key: &[u8]) -> Result<Option<T>> {
        let value = self.storage.get(key);
        may_deserialize(&value)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::MockStorage;
    use cosmwasm_std::{contract_err, NotFound};
    use serde::{Deserialize, Serialize};
    use snafu::OptionExt;

    use crate::prefixed;

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Data {
        pub name: String,
        pub age: i32,
    }

    #[test]
    fn store_and_load() {
        let mut store = MockStorage::new();
        let mut bucket = TypedStorage::<_, Data>::new(&mut store);

        // check empty data handling
        assert!(bucket.load(b"maria").is_err());
        assert_eq!(bucket.may_load(b"maria").unwrap(), None);

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
    fn store_with_prefix() {
        let mut store = MockStorage::new();
        let mut space = prefixed(b"data", &mut store);
        let mut bucket = typed::<_, Data>(&mut space);

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
        let mut bucket = typed::<_, Data>(&mut store);

        // save data
        let data = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        bucket.save(b"maria", &data).unwrap();

        let reader = typed_read::<_, Data>(&mut store);

        // check empty data handling
        assert!(reader.load(b"john").is_err());
        assert_eq!(reader.may_load(b"john").unwrap(), None);

        // load it properly
        let loaded = reader.load(b"maria").unwrap();
        assert_eq!(data, loaded);
    }

    #[test]
    fn update_success() {
        let mut store = MockStorage::new();
        let mut bucket = typed::<_, Data>(&mut store);

        // initial data
        let init = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        bucket.save(b"maria", &init).unwrap();

        // it's my birthday (fail if no data)
        let birthday = |mayd: Option<Data>| -> Result<Data> {
            let mut d = mayd.context(NotFound { kind: "Data" })?;
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
    fn update_fails_on_error() {
        let mut store = MockStorage::new();
        let mut bucket = typed::<_, Data>(&mut store);

        // initial data
        let init = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        bucket.save(b"maria", &init).unwrap();

        // it's my birthday
        let output = bucket.update(b"maria", &|_d| contract_err("cuz i feel like it"));
        assert!(output.is_err());

        // load it properly
        let loaded = bucket.load(b"maria").unwrap();
        assert_eq!(loaded, init);
    }

    #[test]
    fn update_handles_on_no_data() {
        let mut store = MockStorage::new();
        let mut bucket = typed::<_, Data>(&mut store);

        let init_value = Data {
            name: "Maria".to_string(),
            age: 42,
        };

        // it's my birthday
        let output = bucket
            .update(b"maria", &|d| match d {
                Some(_) => contract_err("Ensure this was empty"),
                None => Ok(init_value.clone()),
            })
            .unwrap();
        assert_eq!(output, init_value);

        // nothing stored
        let loaded = bucket.load(b"maria").unwrap();
        assert_eq!(loaded, init_value);
    }
}
