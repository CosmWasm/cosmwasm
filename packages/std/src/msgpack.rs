// This file mostly re-exports some methods from rmp-serde
// The reason is two fold:
// 1. To easily ensure that all calling libraries use the same version (minimize code size)
// 2. To allow us to switch out to another MessagePack library if needed

use core::any::type_name;
use serde::{de::DeserializeOwned, Serialize};

use crate::Binary;
use crate::{StdError, StdResult};

/// Deserializes the given MessagePack bytes to a data structure.
///
/// Errors if the input is not valid MessagePack or cannot be deserialized to the given type.
pub fn from_msgpack<T: DeserializeOwned>(value: impl AsRef<[u8]>) -> StdResult<T> {
    rmp_serde::from_read(value.as_ref()).map_err(|e| StdError::parse_err(type_name::<T>(), e))
}

/// Serializes the given data structure as a MessagePack byte vector.
pub fn to_msgpack_vec<T>(data: &T) -> StdResult<Vec<u8>>
where
    T: Serialize + ?Sized,
{
    rmp_serde::to_vec_named(data).map_err(|e| StdError::serialize_err(type_name::<T>(), e))
}

/// Serializes the given data structure as MessagePack bytes.
pub fn to_msgpack_binary<T>(data: &T) -> StdResult<Binary>
where
    T: Serialize + ?Sized,
{
    to_msgpack_vec(data).map(Binary::new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[serde(rename_all = "snake_case")]
    enum SomeMsg {
        Refund {},
        ReleaseAll {
            image: String,
            amount: u32,
            time: u64,
            karma: i32,
        },
        Cowsay {
            text: String,
        },
    }

    fn refund_test_vector() -> (SomeMsg, &'static [u8]) {
        let msg = SomeMsg::Refund {};
        let serialized = &[129, 166, 114, 101, 102, 117, 110, 100, 128];
        (msg, serialized)
    }

    fn release_all_test_vector() -> (SomeMsg, &'static [u8]) {
        let msg = SomeMsg::ReleaseAll {
            image: "foo".to_string(),
            amount: 42,
            time: 18446744073709551615,
            karma: -17,
        };
        let serialized = &[
            129, 171, 114, 101, 108, 101, 97, 115, 101, 95, 97, 108, 108, 132, 165, 105, 109, 97,
            103, 101, 163, 102, 111, 111, 166, 97, 109, 111, 117, 110, 116, 42, 164, 116, 105, 109,
            101, 207, 255, 255, 255, 255, 255, 255, 255, 255, 165, 107, 97, 114, 109, 97, 239,
        ];
        (msg, serialized)
    }

    fn special_chars_test_vector() -> (SomeMsg, &'static [u8]) {
        let msg = SomeMsg::Cowsay {
            text: "foo\"bar\\\"bla".to_string(),
        };
        let serialized = &[
            129, 166, 99, 111, 119, 115, 97, 121, 129, 164, 116, 101, 120, 116, 172, 102, 111, 111,
            34, 98, 97, 114, 92, 34, 98, 108, 97,
        ];
        (msg, serialized)
    }

    #[test]
    fn to_msgpack_vec_works() {
        let (msg, expected) = refund_test_vector();
        let serialized = to_msgpack_vec(&msg).unwrap();
        assert_eq!(serialized, expected);

        let (msg, expected) = release_all_test_vector();
        let serialized = to_msgpack_vec(&msg).unwrap();
        assert_eq!(serialized, expected);
    }

    #[test]
    fn from_msgpack_works() {
        let (msg, serialized) = refund_test_vector();
        let deserialized: SomeMsg = from_msgpack(serialized).unwrap();
        assert_eq!(deserialized, msg);

        let (msg, serialized) = release_all_test_vector();
        let deserialized: SomeMsg = from_msgpack(serialized).unwrap();
        assert_eq!(deserialized, msg);
    }

    #[test]
    fn from_msgpack_or_binary() {
        let msg = SomeMsg::Refund {};
        let serialized: Binary = to_msgpack_binary(&msg).unwrap();

        let parse_binary: SomeMsg = from_msgpack(&serialized).unwrap();
        assert_eq!(parse_binary, msg);

        let parse_slice: SomeMsg = from_msgpack(serialized.as_slice()).unwrap();
        assert_eq!(parse_slice, msg);
    }

    #[test]
    fn from_msgpack_works_for_special_chars() {
        let (msg, serialized) = special_chars_test_vector();
        let deserialized: SomeMsg = from_msgpack(serialized).unwrap();
        assert_eq!(deserialized, msg);
    }

    #[test]
    fn deserialize_modified_field_order() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestV1 {
            a: String,
            b: u32,
            c: u64,
        }

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestV2 {
            b: u32,
            c: u64,
            a: String,
        }

        let v1 = TestV1 {
            a: "foo".to_string(),
            b: 42,
            c: 18446744073709551615,
        };

        let v2: TestV2 = from_msgpack(to_msgpack_vec(&v1).unwrap()).unwrap();
        assert_eq!(
            v2,
            TestV2 {
                b: 42,
                c: 18446744073709551615,
                a: "foo".to_string()
            }
        );
    }

    #[test]
    fn deserialize_new_fields() {
        // fields can be added, but only to the end of the struct

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestV1 {
            a: String,
        }

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestV2 {
            a: String,
            #[serde(default)]
            b: u32,
        }

        let v1 = TestV1 {
            a: "foo".to_string(),
        };
        let v2: TestV2 = from_msgpack(to_msgpack_vec(&v1).unwrap()).unwrap();

        assert_eq!(
            v2,
            TestV2 {
                a: "foo".to_string(),
                b: 0
            }
        );
    }

    #[test]
    fn deserialize_new_fields_in_the_middle() {
        // fields can be added, but only to the end of the struct

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestV1 {
            a: String,
            b: u32,
        }

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestV2 {
            a: String,
            #[serde(default)]
            c: u8,
            b: u32,
        }

        let v1 = TestV1 {
            a: "foo".to_string(),
            b: 999999,
        };
        let v2: TestV2 = from_msgpack(to_msgpack_vec(&v1).unwrap()).unwrap();

        assert_eq!(
            v2,
            TestV2 {
                a: "foo".to_string(),
                c: 0,
                b: 999999,
            }
        );
    }
}
