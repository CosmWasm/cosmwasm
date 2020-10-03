use serde::{de::DeserializeOwned, ser::Serialize};
use std::marker::PhantomData;

use cosmwasm_std::{to_vec, ReadonlyStorage, StdError, StdResult, Storage};

use crate::length_prefixed::to_length_prefixed;
use crate::type_helpers::{may_deserialize, must_deserialize};

/// An alias of Singleton::new for less verbose usage
pub fn singleton<T>(key: &[u8]) -> Singleton<T>
where
    T: Serialize + DeserializeOwned,
{
    Singleton::new(key)
}

/// Singleton effectively combines PrefixedStorage with TypedStorage to
/// work on a single storage key. It performs the to_length_prefixed transformation
/// on the given name to ensure no collisions, and then provides the standard
/// TypedStorage accessors, without requiring a key (which is defined in the constructor)
pub struct Singleton<T>
where
    T: Serialize + DeserializeOwned,
{
    key: Vec<u8>,
    // see https://doc.rust-lang.org/std/marker/struct.PhantomData.html#unused-type-parameters for why this is needed
    data: PhantomData<T>,
}

impl<T> Singleton<T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn new(key: &[u8]) -> Self {
        Singleton {
            key: to_length_prefixed(key),
            data: PhantomData,
        }
    }

    /// save will serialize the model and store, returns an error on serialization issues
    pub fn save<S: Storage>(&self, store: &mut S, data: &T) -> StdResult<()> {
        store.set(&self.key, &to_vec(data)?);
        Ok(())
    }

    pub fn remove<S: Storage>(&self, store: &mut S) {
        store.remove(&self.key)
    }

    /// load will return an error if no data is set at the given key, or on parse error
    pub fn load<S: ReadonlyStorage>(&self, store: &S) -> StdResult<T> {
        let value = store.get(&self.key);
        must_deserialize(&value)
    }

    /// may_load will parse the data stored at the key if present, returns Ok(None) if no data there.
    /// returns an error on issues parsing
    pub fn may_load<S: ReadonlyStorage>(&self, store: &S) -> StdResult<Option<T>> {
        let value = store.get(&self.key);
        may_deserialize(&value)
    }

    /// update will load the data, perform the specified action, and store the result
    /// in the database. This is shorthand for some common sequences, which may be useful
    ///
    /// This is the least stable of the APIs, and definitely needs some usage
    pub fn update<A, E, S>(&self, store: &mut S, action: A) -> Result<T, E>
    where
        A: FnOnce(T) -> Result<T, E>,
        E: From<StdError>,
        S: Storage,
    {
        let input = self.load(store)?;
        let output = action(input)?;
        self.save(store, &output)?;
        Ok(output)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::MockStorage;
    use serde::{Deserialize, Serialize};

    use cosmwasm_std::StdError;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Config {
        pub owner: String,
        pub max_tokens: i32,
    }

    #[test]
    fn save_and_load() {
        let mut store = MockStorage::new();
        let single = Singleton::<Config>::new(b"config");

        assert!(single.load(&store).is_err());
        assert_eq!(single.may_load(&store).unwrap(), None);

        let cfg = Config {
            owner: "admin".to_string(),
            max_tokens: 1234,
        };
        single.save(&mut store, &cfg).unwrap();

        assert_eq!(cfg, single.load(&store).unwrap());
    }

    #[test]
    fn remove_works() {
        let mut store = MockStorage::new();
        let single = Singleton::<Config>::new(b"config");

        // store data
        let cfg = Config {
            owner: "admin".to_string(),
            max_tokens: 1234,
        };
        single.save(&mut store, &cfg).unwrap();
        assert_eq!(cfg, single.load(&store).unwrap());

        // remove it and loads None
        single.remove(&mut store);
        assert_eq!(None, single.may_load(&store).unwrap());

        // safe to remove 2 times
        single.remove(&mut store);
        assert_eq!(None, single.may_load(&store).unwrap());
    }

    #[test]
    fn isolated_reads() {
        let mut store = MockStorage::new();
        let writer = singleton::<Config>(b"config");
        let other_reader = singleton::<Config>(b"config2");

        let cfg = Config {
            owner: "admin".to_string(),
            max_tokens: 1234,
        };
        writer.save(&mut store, &cfg).unwrap();

        assert_eq!(cfg, writer.load(&store).unwrap());
        assert_eq!(None, other_reader.may_load(&store).unwrap());
    }

    #[test]
    fn update_success() {
        let mut store = MockStorage::new();
        let writer = singleton::<Config>(b"config");

        let cfg = Config {
            owner: "admin".to_string(),
            max_tokens: 1234,
        };
        writer.save(&mut store, &cfg).unwrap();

        let output = writer.update(&mut store, |mut c| -> StdResult<_> {
            c.max_tokens *= 2;
            Ok(c)
        });
        let expected = Config {
            owner: "admin".to_string(),
            max_tokens: 2468,
        };
        assert_eq!(output.unwrap(), expected);
        assert_eq!(writer.load(&store).unwrap(), expected);
    }

    #[test]
    fn update_can_change_variable_from_outer_scope() {
        let mut store = MockStorage::new();
        let writer = singleton::<Config>(b"config");
        let cfg = Config {
            owner: "admin".to_string(),
            max_tokens: 1234,
        };
        writer.save(&mut store, &cfg).unwrap();

        let mut old_max_tokens = 0i32;
        writer
            .update(&mut store, |mut c| -> StdResult<_> {
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
        let writer = singleton::<Config>(b"config");

        let cfg = Config {
            owner: "admin".to_string(),
            max_tokens: 1234,
        };
        writer.save(&mut store, &cfg).unwrap();

        let output = writer.update(&mut store, &|_c| Err(StdError::underflow(4, 7)));
        match output.unwrap_err() {
            StdError::Underflow { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
        assert_eq!(writer.load(&store).unwrap(), cfg);
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
        let writer = singleton::<Config>(b"config");

        let cfg = Config {
            owner: "admin".to_string(),
            max_tokens: 1234,
        };
        writer.save(&mut store, &cfg).unwrap();

        let res = writer.update(&mut store, |mut c| {
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
        assert_eq!(writer.load(&store).unwrap(), cfg);
    }
}
