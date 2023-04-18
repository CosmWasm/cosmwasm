use serde::{de::DeserializeOwned, ser::Serialize};
use std::marker::PhantomData;

use crate::cosmwasm_std::{to_vec, StdError, StdResult, Storage};

use crate::length_prefixed::to_length_prefixed;
use crate::type_helpers::{may_deserialize, must_deserialize};

/// An alias of Singleton::new for less verbose usage
pub fn singleton<'a, T>(storage: &'a mut dyn Storage, key: &[u8]) -> Singleton<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    Singleton::new(storage, key)
}

/// An alias of ReadonlySingleton::new for less verbose usage
pub fn singleton_read<'a, T>(storage: &'a dyn Storage, key: &[u8]) -> ReadonlySingleton<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    ReadonlySingleton::new(storage, key)
}

/// Singleton effectively combines PrefixedStorage with TypedStorage to
/// work on a single storage key. It performs the to_length_prefixed transformation
/// on the given name to ensure no collisions, and then provides the standard
/// TypedStorage accessors, without requiring a key (which is defined in the constructor)
pub struct Singleton<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    storage: &'a mut dyn Storage,
    key: Vec<u8>,
    // see https://doc.rust-lang.org/std/marker/struct.PhantomData.html#unused-type-parameters for why this is needed
    data: PhantomData<T>,
}

impl<'a, T> Singleton<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn new(storage: &'a mut dyn Storage, key: &[u8]) -> Self {
        Singleton {
            storage,
            key: to_length_prefixed(key),
            data: PhantomData,
        }
    }

    /// save will serialize the model and store, returns an error on serialization issues
    pub fn save(&mut self, data: &T) -> StdResult<()> {
        self.storage.set(&self.key, &to_vec(data)?);
        Ok(())
    }

    pub fn remove(&mut self) {
        self.storage.remove(&self.key)
    }

    /// load will return an error if no data is set at the given key, or on parse error
    pub fn load(&self) -> StdResult<T> {
        let value = self.storage.get(&self.key);
        must_deserialize(&value)
    }

    /// may_load will parse the data stored at the key if present, returns Ok(None) if no data there.
    /// returns an error on issues parsing
    pub fn may_load(&self) -> StdResult<Option<T>> {
        let value = self.storage.get(&self.key);
        may_deserialize(&value)
    }

    /// update will load the data, perform the specified action, and store the result
    /// in the database. This is shorthand for some common sequences, which may be useful
    ///
    /// This is the least stable of the APIs, and definitely needs some usage
    pub fn update<A, E>(&mut self, action: A) -> Result<T, E>
    where
        A: FnOnce(T) -> Result<T, E>,
        E: From<StdError>,
    {
        let input = self.load()?;
        let output = action(input)?;
        self.save(&output)?;
        Ok(output)
    }
}

/// ReadonlySingleton only requires a Storage and exposes only the
/// methods of Singleton that don't modify state.
pub struct ReadonlySingleton<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    storage: &'a dyn Storage,
    key: Vec<u8>,
    // see https://doc.rust-lang.org/std/marker/struct.PhantomData.html#unused-type-parameters for why this is needed
    data: PhantomData<T>,
}

