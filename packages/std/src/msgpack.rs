// This file mostly re-exports some methods from rmp-serde
// The reason is two fold:
// 1. To easily ensure that all calling libraries use the same version (minimize code size)
// 2. To allow us to switch out to another MessagePack library if needed

use serde::{de::DeserializeOwned, Serialize};

use crate::{Binary, StdResult};

/// Deserializes the given MessagePack bytes to a data structure.
///
/// Errors if the input is not valid MessagePack or cannot be deserialized to the given type.
///
/// ## Examples
///
/// Encoding and decoding an enum using MessagePack.
///
/// ```
/// use cosmwasm_schema::cw_serde;
/// use cosmwasm_std::{to_msgpack_binary, from_msgpack};
///
/// #[cw_serde]
/// enum MyPacket {
///     Cowsay {
///         text: String,
///     },
/// }
///
/// let packet = MyPacket::Cowsay { text: "hi".to_string() };
/// let encoded = to_msgpack_binary(&packet).unwrap();
/// let decoded: MyPacket  = from_msgpack(&encoded).unwrap();
/// assert_eq!(decoded, packet);
pub fn from_msgpack<T: DeserializeOwned>(value: impl AsRef<[u8]>) -> StdResult<T> {
    Ok(rmp_serde::from_read(value.as_ref())?)
}

/// Serializes the given data structure as a MessagePack byte vector.
///
/// ## Examples
///
/// Encoding and decoding an enum using MessagePack.
///
/// ```
/// use cosmwasm_schema::cw_serde;
/// use cosmwasm_std::{to_msgpack_vec, from_msgpack};
///
/// #[cw_serde]
/// enum MyPacket {
///     Cowsay {
///         text: String,
///     },
/// }
///
/// let packet = MyPacket::Cowsay { text: "hi".to_string() };
/// let encoded = to_msgpack_vec(&packet).unwrap();
/// let decoded: MyPacket  = from_msgpack(&encoded).unwrap();
/// assert_eq!(decoded, packet);
pub fn to_msgpack_vec<T>(data: &T) -> StdResult<Vec<u8>>
where
    T: Serialize + ?Sized,
{
    Ok(rmp_serde::to_vec_named(data)?)
}

