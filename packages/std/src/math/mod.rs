mod conversion;
mod decimal;
mod decimal256;
mod fraction;
mod int128;
mod int256;
mod int512;
mod int64;
mod isqrt;
mod num_consts;
mod signed_decimal;
mod signed_decimal_256;
mod uint128;
mod uint256;
mod uint512;
mod uint64;

pub use decimal::{Decimal, DecimalRangeExceeded};
pub use decimal256::{Decimal256, Decimal256RangeExceeded};
pub use fraction::Fraction;
pub use int128::Int128;
pub use int256::Int256;
pub use int512::Int512;
pub use int64::Int64;
pub use isqrt::Isqrt;
pub use signed_decimal::{SignedDecimal, SignedDecimalRangeExceeded};
pub use signed_decimal_256::{SignedDecimal256, SignedDecimal256RangeExceeded};
pub use uint128::Uint128;
pub use uint256::Uint256;
pub use uint512::Uint512;
pub use uint64::Uint64;

macro_rules! impl_int_serde {
    ($ty:ty) => {
        impl ::serde::Serialize for $ty {
            /// Serializes as an integer string using base 10.
            ///
            /// We consistently serialize all `UintXXX` and `IntYYY` types as strings in JSON
            /// to ensure the best possible compatibility with clients. E.g. JavaScript and jq
            /// only support up to ~53bit numbers without losing precision, making it hard to use
            /// serialized `u64`s on other systems than Rust or Go. `Uint64`/`Int64` ensure the full
            /// 64 bit range is supported. For larger integers, the use of strings is pretty much the
            /// only reasonable way to store them in JSON.
            ///
            /// For binary encodings (notably MessagePack) strings are used too. The reason is that
            /// in MessagePack integers are limited to 64 bit and we strive for consistent encoding
            /// within the `UintXXX`/`IntYYY` family. Also for small to mid sized values, decimal strings
            /// are often more compact than a fixed-length binary encoding.
            ///
            /// ## Examples
            ///
            /// Serialize to JSON:
            ///
            /// ```
            /// # use cosmwasm_std::{to_json_vec, Uint64};
            /// let value = Uint64::new(17);
            /// let serialized = to_json_vec(&value).unwrap();
            /// assert_eq!(serialized, b"\"17\"");
            /// ```
            ///
            /// Serialize to MessagePack:
            ///
            /// ```
            /// # use cosmwasm_std::{to_msgpack_vec, Uint64};
            /// let value = Uint64::new(17);
            /// let serialized = to_msgpack_vec(&value).unwrap();
            /// assert_eq!(serialized, [0b10100000 ^ 2, b'1', b'7']); // string of lengths 2 with value "17"
            /// ```
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::ser::Serializer,
            {
                serializer.serialize_str(&self.to_string())
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $ty {
            /// Deserializes from an integer string using base 10.
            ///
            /// See the [`Serialize` documentation](#method.serialize) for a few more words
            /// on the encoding of the `UintXXX`/`IntYYY` family.
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::de::Deserializer<'de>,
            {
                struct IntVisitor;

                impl<'de> ::serde::de::Visitor<'de> for IntVisitor {
                    type Value = $ty;

                    fn expecting(
                        &self,
                        formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        formatter.write_str("string-encoded integer")
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: ::serde::de::Error,
                    {
                        <_>::try_from(v).map_err(|e| {
                            E::custom(format_args!("invalid {} '{v}' - {e}", stringify!($t)))
                        })
                    }
                }

                deserializer.deserialize_str(IntVisitor)
            }
        }
    };
}
use impl_int_serde;

#[cfg(test)]
mod tests {
    use super::*;
    use core::ops::*;

    /// A trait that ensures other traits are implemented for our number types
    trait AllImpl<'a>:
        Add
        + Add<&'a Self>
        + AddAssign
        + AddAssign<&'a Self>
        + Sub
        + Sub<&'a Self>
        + SubAssign
        + SubAssign<&'a Self>
        + Mul
        + Mul<&'a Self>
        + MulAssign
        + MulAssign<&'a Self>
        + Div
        + Div<&'a Self>
        + DivAssign
        + DivAssign<&'a Self>
        + Rem
        + Rem<&'a Self>
        + RemAssign
        + RemAssign<&'a Self>
        + Sized
        + Copy
    where
        Self: 'a,
    {
    }

    /// A trait that ensures other traits are implemented for our integer types
    trait IntImpl<'a>:
        AllImpl<'a>
        + Shl<u32>
        + Shl<&'a u32>
        + ShlAssign<u32>
        + ShlAssign<&'a u32>
        + Shr<u32>
        + Shr<&'a u32>
        + ShrAssign<u32>
        + ShrAssign<&'a u32>
        + Not<Output = Self>
        + super::num_consts::NumConsts
    {
    }

    #[allow(dead_code)] // This is used to statically ensure all the integers have a shared set of traits
    trait SignedImpl<'a>: IntImpl<'a> + Neg<Output = Self> {}

    impl AllImpl<'_> for Uint64 {}
    impl AllImpl<'_> for Uint128 {}
    impl AllImpl<'_> for Uint256 {}
    impl AllImpl<'_> for Uint512 {}
    impl AllImpl<'_> for Int64 {}
    impl AllImpl<'_> for Int128 {}
    impl AllImpl<'_> for Int256 {}
    impl AllImpl<'_> for Int512 {}

    impl IntImpl<'_> for Int64 {}
    impl IntImpl<'_> for Int128 {}
    impl IntImpl<'_> for Int256 {}
    impl IntImpl<'_> for Int512 {}
    impl IntImpl<'_> for Uint64 {}
    impl IntImpl<'_> for Uint128 {}
    impl IntImpl<'_> for Uint256 {}
    impl IntImpl<'_> for Uint512 {}

    impl AllImpl<'_> for Decimal {}
    impl AllImpl<'_> for Decimal256 {}

    impl SignedImpl<'_> for Int64 {}
    impl SignedImpl<'_> for Int128 {}
    impl SignedImpl<'_> for Int256 {}
    impl SignedImpl<'_> for Int512 {}
}
