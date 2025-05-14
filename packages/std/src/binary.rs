use alloc::{string::String, vec::Vec};
use core::fmt;
use core::ops::Deref;

use serde::{de, ser, Deserialize, Deserializer, Serialize};

use crate::{
    encoding::{from_base64, to_base64},
    errors::{StdError, StdResult},
};

/// Binary is a wrapper around Vec<u8> to add base64 de/serialization
/// with serde. It also adds some helper methods to help encode inline.
///
/// This is only needed as serde-json-{core,wasm} has a horrible encoding for Vec<u8>.
/// See also <https://github.com/CosmWasm/cosmwasm/blob/main/docs/MESSAGE_TYPES.md>.
#[derive(Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord, schemars::JsonSchema)]
pub struct Binary(#[schemars(with = "String")] Vec<u8>);

impl Binary {
    /// Creates a new `Binary` containing the given data.
    pub const fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    /// take an (untrusted) string and decode it into bytes.
    /// fails if it is not valid base64
    pub fn from_base64(encoded: &str) -> StdResult<Self> {
        from_base64(encoded).map(Self::new)
    }

    /// encode to base64 string (guaranteed to be success as we control the data inside).
    /// this returns normalized form (with trailing = if needed)
    pub fn to_base64(&self) -> String {
        to_base64(&self.0)
    }

    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// Copies content into fixed-sized array.
    ///
    /// # Examples
    ///
    /// Copy to array of explicit length
    ///
    /// ```
    /// # use cosmwasm_std::Binary;
    /// let binary = Binary::from(&[0xfb, 0x1f, 0x37]);
    /// let array: [u8; 3] = binary.to_array().unwrap();
    /// assert_eq!(array, [0xfb, 0x1f, 0x37]);
    /// ```
    ///
    /// Copy to integer
    ///
    /// ```
    /// # use cosmwasm_std::Binary;
    /// let binary = Binary::from(&[0x8b, 0x67, 0x64, 0x84, 0xb5, 0xfb, 0x1f, 0x37]);
    /// let num = u64::from_be_bytes(binary.to_array().unwrap());
    /// assert_eq!(num, 10045108015024774967);
    /// ```
    pub fn to_array<const LENGTH: usize>(&self) -> StdResult<[u8; LENGTH]> {
        if self.len() != LENGTH {
            return Err(StdError::invalid_data_size(LENGTH, self.len()));
        }

        let mut out: [u8; LENGTH] = [0; LENGTH];
        out.copy_from_slice(&self.0);
        Ok(out)
    }

    pub fn from_hex(input: &str) -> StdResult<Self> {
        let binary = hex::decode(input).map_err(StdError::invalid_hex)?;
        Ok(Self(binary))
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }
}

impl fmt::Display for Binary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_base64())
    }
}

