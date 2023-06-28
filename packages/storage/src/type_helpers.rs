use serde::de::DeserializeOwned;
use std::any::type_name;

#[cfg(feature = "iterator")]
use crate::cosmwasm_std::Record;
use crate::cosmwasm_std::{from_slice, StdError, StdResult};

/// may_deserialize parses json bytes from storage (Option), returning Ok(None) if no data present
///
/// value is an odd type, but this is meant to be easy to use with output from storage.get (Option<Vec<u8>>)
/// and value.map(|s| s.as_slice()) seems trickier than &value
pub(crate) fn may_deserialize<T: DeserializeOwned>(
    value: &Option<Vec<u8>>,
) -> StdResult<Option<T>> {
    match value {
        Some(data) => Ok(Some(from_slice(data)?)),
        None => Ok(None),
    }
}

/// must_deserialize parses json bytes from storage (Option), returning NotFound error if no data present
pub(crate) fn must_deserialize<T: DeserializeOwned>(value: &Option<Vec<u8>>) -> StdResult<T> {
    match value {
        Some(data) => from_slice(data),
        None => Err(StdError::not_found(type_name::<T>())),
    }
}

#[cfg(feature = "iterator")]
pub(crate) fn deserialize_kv<T: DeserializeOwned>(kv: Record<Vec<u8>>) -> StdResult<Record<T>> {
    let (k, v) = kv;
    let t = from_slice::<T>(&v)?;
    Ok((k, t))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cosmwasm_std::{to_vec, StdError};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Person {
        pub name: String,
        pub age: i32,
    }

    #[test]
    fn may_deserialize_handles_some() {
        let person = Person {
            name: "Maria".to_string(),
            age: 42,
        };
        let value = to_vec(&person).unwrap();

        let may_parse: Option<Person> = may_deserialize(&Some(value)).unwrap();
        assert_eq!(may_parse, Some(person));
    }

    #[test]
    fn may_deserialize_handles_none() {
        let may_parse = may_deserialize::<Person>(&None).unwrap();
        assert_eq!(may_parse, None);
    }

    #[test]
    fn must_deserialize_handles_some() {
        let person = Person {
            name: "Maria".to_string(),
            age: 42,
        };
        let value = to_vec(&person).unwrap();
        let loaded = Some(value);

        let parsed: Person = must_deserialize(&loaded).unwrap();
        assert_eq!(parsed, person);
    }

    #[test]
    fn must_deserialize_handles_none() {
        let parsed = must_deserialize::<Person>(&None);
        match parsed.unwrap_err() {
            StdError::NotFound { kind, .. } => {
                assert_eq!(kind, "secret_cosmwasm_storage::type_helpers::tests::Person")
            }
            e => panic!("Unexpected error {}", e),
        }
    }
}