/// Serializes the given data structure as MessagePack bytes.
///
/// ## Examples
///
/// Encoding and decoding an enum using MessagePack.
///
/// ```
/// use cosmwasm_schema::cw_serde;
/// use cosmwasm_std::{to_msgpack_binary, from_msgpack};
///
/// #[cw_serde]
/// enum MyPacket {
///     Cowsay {
///         text: String,
///     },
/// }
///
/// let packet = MyPacket::Cowsay { text: "hi".to_string() };
/// let encoded = to_msgpack_binary(&packet).unwrap();
/// let decoded: MyPacket  = from_msgpack(&encoded).unwrap();
/// assert_eq!(decoded, packet);
/// ```
pub fn to_msgpack_binary<T>(data: &T) -> StdResult<Binary>
where
    T: Serialize + ?Sized,
{
    to_msgpack_vec(data).map(Binary::new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Int128, Int256, Int512, Int64, Uint128, Uint256, Uint512, Uint64};
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
            text: "foo\"bar\\\"bla🦴👁🦶🏻".to_string(),
        };
        let serialized = &[
            129, 166, 99, 111, 119, 115, 97, 121, 129, 164, 116, 101, 120, 116, 188, 102, 111, 111,
            34, 98, 97, 114, 92, 34, 98, 108, 97, 240, 159, 166, 180, 240, 159, 145, 129, 240, 159,
            166, 182, 240, 159, 143, 187,
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
        // field order doesn't matter since we encode field names

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
        // new fields can be added at the end

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
        // fields can be added even in the middle
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

    #[test]
    fn msgpack_serialization_for_boolean_types() {
        // "Bool format family stores false or true in 1 byte."
        let serialized = to_msgpack_vec(&false).unwrap();
        assert_eq!(serialized, [0xc2]);
        let serialized = to_msgpack_vec(&true).unwrap();
        assert_eq!(serialized, [0xc3]);
    }

    #[test]
    fn msgpack_serialization_for_integer_types() {
        // primitive integers up to 64bit
        // similar to VARINT in protobuf or number in JSON, the encoding does not contain integer size
        {
            // "positive fixint stores 7-bit positive integer"
            let serialized = to_msgpack_vec(&0u8).unwrap();
            assert_eq!(serialized, [0]);
            let serialized = to_msgpack_vec(&0u16).unwrap();
            assert_eq!(serialized, [0]);
            let serialized = to_msgpack_vec(&0u32).unwrap();
            assert_eq!(serialized, [0]);
            let serialized = to_msgpack_vec(&0u64).unwrap();
            assert_eq!(serialized, [0]);
            let serialized = to_msgpack_vec(&0i64).unwrap();
            assert_eq!(serialized, [0]);
            let serialized = to_msgpack_vec(&7u8).unwrap();
            assert_eq!(serialized, [7]);
            let serialized = to_msgpack_vec(&7u16).unwrap();
            assert_eq!(serialized, [7]);
            let serialized = to_msgpack_vec(&7u32).unwrap();
            assert_eq!(serialized, [7]);
            let serialized = to_msgpack_vec(&7u64).unwrap();
            assert_eq!(serialized, [7]);
            let serialized = to_msgpack_vec(&127u32).unwrap();
            assert_eq!(serialized, [127]);

            // "negative fixint stores 5-bit negative integer"
            let serialized = to_msgpack_vec(&-1i32).unwrap();
            assert_eq!(serialized, [255]);
            let serialized = to_msgpack_vec(&-1i64).unwrap();
            assert_eq!(serialized, [255]);
            let serialized = to_msgpack_vec(&-10i64).unwrap();
            assert_eq!(serialized, [246]);
            let serialized = to_msgpack_vec(&-24i64).unwrap();
            assert_eq!(serialized, [232]);

            // "uint 8 stores an 8-bit unsigned integer"
            let serialized = to_msgpack_vec(&128u32).unwrap();
            assert_eq!(serialized, [0xcc, 128]);
            let serialized = to_msgpack_vec(&237u32).unwrap();
            assert_eq!(serialized, [0xcc, 237]);

            // "uint 16 stores a 16-bit big-endian unsigned integer"
            let serialized = to_msgpack_vec(&1000u32).unwrap();
            assert_eq!(serialized, [0xcd, 3, 232]);

            // "uint 32 stores a 32-bit big-endian unsigned integer"
            let serialized = to_msgpack_vec(&u32::MAX).unwrap();
            assert_eq!(serialized, [0xce, 255, 255, 255, 255]);

            // "uint 64 stores a 64-bit big-endian unsigned integer"
            let serialized = to_msgpack_vec(&575747839886u64).unwrap();
            assert_eq!(serialized, [0xcf, 0, 0, 0, 134, 13, 62, 215, 142]);
            let serialized = to_msgpack_vec(&u64::MAX).unwrap();
            assert_eq!(serialized, [0xcf, 255, 255, 255, 255, 255, 255, 255, 255]);

            // "int 8 stores an 8-bit signed integer"
            let serialized = to_msgpack_vec(&i8::MIN).unwrap();
            assert_eq!(serialized, [0xd0, 128]);
            let serialized = to_msgpack_vec(&-111i8).unwrap();
            assert_eq!(serialized, [0xd0, 145]);

            // "int 16 stores a 16-bit big-endian signed integer"
            let serialized = to_msgpack_vec(&i16::MIN).unwrap();
            assert_eq!(serialized, [0xd1, 128, 0]);

            // "int 32 stores a 32-bit big-endian signed integer"
            let serialized = to_msgpack_vec(&i32::MIN).unwrap();
            assert_eq!(serialized, [0xd2, 128, 0, 0, 0]);

            // "int 64 stores a 64-bit big-endian signed integer"
            let serialized = to_msgpack_vec(&i64::MIN).unwrap();
            assert_eq!(serialized, [0xd3, 128, 0, 0, 0, 0, 0, 0, 0]);
        }

        // u128/i128
        // cannot be serialized as integers in messagepack due to the limitation
        // "a value of an Integer object is limited from -(2^63) upto (2^64)-1"
        {
            // encoded as 16 bytes big endian
            // i.e. takes 18 bytes of storage
            assert_eq!(
                to_msgpack_vec(&0u128).unwrap(),
                [0xc4, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
            );
            assert_eq!(
                to_msgpack_vec(&1u128).unwrap(),
                [0xc4, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
            );
            assert_eq!(
                to_msgpack_vec(&17u128).unwrap(),
                [0xc4, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 17]
            );
            assert_eq!(
                to_msgpack_vec(&u128::MAX).unwrap(),
                [
                    0xc4, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
                    255, 255
                ]
            );

            assert_eq!(
                to_msgpack_vec(&0i128).unwrap(),
                [0xc4, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
            );
            assert_eq!(
                to_msgpack_vec(&1i128).unwrap(),
                [0xc4, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
            );
            assert_eq!(
                to_msgpack_vec(&17i128).unwrap(),
                [0xc4, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 17]
            );
            assert_eq!(
                to_msgpack_vec(&-1i128).unwrap(),
                [
                    0xc4, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
                    255, 255
                ]
            );
            assert_eq!(
                to_msgpack_vec(&i128::MIN).unwrap(),
                [0xc4, 16, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
            );
            assert_eq!(
                to_msgpack_vec(&i128::MAX).unwrap(),
                [
                    0xc4, 16, 127, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
                    255, 255
                ]
            );
        }

        // Uint64/Uint128/Uint256/Uint512
        {
            let s = to_msgpack_vec(&Uint64::zero()).unwrap();
            assert_eq!(s, [0b10100000 ^ 1, b'0']); // string of lengths 1 with value "0"
            let s = to_msgpack_vec(&Uint128::zero()).unwrap();
            assert_eq!(s, [0b10100000 ^ 1, b'0']); // string of lengths 1 with value "0"
            let s = to_msgpack_vec(&Uint256::zero()).unwrap();
            assert_eq!(s, [0b10100000 ^ 1, b'0']); // string of lengths 1 with value "0"
            let s = to_msgpack_vec(&Uint512::zero()).unwrap();
            assert_eq!(s, [0b10100000 ^ 1, b'0']); // string of lengths 1 with value "0"

            let s = to_msgpack_vec(&Uint64::one()).unwrap();
            assert_eq!(s, [0b10100000 ^ 1, b'1']); // string of lengths 1 with value "1"
            let s = to_msgpack_vec(&Uint128::one()).unwrap();
            assert_eq!(s, [0b10100000 ^ 1, b'1']); // string of lengths 1 with value "1"
            let s = to_msgpack_vec(&Uint256::one()).unwrap();
            assert_eq!(s, [0b10100000 ^ 1, b'1']); // string of lengths 1 with value "1"
            let s = to_msgpack_vec(&Uint512::one()).unwrap();
            assert_eq!(s, [0b10100000 ^ 1, b'1']); // string of lengths 1 with value "1"

            let s = to_msgpack_vec(&Uint64::MAX).unwrap();
            assert_eq!(
                s,
                [
                    0b10100000 ^ 20,
                    b'1',
                    b'8',
                    b'4',
                    b'4',
                    b'6',
                    b'7',
                    b'4',
                    b'4',
                    b'0',
                    b'7',
                    b'3',
                    b'7',
                    b'0',
                    b'9',
                    b'5',
                    b'5',
                    b'1',
                    b'6',
                    b'1',
                    b'5'
                ]
            ); // string of lengths 1 with value "1"
        }

        // Int64/Int128/Int256/Int512
        {
            let s = to_msgpack_vec(&Int64::zero()).unwrap();
            assert_eq!(s, [0b10100000 ^ 1, b'0']); // string of lengths 1 with value "0"
            let s = to_msgpack_vec(&Int128::zero()).unwrap();
            assert_eq!(s, [0b10100000 ^ 1, b'0']); // string of lengths 1 with value "0"
            let s = to_msgpack_vec(&Int256::zero()).unwrap();
            assert_eq!(s, [0b10100000 ^ 1, b'0']); // string of lengths 1 with value "0"
            let s = to_msgpack_vec(&Int512::zero()).unwrap();
            assert_eq!(s, [0b10100000 ^ 1, b'0']); // string of lengths 1 with value "0"

            let s = to_msgpack_vec(&Int64::one()).unwrap();
            assert_eq!(s, [0b10100000 ^ 1, b'1']); // string of lengths 1 with value "1"
            let s = to_msgpack_vec(&Int128::one()).unwrap();
            assert_eq!(s, [0b10100000 ^ 1, b'1']); // string of lengths 1 with value "1"
            let s = to_msgpack_vec(&Int256::one()).unwrap();
            assert_eq!(s, [0b10100000 ^ 1, b'1']); // string of lengths 1 with value "1"
            let s = to_msgpack_vec(&Int512::one()).unwrap();
            assert_eq!(s, [0b10100000 ^ 1, b'1']); // string of lengths 1 with value "1"

            let s = to_msgpack_vec(&Int64::from(15i32)).unwrap();
            assert_eq!(s, [0b10100000 ^ 2, b'1', b'5']); // string of lengths 2 with value "15"
            let s = to_msgpack_vec(&Int128::from(15i32)).unwrap();
            assert_eq!(s, [0b10100000 ^ 2, b'1', b'5']); // string of lengths 2 with value "15"
            let s = to_msgpack_vec(&Int256::from(15i32)).unwrap();
            assert_eq!(s, [0b10100000 ^ 2, b'1', b'5']); // string of lengths 2 with value "15"
            let s = to_msgpack_vec(&Int512::from(15i32)).unwrap();
            assert_eq!(s, [0b10100000 ^ 2, b'1', b'5']); // string of lengths 2 with value "15"

            let s = to_msgpack_vec(&Int64::from(-1i64)).unwrap();
            assert_eq!(s, [0b10100000 ^ 2, b'-', b'1']); // string of lengths 2 with value "-1"
            let s = to_msgpack_vec(&Int128::from(-1i64)).unwrap();
            assert_eq!(s, [0b10100000 ^ 2, b'-', b'1']); // string of lengths 2 with value "-1"
            let s = to_msgpack_vec(&Int256::from(-1i64)).unwrap();
            assert_eq!(s, [0b10100000 ^ 2, b'-', b'1']); // string of lengths 2 with value "-1"
            let s = to_msgpack_vec(&Int512::from(-1i64)).unwrap();
            assert_eq!(s, [0b10100000 ^ 2, b'-', b'1']); // string of lengths 2 with value "-1"
        }
    }
}