impl fmt::Debug for Binary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use an output inspired by tuples (https://doc.rust-lang.org/std/fmt/struct.Formatter.html#method.debug_tuple)
        // but with a custom implementation to avoid the need for an intermediate hex string.
        write!(f, "Binary(")?;
        for byte in self.0.iter() {
            write!(f, "{byte:02x}")?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

/// Just like Vec<u8>, Binary is a smart pointer to [u8].
/// This implements `*binary` for us and allows us to
/// do `&*binary`, returning a `&[u8]` from a `&Binary`.
/// With [deref coercions](https://doc.rust-lang.org/1.22.1/book/first-edition/deref-coercions.html#deref-coercions),
/// this allows us to use `&binary` whenever a `&[u8]` is required.
impl Deref for Binary {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl AsRef<[u8]> for Binary {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

// Slice
impl From<&[u8]> for Binary {
    fn from(binary: &[u8]) -> Self {
        Self(binary.to_vec())
    }
}

// Array reference
impl<const LENGTH: usize> From<&[u8; LENGTH]> for Binary {
    fn from(source: &[u8; LENGTH]) -> Self {
        Self(source.to_vec())
    }
}

// Owned array
impl<const LENGTH: usize> From<[u8; LENGTH]> for Binary {
    fn from(source: [u8; LENGTH]) -> Self {
        Self(source.into())
    }
}

impl From<Vec<u8>> for Binary {
    fn from(vec: Vec<u8>) -> Self {
        Self(vec)
    }
}

impl From<Binary> for Vec<u8> {
    fn from(original: Binary) -> Vec<u8> {
        original.0
    }
}

/// Implement `encoding::Binary == alloc::vec::Vec<u8>`
impl PartialEq<Vec<u8>> for Binary {
    fn eq(&self, rhs: &Vec<u8>) -> bool {
        // Use Vec<u8> == Vec<u8>
        self.0 == *rhs
    }
}

/// Implement `alloc::vec::Vec<u8> == encoding::Binary`
impl PartialEq<Binary> for Vec<u8> {
    fn eq(&self, rhs: &Binary) -> bool {
        // Use Vec<u8> == Vec<u8>
        *self == rhs.0
    }
}

/// Implement `Binary == &[u8]`
impl PartialEq<&[u8]> for Binary {
    fn eq(&self, rhs: &&[u8]) -> bool {
        // Use &[u8] == &[u8]
        self.as_slice() == *rhs
    }
}

/// Implement `&[u8] == Binary`
impl PartialEq<Binary> for &[u8] {
    fn eq(&self, rhs: &Binary) -> bool {
        // Use &[u8] == &[u8]
        *self == rhs.as_slice()
    }
}

/// Implement `Binary == &[u8; LENGTH]`
impl<const LENGTH: usize> PartialEq<&[u8; LENGTH]> for Binary {
    fn eq(&self, rhs: &&[u8; LENGTH]) -> bool {
        self.as_slice() == rhs.as_slice()
    }
}

/// Implement `&[u8; LENGTH] == Binary`
impl<const LENGTH: usize> PartialEq<Binary> for &[u8; LENGTH] {
    fn eq(&self, rhs: &Binary) -> bool {
        self.as_slice() == rhs.as_slice()
    }
}

/// Implement `Binary == [u8; LENGTH]`
impl<const LENGTH: usize> PartialEq<[u8; LENGTH]> for Binary {
    fn eq(&self, rhs: &[u8; LENGTH]) -> bool {
        self.as_slice() == rhs.as_slice()
    }
}

/// Implement `[u8; LENGTH] == Binary`
impl<const LENGTH: usize> PartialEq<Binary> for [u8; LENGTH] {
    fn eq(&self, rhs: &Binary) -> bool {
        self.as_slice() == rhs.as_slice()
    }
}

/// Serializes as a base64 string
impl Serialize for Binary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_base64())
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

/// Deserializes as a base64 string
impl<'de> Deserialize<'de> for Binary {
    fn deserialize<D>(deserializer: D) -> Result<Binary, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(Base64Visitor)
        } else {
            deserializer.deserialize_bytes(BytesVisitor)
        }
    }
}

struct Base64Visitor;

impl de::Visitor<'_> for Base64Visitor {
    type Value = Binary;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("valid base64 encoded string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match Binary::from_base64(v) {
            Ok(binary) => Ok(binary),
            Err(_) => Err(E::custom(format_args!("invalid base64: {v}"))),
        }
    }
}

struct BytesVisitor;