impl<'a, T> ReadonlySingleton<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn new(storage: &'a dyn Storage, key: &[u8]) -> Self {
        ReadonlySingleton {
            storage,
            key: to_length_prefixed(key),
            data: PhantomData,
        }
    }

    /// load will return an error if no data is set at the given key, or on parse error
    pub fn load(&self) -> StdResult<T> {
        let value = self.storage.get(&self.key);
        must_deserialize(&value)
    }

    /// may_load will parse the data stored at the key if present, returns Ok(None) if no data there.
    /// returns an error on issues parsing
    pub fn may_load(&self) -> StdResult<Option<T>> {
        let value = self.storage.get(&self.key);
        may_deserialize(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cosmwasm_std::testing::MockStorage;
    use serde::{Deserialize, Serialize};

    use crate::cosmwasm_std::{OverflowError, OverflowOperation, StdError};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Config {
        pub owner: String,
        pub max_tokens: i32,
    }

    #[test]
    fn save_and_load() {
        let mut store = MockStorage::new();
        let mut single = Singleton::<Config>::new(&mut store, b"config");

        assert!(single.load().is_err());
        assert_eq!(single.may_load().unwrap(), None);

        let cfg = Config {
            owner: "admin".to_string(),
            max_tokens: 1234,
        };
        single.save(&cfg).unwrap();

        assert_eq!(cfg, single.load().unwrap());
    }

    #[test]
    fn remove_works() {
        let mut store = MockStorage::new();
        let mut single = Singleton::<Config>::new(&mut store, b"config");

        // store data
        let cfg = Config {
            owner: "admin".to_string(),
            max_tokens: 1234,
        };
        single.save(&cfg).unwrap();
        assert_eq!(cfg, single.load().unwrap());

        // remove it and loads None
        single.remove();
        assert_eq!(None, single.may_load().unwrap());

        // safe to remove 2 times
        single.remove();
        assert_eq!(None, single.may_load().unwrap());
    }

    #[test]
    fn isolated_reads() {
        let mut store = MockStorage::new();
        let mut writer = singleton::<Config>(&mut store, b"config");

        let cfg = Config {
            owner: "admin".to_string(),
            max_tokens: 1234,
        };
        writer.save(&cfg).unwrap();

        let reader = singleton_read::<Config>(&store, b"config");
        assert_eq!(cfg, reader.load().unwrap());

        let other_reader = singleton_read::<Config>(&store, b"config2");
        assert_eq!(other_reader.may_load().unwrap(), None);
    }

    #[test]
    fn update_success() {
        let mut store = MockStorage::new();
        let mut writer = singleton::<Config>(&mut store, b"config");

        let cfg = Config {
            owner: "admin".to_string(),
            max_tokens: 1234,
        };
        writer.save(&cfg).unwrap();

        let output = writer.update(|mut c| -> StdResult<_> {
            c.max_tokens *= 2;
            Ok(c)
        });
        let expected = Config {
            owner: "admin".to_string(),
            max_tokens: 2468,
        };
        assert_eq!(output.unwrap(), expected);
        assert_eq!(writer.load().unwrap(), expected);
    }

    #[test]
    fn update_can_change_variable_from_outer_scope() {
        let mut store = MockStorage::new();
        let mut writer = singleton::<Config>(&mut store, b"config");
        let cfg = Config {
            owner: "admin".to_string(),
            max_tokens: 1234,
        };
        writer.save(&cfg).unwrap();

        let mut old_max_tokens = 0i32;
        writer
            .update(|mut c| -> StdResult<_> {
                old_max_tokens = c.max_tokens;
                c.max_tokens *= 2;
                Ok(c)
            })
            .unwrap();
        assert_eq!(old_max_tokens, 1234);
    }

    #[test]
    fn update_does_not_change_data_on_error() {
        let mut store = MockStorage::new();
        let mut writer = singleton::<Config>(&mut store, b"config");

        let cfg = Config {
            owner: "admin".to_string(),
            max_tokens: 1234,
        };
        writer.save(&cfg).unwrap();

        let output = writer.update(|_c| {
            Err(StdError::from(OverflowError::new(
                OverflowOperation::Sub,
                4,
                7,
            )))
        });
        match output.unwrap_err() {
            StdError::Overflow { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
        assert_eq!(writer.load().unwrap(), cfg);
    }

    #[test]
    fn update_supports_custom_errors() {
        #[derive(Debug)]
        enum MyError {
            Std(StdError),
            Foo,
        }

        impl From<StdError> for MyError {
            fn from(original: StdError) -> MyError {
                MyError::Std(original)
            }
        }

        let mut store = MockStorage::new();
        let mut writer = singleton::<Config>(&mut store, b"config");

        let cfg = Config {
            owner: "admin".to_string(),
            max_tokens: 1234,
        };
        writer.save(&cfg).unwrap();

        let res = writer.update(|mut c| {
            if c.max_tokens > 5000 {
                return Err(MyError::Foo);
            }
            if c.max_tokens > 20 {
                return Err(StdError::generic_err("broken stuff").into()); // Uses Into to convert StdError to MyError
            }
            if c.max_tokens > 10 {
                to_vec(&c)?; // Uses From to convert StdError to MyError
            }
            c.max_tokens += 20;
            Ok(c)
        });
        match res.unwrap_err() {
            MyError::Std(StdError::GenericErr { .. }) => {}
            err => panic!("Unexpected error: {:?}", err),
        }
        assert_eq!(writer.load().unwrap(), cfg);
    }
}
