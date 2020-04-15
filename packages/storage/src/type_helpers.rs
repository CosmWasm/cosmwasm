use serde::de::DeserializeOwned;
use std::any::type_name;

#[cfg(feature = "iterator")]
use cosmwasm_std::KV;
use cosmwasm_std::{from_slice, NotFound, StdResult};

/// may_deserialize parses json bytes from storage (Option), returning Ok(None) if no data present
///
/// value is an odd type, but this is meant to be easy to use with output from storage.get (Option<Vec<u8>>)
/// and value.map(|s| s.as_slice()) seems trickier than &value
pub(crate) fn may_deserialize<T: DeserializeOwned>(
    value: &Option<Vec<u8>>,
) -> StdResult<Option<T>> {
    match value {
        Some(d) => Ok(Some(from_slice(d.as_slice())?)),
        None => Ok(None),
    }
}

/// must_deserialize parses json bytes from storage (Option), returning NotFound error if no data present
pub(crate) fn must_deserialize<T: DeserializeOwned>(value: &Option<Vec<u8>>) -> StdResult<T> {
    match value {
        Some(d) => from_slice(&d),
        None => NotFound {
            kind: type_name::<T>(),
        }
        .fail(),
    }
}

#[cfg(feature = "iterator")]
pub(crate) fn deserialize_kv<T: DeserializeOwned>(kv: KV) -> StdResult<KV<T>> {
    let (k, v) = kv;
    let t = from_slice::<T>(&v)?;
    Ok((k, t))
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::{to_vec, Error};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Data {
        pub name: String,
        pub age: i32,
    }

    #[test]
    fn serialize_and_deserialize() {
        // save data
        let data = Data {
            name: "Maria".to_string(),
            age: 42,
        };
        let value = to_vec(&data).unwrap();
        let loaded = Some(value);

        //        let parsed: Data = deserialize(loaded.map(|s| s.as_slice())).unwrap();
        //        assert_eq!(parsed, data);
        let parsed: Data = must_deserialize(&loaded).unwrap();
        assert_eq!(parsed, data);

        let may_parse: Option<Data> = may_deserialize(&loaded).unwrap();
        assert_eq!(may_parse, Some(data));
    }

    #[test]
    fn handle_none() {
        let may_parse = may_deserialize::<Data>(&None).unwrap();
        assert_eq!(may_parse, None);

        let parsed = must_deserialize::<Data>(&None);
        match parsed {
            // if we used short_type_name, this would just be Data
            Err(Error::NotFound { kind, .. }) => {
                assert_eq!(kind, "cosmwasm_storage::type_helpers::test::Data")
            }
            Err(e) => panic!("Unexpected error {}", e),
            Ok(_) => panic!("should error"),
        }
    }
}