impl de::Visitor<'_> for BytesVisitor {
    type Value = Binary;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Binary(v.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_hash_works;
    use crate::errors::StdError;

    #[test]
    fn to_array_works() {
        // simple
        let binary = Binary::from(&[1, 2, 3]);
        let array: [u8; 3] = binary.to_array().unwrap();
        assert_eq!(array, [1, 2, 3]);

        // empty
        let binary = Binary::from(&[]);
        let array: [u8; 0] = binary.to_array().unwrap();
        assert_eq!(array, [] as [u8; 0]);

        // invalid size
        let binary = Binary::from(&[1, 2, 3]);
        let error = binary.to_array::<8>().unwrap_err();
        match error {
            StdError::InvalidDataSize {
                expected, actual, ..
            } => {
                assert_eq!(expected, 8);
                assert_eq!(actual, 3);
            }
            err => panic!("Unexpected error: {err:?}"),
        }

        // long array (32 bytes)
        let binary = Binary::from_base64("t119JOQox4WUQEmO/nyqOZfO+wjJm91YG2sfn4ZglvA=").unwrap();
        let array: [u8; 32] = binary.to_array().unwrap();
        assert_eq!(
            array,
            [
                0xb7, 0x5d, 0x7d, 0x24, 0xe4, 0x28, 0xc7, 0x85, 0x94, 0x40, 0x49, 0x8e, 0xfe, 0x7c,
                0xaa, 0x39, 0x97, 0xce, 0xfb, 0x08, 0xc9, 0x9b, 0xdd, 0x58, 0x1b, 0x6b, 0x1f, 0x9f,
                0x86, 0x60, 0x96, 0xf0,
            ]
        );

        // very long array > 32 bytes (requires Rust 1.47+)
        let binary =
            Binary::from_base64("t119JOQox4WUQEmO/nyqOZfO+wjJm91YG2sfn4ZglvBzyMOwMWq+").unwrap();
        let array: [u8; 39] = binary.to_array().unwrap();
        assert_eq!(
            array,
            [
                0xb7, 0x5d, 0x7d, 0x24, 0xe4, 0x28, 0xc7, 0x85, 0x94, 0x40, 0x49, 0x8e, 0xfe, 0x7c,
                0xaa, 0x39, 0x97, 0xce, 0xfb, 0x08, 0xc9, 0x9b, 0xdd, 0x58, 0x1b, 0x6b, 0x1f, 0x9f,
                0x86, 0x60, 0x96, 0xf0, 0x73, 0xc8, 0xc3, 0xb0, 0x31, 0x6a, 0xbe,
            ]
        );
    }

    #[test]
    fn test_base64_encoding_success() {
        for (value, encoded, encoded_no_pad) in [
            (&b""[..], "", ""),
            (&b"hello"[..], "aGVsbG8=", "aGVsbG8"),
            (&b"\x0C\xBB\x00\x11\xFA\x01"[..], "DLsAEfoB", "DLsAEfoB"),
            (&b"rand"[..], "cmFuZA==", "cmFuZA"),
            (&b"rand"[..], "cmFuZA==", "cmFuZA="),
            (&b"randomiZ"[..], "cmFuZG9taVo=", "cmFuZG9taVo"),
        ] {
            let value = Binary::from(value);
            assert_eq!(encoded, value.to_base64());
            assert_eq!(Ok(value.clone()), Binary::from_base64(encoded));
            assert_eq!(Ok(value.clone()), Binary::from_base64(encoded_no_pad));
        }
    }

    #[test]
    fn test_base64_encoding_error() {
        for (invalid_base64, want) in [
            ("cm%uZG9taVo", "Invalid symbol 37, offset 2."),
            ("cmFuZ", "Invalid input length: 5"),
        ] {
            match Binary::from_base64(invalid_base64) {
                Err(StdError::InvalidBase64 { msg, .. }) => assert_eq!(want, msg),
                result => panic!("Unexpected result: {result:?}"),
            }
        }
    }

    #[test]
    fn from_slice_works() {
        let original: &[u8] = &[0u8, 187, 61, 11, 250, 0];
        let binary: Binary = original.into();
        assert_eq!(binary.as_slice(), [0u8, 187, 61, 11, 250, 0]);
    }

    #[test]
    fn from_fixed_length_array_works() {
        let original = &[];
        let binary: Binary = original.into();
        assert_eq!(binary.len(), 0);

        let original = &[0u8];
        let binary: Binary = original.into();
        assert_eq!(binary.as_slice(), [0u8]);

        let original = &[0u8, 187, 61, 11, 250, 0];
        let binary: Binary = original.into();
        assert_eq!(binary.as_slice(), [0u8, 187, 61, 11, 250, 0]);

        let original = &[
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1,
        ];
        let binary: Binary = original.into();
        assert_eq!(
            binary.as_slice(),
            [
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1,
            ]
        );
    }

    #[test]
    fn from_owned_fixed_length_array_works() {
        let original = [];
        let binary: Binary = original.into();
        assert_eq!(binary.len(), 0);

        let original = [0u8];
        let binary: Binary = original.into();
        assert_eq!(binary.as_slice(), [0u8]);

        let original = [0u8, 187, 61, 11, 250, 0];
        let binary: Binary = original.into();
        assert_eq!(binary.as_slice(), [0u8, 187, 61, 11, 250, 0]);

        let original = [
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1,
        ];
        let binary: Binary = original.into();
        assert_eq!(
            binary.as_slice(),
            [
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1,
            ]
        );
    }

    #[test]
    fn from_literal_works() {
        let a: Binary = b"".into();
        assert_eq!(a.len(), 0);

        let a: Binary = b".".into();
        assert_eq!(a.len(), 1);

        let a: Binary = b"...".into();
        assert_eq!(a.len(), 3);

        let a: Binary = b"...............................".into();
        assert_eq!(a.len(), 31);

        let a: Binary = b"................................".into();
        assert_eq!(a.len(), 32);

        let a: Binary = b".................................".into();
        assert_eq!(a.len(), 33);
    }

    #[test]
    fn from_vec_works() {
        let original = vec![0u8, 187, 61, 11, 250, 0];
        let original_ptr = original.as_ptr();
        let binary: Binary = original.into();
        assert_eq!(binary.as_slice(), [0u8, 187, 61, 11, 250, 0]);
        assert_eq!(binary.0.as_ptr(), original_ptr, "vector must not be copied");
    }

    #[test]
    fn into_vec_works() {
        // Into<Vec<u8>> for Binary
        let original = Binary(vec![0u8, 187, 61, 11, 250, 0]);
        let original_ptr = original.0.as_ptr();
        let vec: Vec<u8> = original.into();
        assert_eq!(vec.as_slice(), [0u8, 187, 61, 11, 250, 0]);
        assert_eq!(vec.as_ptr(), original_ptr, "vector must not be copied");

        // From<Binary> for Vec<u8>
        let original = Binary(vec![7u8, 35, 49, 101, 0, 255]);
        let original_ptr = original.0.as_ptr();
        let vec = Vec::<u8>::from(original);
        assert_eq!(vec.as_slice(), [7u8, 35, 49, 101, 0, 255]);
        assert_eq!(vec.as_ptr(), original_ptr, "vector must not be copied");
    }

    #[test]
    fn json_serialization_works() {
        let binary = Binary(vec![0u8, 187, 61, 11, 250, 0]);

        let json = serde_json::to_vec(&binary).unwrap();
        let deserialized: Binary = serde_json::from_slice(&json).unwrap();

        assert_eq!(binary, deserialized);
    }

    #[test]
    fn json_deserialize_from_valid_string() {
        let b64_str = "ALs9C/oA";
        // this is the binary behind above string
        let expected = vec![0u8, 187, 61, 11, 250, 0];

        let serialized = serde_json::to_vec(&b64_str).unwrap();
        let deserialized: Binary = serde_json::from_slice(&serialized).unwrap();
        assert_eq!(expected, deserialized.as_slice());
    }

    #[test]
    fn json_deserialize_from_invalid_string() {
        let invalid_str = "**BAD!**";
        let serialized = serde_json::to_vec(&invalid_str).unwrap();
        let res = serde_json::from_slice::<Binary>(&serialized);
        assert!(res.is_err());
    }

    #[test]
    fn msgpack_serialization_works() {
        let data = Binary(vec![0u8, 187, 61, 11, 250, 0]);
        // see: https://github.com/msgpack/msgpack/blob/8aa09e2/spec.md#bin-format-family
        let expected = [196, 6, 0, 187, 61, 11, 250, 0];

        assert_eq!(rmp_serde::to_vec(&data).unwrap(), expected);
    }

    #[test]
    fn msgpack_deserialize_from_valid_data() {
        // see: https://github.com/msgpack/msgpack/blob/8aa09e2/spec.md#bin-format-family
        let serialized = vec![196, 6, 0, 187, 61, 11, 250, 0];
        let expected = vec![0u8, 187, 61, 11, 250, 0];

        let deserialized: Binary = rmp_serde::from_slice(&serialized).unwrap();
        assert_eq!(expected, deserialized.as_slice());
    }

    #[test]
    fn msgpack_deserialize_from_invalid_data() {
        let invalid_data = vec![0, 1, 2, 3, 4, 5];
        let res = rmp_serde::from_slice::<Binary>(&invalid_data);
        assert!(res.is_err());
    }

    #[test]
    fn binary_implements_debug() {
        // Some data
        let binary = Binary(vec![0x07, 0x35, 0xAA, 0xcb, 0x00, 0xff]);
        assert_eq!(format!("{binary:?}"), "Binary(0735aacb00ff)",);

        // Empty
        let binary = Binary(vec![]);
        assert_eq!(format!("{binary:?}"), "Binary()",);
    }

    #[test]
    fn binary_implements_deref() {
        // Dereference to [u8]
        let binary = Binary(vec![7u8, 35, 49, 101, 0, 255]);
        assert_eq!(*binary, [7u8, 35, 49, 101, 0, 255]);

        // This checks deref coercions from &Binary to &[u8] works
        let binary = Binary(vec![7u8, 35, 49, 101, 0, 255]);
        assert_eq!(binary.len(), 6);
        let binary_slice: &[u8] = &binary;
        assert_eq!(binary_slice, &[7u8, 35, 49, 101, 0, 255]);
    }

    #[test]
    fn binary_implements_as_ref() {
        let want = &[7u8, 35, 49, 101, 0, 255];
        let data = Binary(want.to_vec());
        assert_eq!(want, AsRef::<[u8]>::as_ref(&data));
        assert_eq!(want, AsRef::<[u8]>::as_ref(&&data));
    }

    /// Tests that `Binary` implements `EQ` and `Hash` correctly and thus can be
    /// used with hash maps and sets.
    #[test]
    fn binary_implements_hash_eq() {
        let a = Binary::from([0, 187, 61, 11, 250, 0]);
        let b = Binary::from([16, 21, 33, 0, 255, 9]);
        assert_hash_works!(a, b);
    }

    #[test]
    fn binary_implements_partial_eq_with_vector() {
        let a = Binary(vec![5u8; 3]);
        let b = vec![5u8; 3];
        let c = vec![9u8; 3];
        assert_eq!(a, b);
        assert_eq!(b, a);
        assert_ne!(a, c);
        assert_ne!(c, a);
    }

    #[test]
    fn binary_implements_partial_eq_with_slice_and_array() {
        let a = Binary(vec![0xAA, 0xBB]);

        // Slice: &[u8]
        assert_eq!(a, b"\xAA\xBB" as &[u8]);
        assert_eq!(b"\xAA\xBB" as &[u8], a);
        assert_ne!(a, b"\x11\x22" as &[u8]);
        assert_ne!(b"\x11\x22" as &[u8], a);

        // Array reference: &[u8; 2]
        assert_eq!(a, b"\xAA\xBB");
        assert_eq!(b"\xAA\xBB", a);
        assert_ne!(a, b"\x11\x22");
        assert_ne!(b"\x11\x22", a);

        // Array: [u8; 2]
        assert_eq!(a, [0xAA, 0xBB]);
        assert_eq!([0xAA, 0xBB], a);
        assert_ne!(a, [0x11, 0x22]);
        assert_ne!([0x11, 0x22], a);
    }
}
