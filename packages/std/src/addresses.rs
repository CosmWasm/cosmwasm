use alloc::{borrow::Cow, string::String, vec::Vec};
use core::fmt;
use core::ops::Deref;
use serde::{Deserialize, Serialize};
use sha2::{
    digest::{Digest, Update},
    Sha256,
};

use crate::Binary;
use crate::{HexBinary, __internal::forward_ref_partial_eq};

/// A human readable address.
///
/// In Cosmos, this is typically bech32 encoded. But for multi-chain smart contracts no
/// assumptions should be made other than being UTF-8 encoded and of reasonable length.
///
/// This type represents a validated address. It can be created in the following ways
/// 1. Use `Addr::unchecked(input)`
/// 2. Use `let checked: Addr = deps.api.addr_validate(input)?`
/// 3. Use `let checked: Addr = deps.api.addr_humanize(canonical_addr)?`
/// 4. Deserialize from JSON. This must only be done from JSON that was validated before
///    such as a contract's state. `Addr` must not be used in messages sent by the user
///    because this would result in unvalidated instances.
///
/// This type is immutable. If you really need to mutate it (Really? Are you sure?), create
/// a mutable copy using `let mut mutable = Addr::to_string()` and operate on that `String`
/// instance.
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    schemars::JsonSchema,
    cw_schema::Schemaifier,
)]
#[schemaifier(type = cw_schema::NodeType::Address)]
pub struct Addr(String);

forward_ref_partial_eq!(Addr, Addr);

impl Addr {
    /// Creates a new `Addr` instance from the given input without checking the validity
    /// of the input. Since `Addr` must always contain valid addresses, the caller is
    /// responsible for ensuring the input is valid.
    ///
    /// Use this in cases where the address was validated before or in test code.
    /// If you see this in contract code, it should most likely be replaced with
    /// `let checked: Addr = deps.api.addr_humanize(canonical_addr)?`.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use cosmwasm_std::{Addr};
    /// let address = Addr::unchecked("foobar");
    /// assert_eq!(address.as_str(), "foobar");
    /// ```
    pub fn unchecked(input: impl Into<String>) -> Addr {
        Addr(input.into())
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Returns the UTF-8 encoded address string as a byte array.
    ///
    /// This is equivalent to `address.as_str().as_bytes()`.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Utility for explicit conversion to `String`.
    #[inline]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl AsRef<str> for Addr {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

// Addr->String is a safe conversion.
// However, the opposite direction is unsafe and must not be implemented.

impl From<Addr> for String {
    fn from(addr: Addr) -> Self {
        addr.0
    }
}

impl From<&Addr> for String {
    fn from(addr: &Addr) -> Self {
        addr.0.clone()
    }
}

impl From<Addr> for Cow<'_, Addr> {
    fn from(addr: Addr) -> Self {
        Cow::Owned(addr)
    }
}

impl<'a> From<&'a Addr> for Cow<'a, Addr> {
    fn from(addr: &'a Addr) -> Self {
        Cow::Borrowed(addr)
    }
}

/// A blockchain address in its binary form.
///
/// The specific implementation is up to the underlying chain and CosmWasm as well as
/// contracts should not make assumptions on that data. In Ethereum for example, an
/// `Addr` would contain a user visible address like 0x14d3cc818735723ab86eaf9502376e847a64ddad
/// and the corresponding `CanonicalAddr` would store the 20 bytes 0x14, 0xD3, ..., 0xAD.
/// In Cosmos, the bech32 format is used for `Addr`s and the `CanonicalAddr` holds the
/// encoded bech32 data without the checksum. Typical sizes are 20 bytes for externally
/// owned addresses and 32 bytes for module addresses (such as x/wasm contract addresses).
/// That being said, a chain might decide to use any size other than 20 or 32 bytes.
///
/// The safe way to obtain a valid `CanonicalAddr` is using `Api::addr_canonicalize`. In
/// addition to that there are many unsafe ways to convert any binary data into an instance.
/// So the type should be treated as a marker to express the intended data type, not as
/// a validity guarantee of any sort.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, schemars::JsonSchema)]
pub struct CanonicalAddr(Binary);

/// Implement `CanonicalAddr == Binary`
impl PartialEq<Binary> for CanonicalAddr {
    fn eq(&self, rhs: &Binary) -> bool {
        &self.0 == rhs
    }
}

/// Implement `Binary == CanonicalAddr`
impl PartialEq<CanonicalAddr> for Binary {
    fn eq(&self, rhs: &CanonicalAddr) -> bool {
        self == &rhs.0
    }
}

/// Implement `CanonicalAddr == HexBinary`
impl PartialEq<HexBinary> for CanonicalAddr {
    fn eq(&self, rhs: &HexBinary) -> bool {
        self.as_slice() == rhs.as_slice()
    }
}

/// Implement `HexBinary == CanonicalAddr`
impl PartialEq<CanonicalAddr> for HexBinary {
    fn eq(&self, rhs: &CanonicalAddr) -> bool {
        self.as_slice() == rhs.0.as_slice()
    }
}

impl From<&[u8]> for CanonicalAddr {
    fn from(source: &[u8]) -> Self {
        Self(source.into())
    }
}

// Array reference
impl<const LENGTH: usize> From<&[u8; LENGTH]> for CanonicalAddr {
    fn from(source: &[u8; LENGTH]) -> Self {
        Self(source.into())
    }
}

// Owned array
impl<const LENGTH: usize> From<[u8; LENGTH]> for CanonicalAddr {
    fn from(source: [u8; LENGTH]) -> Self {
        Self(source.into())
    }
}

// Owned vector -> CanonicalAddr
impl From<Vec<u8>> for CanonicalAddr {
    fn from(source: Vec<u8>) -> Self {
        Self(source.into())
    }
}

// CanonicalAddr -> Owned vector
impl From<CanonicalAddr> for Vec<u8> {
    fn from(source: CanonicalAddr) -> Vec<u8> {
        source.0.into()
    }
}

// Owned Binary -> CanonicalAddr
impl From<Binary> for CanonicalAddr {
    fn from(source: Binary) -> Self {
        Self(source)
    }
}

// CanonicalAddr -> Owned Binary
impl From<CanonicalAddr> for Binary {
    fn from(source: CanonicalAddr) -> Binary {
        source.0
    }
}

// Owned HexBinary -> CanonicalAddr
impl From<HexBinary> for CanonicalAddr {
    fn from(source: HexBinary) -> Self {
        Self(source.into())
    }
}

// CanonicalAddr -> Owned HexBinary
impl From<CanonicalAddr> for HexBinary {
    fn from(source: CanonicalAddr) -> HexBinary {
        source.0.into()
    }
}

/// Just like Vec<u8>, CanonicalAddr is a smart pointer to [u8].
/// This implements `*canonical_address` for us and allows us to
/// do `&*canonical_address`, returning a `&[u8]` from a `&CanonicalAddr`.
/// With [deref coercions](https://doc.rust-lang.org/1.22.1/book/first-edition/deref-coercions.html#deref-coercions),
/// this allows us to use `&canonical_address` whenever a `&[u8]` is required.
impl Deref for CanonicalAddr {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl CanonicalAddr {
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl fmt::Display for CanonicalAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for byte in self.0.as_slice() {
            write!(f, "{byte:02X}")?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Instantiate2AddressError {
    /// Checksum must be 32 bytes
    InvalidChecksumLength,
    /// Salt must be between 1 and 64 bytes
    InvalidSaltLength,
}

impl fmt::Display for Instantiate2AddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Instantiate2AddressError::InvalidChecksumLength => write!(f, "invalid checksum length"),
            Instantiate2AddressError::InvalidSaltLength => write!(f, "invalid salt length"),
        }
    }
}

/// Creates a contract address using the predictable address format introduced with
/// wasmd 0.29. When using instantiate2, this is a way to precompute the address.
/// When using instantiate, the contract address will use a different algorithm and
/// cannot be pre-computed as it contains inputs from the chain's state at the time of
/// message execution.
///
/// The predictable address format of instantiate2 is stable. But bear in mind this is
/// a powerful tool that requires multiple software components to work together smoothly.
/// It should be used carefully and tested thoroughly to avoid the loss of funds.
///
/// This method operates on [`CanonicalAddr`] to be implemented without chain interaction.
/// The typical usage looks like this:
///
/// ```
/// # use cosmwasm_std::{
/// #     HexBinary,
/// #     Storage, Api, Querier, DepsMut, Deps, entry_point, Env, StdError, MessageInfo,
/// #     Response, QueryResponse,
/// # };
/// # type ExecuteMsg = ();
/// use cosmwasm_std::instantiate2_address;
///
/// #[entry_point]
/// pub fn execute(
///     deps: DepsMut,
///     env: Env,
///     info: MessageInfo,
///     msg: ExecuteMsg,
/// ) -> Result<Response, StdError> {
///     let canonical_creator = deps.api.addr_canonicalize(env.contract.address.as_str())?;
///     let checksum = HexBinary::from_hex("9af782a3a1bcbcd22dbb6a45c751551d9af782a3a1bcbcd22dbb6a45c751551d")?;
///     let salt = b"instance 1231";
///     let canonical_addr = instantiate2_address(&checksum, &canonical_creator, salt)
///         .map_err(|_| StdError::generic_err("Could not calculate addr"))?;
///     let addr = deps.api.addr_humanize(&canonical_addr)?;
///
/// #   Ok(Default::default())
/// }
/// ```
pub fn instantiate2_address(
    checksum: &[u8],
    creator: &CanonicalAddr,
    salt: &[u8],
) -> Result<CanonicalAddr, Instantiate2AddressError> {
    // Non-empty msg values are discouraged.
    // See https://medium.com/cosmwasm/dev-note-3-limitations-of-instantiate2-and-how-to-deal-with-them-a3f946874230.
    let msg = b"";
    let len = 32;
    instantiate2_address_impl(checksum, creator, salt, msg, len)
}

/// The instantiate2 address derivation implementation. This API is used for
/// testing purposes only. The `msg` field is discouraged and should not be used.
/// Use [`instantiate2_address`].
///
/// `len` is the address length on bytes. The resulting address data will be truncated to
/// that length. A value > 32 is invalid because [`hash`] returns only 32 bytes of data.
/// A value of 0 is considered invalid because it indicates a bug.
/// For ADR-028 compatibility, 32 must be used.
/// However, some chains use 20 for compatibility with the Ethereum ecosystem.
/// Using any other value than 32 requires a coordination with the chain implementation.
/// See also <https://github.com/CosmWasm/cosmwasm/issues/2155>.
///
/// ## Examples
///
/// ```
/// use cosmwasm_std::{instantiate2_address_impl, CanonicalAddr, HexBinary, Instantiate2AddressError};
///
/// fn instantiate2_address_evm_compatible(
///    checksum: &[u8],
///    creator: &CanonicalAddr,
///    salt: &[u8],
/// ) -> Result<CanonicalAddr, Instantiate2AddressError> {
///     instantiate2_address_impl(checksum, creator, salt, b"", 20)
/// }
///
/// let checksum = HexBinary::from_hex("13a1fc994cc6d1c81b746ee0c0ff6f90043875e0bf1d9be6b7d779fc978dc2a5").unwrap();
/// let creator = CanonicalAddr::from(HexBinary::from_hex("9999999999aaaaaaaaaabbbbbbbbbbcccccccccc").unwrap());
/// let salt = b"\x61";
///
/// let address = instantiate2_address_evm_compatible(&checksum, &creator, salt).unwrap();
/// assert_eq!(address, HexBinary::from_hex("5e865d3e45ad3e961f77fd77d46543417ced44d9").unwrap());
/// ```
#[doc(hidden)]
#[inline] // Only call this through a wrapper like instantiate2_address or a custom instantiate2_address_evm_compatible
pub fn instantiate2_address_impl(
    checksum: &[u8],
    creator: &CanonicalAddr,
    salt: &[u8],
    msg: &[u8],
    len: usize,
) -> Result<CanonicalAddr, Instantiate2AddressError> {
    if checksum.len() != 32 {
        return Err(Instantiate2AddressError::InvalidChecksumLength);
    }

    if salt.is_empty() || salt.len() > 64 {
        return Err(Instantiate2AddressError::InvalidSaltLength);
    };

    let mut key = Vec::<u8>::new();
    key.extend_from_slice(b"wasm\0");
    key.extend_from_slice(&(checksum.len() as u64).to_be_bytes());
    key.extend_from_slice(checksum);
    key.extend_from_slice(&(creator.len() as u64).to_be_bytes());
    key.extend_from_slice(creator);
    key.extend_from_slice(&(salt.len() as u64).to_be_bytes());
    key.extend_from_slice(salt);
    key.extend_from_slice(&(msg.len() as u64).to_be_bytes());
    key.extend_from_slice(msg);
    let mut address_data = hash("module", &key);

    // Use the first `len` bytes
    // Fingers crossed Rust can optimize this whole block out in the default case (32), because otherwise
    // truncate will do a resize for len == address_data.len(), see https://github.com/rust-lang/rust/issues/76089
    if len != 32 {
        debug_assert!(len <= 32);
        debug_assert!(len > 0);
        address_data.truncate(len);
    }

    Ok(address_data.into())
}

/// The "Basic Address" Hash from
/// https://github.com/cosmos/cosmos-sdk/blob/v0.45.8/docs/architecture/adr-028-public-key-addresses.md
fn hash(ty: &str, key: &[u8]) -> Vec<u8> {
    let inner = Sha256::digest(ty.as_bytes());
    Sha256::new().chain(inner).chain(key).finalize().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_hash_works;
    use crate::HexBinary;

    use hex_literal::hex;

    #[test]
    fn addr_unchecked_works() {
        let a = Addr::unchecked("123");
        let aa = Addr::unchecked(String::from("123"));
        let b = Addr::unchecked("be");
        assert_eq!(a, aa);
        assert_ne!(a, b);
    }

    #[test]
    fn addr_as_str_works() {
        let addr = Addr::unchecked("literal-string");
        assert_eq!(addr.as_str(), "literal-string");
    }

    #[test]
    fn addr_as_bytes_works() {
        let addr = Addr::unchecked("literal-string");
        assert_eq!(
            addr.as_bytes(),
            [108, 105, 116, 101, 114, 97, 108, 45, 115, 116, 114, 105, 110, 103]
        );
    }

    #[test]
    fn addr_implements_display() {
        let addr = Addr::unchecked("cos934gh9034hg04g0h134");
        let embedded = format!("Address: {addr}");
        assert_eq!(embedded, "Address: cos934gh9034hg04g0h134");
        assert_eq!(addr.to_string(), "cos934gh9034hg04g0h134");
    }

    #[test]
    fn addr_implements_as_ref_for_str() {
        let addr = Addr::unchecked("literal-string");
        assert_eq!(addr.as_ref(), "literal-string");
    }

    #[test]
    fn addr_implements_partial_eq_addr_ref() {
        let addr = Addr::unchecked("cos934gh9034hg04g0h134");
        let addr_ref = &addr;
        let addr_ref2 = &addr;

        // `Addr == &Addr`
        assert_eq!(addr, addr_ref);
        // `&Addr == Addr`
        assert_eq!(addr_ref, addr);
        // `&Addr == &Addr`
        assert_eq!(addr_ref, addr_ref2);
    }

    #[test]
    fn addr_implements_into_string() {
        // owned Addr
        let addr = Addr::unchecked("cos934gh9034hg04g0h134");
        let string: String = addr.into();
        assert_eq!(string, "cos934gh9034hg04g0h134");

        // &Addr
        let addr = Addr::unchecked("cos934gh9034hg04g0h134");
        let addr_ref = &addr;
        let string: String = addr_ref.into();
        assert_eq!(string, "cos934gh9034hg04g0h134");
    }

    // Test CanonicalAddr as_slice() for each CanonicalAddr::from input type
    #[test]
    fn canonical_addr_from_slice() {
        // slice
        let bytes: &[u8] = &[0u8, 187, 61, 11, 250, 0];
        let canonical_addr_slice = CanonicalAddr::from(bytes);
        assert_eq!(canonical_addr_slice.as_slice(), &[0u8, 187, 61, 11, 250, 0]);

        // Vector
        let bytes: Vec<u8> = vec![0u8, 187, 61, 11, 250, 0];
        let canonical_addr_vec = CanonicalAddr::from(bytes);
        assert_eq!(canonical_addr_vec.as_slice(), &[0u8, 187, 61, 11, 250, 0]);
    }

    #[test]
    fn canonical_addr_implements_partial_eq_with_binary() {
        let addr = CanonicalAddr::from([1, 2, 3]);
        let bin1 = Binary::from([1, 2, 3]);
        let bin2 = Binary::from([42, 43]);

        assert_eq!(addr, bin1);
        assert_eq!(bin1, addr);
        assert_ne!(addr, bin2);
        assert_ne!(bin2, addr);
    }

    #[test]
    fn canonical_addr_implements_partial_eq_with_hex_binary() {
        let addr = CanonicalAddr::from([1, 2, 3]);
        let bin1 = HexBinary::from([1, 2, 3]);
        let bin2 = HexBinary::from([42, 43]);

        assert_eq!(addr, bin1);
        assert_eq!(bin1, addr);
        assert_ne!(addr, bin2);
        assert_ne!(bin2, addr);
    }

    #[test]
    fn canonical_addr_implements_from_array() {
        let array = [1, 2, 3];
        let addr = CanonicalAddr::from(array);
        assert_eq!(addr.as_slice(), [1, 2, 3]);

        let array_ref = b"foo";
        let addr = CanonicalAddr::from(array_ref);
        assert_eq!(addr.as_slice(), [0x66, 0x6f, 0x6f]);
    }

    #[test]
    fn canonical_addr_implements_from_and_to_vector() {
        // Into<CanonicalAddr> for Vec<u8>
        // This test is a bit pointless because we get Into from the From implementation
        let original = vec![0u8, 187, 61, 11, 250, 0];
        let original_ptr = original.as_ptr();
        let addr: CanonicalAddr = original.into();
        assert_eq!(addr.as_slice(), [0u8, 187, 61, 11, 250, 0]);
        assert_eq!(
            (addr.0).as_slice().as_ptr(),
            original_ptr,
            "must not be copied"
        );

        // From<Vec<u8>> for CanonicalAddr
        let original = vec![0u8, 187, 61, 11, 250, 0];
        let original_ptr = original.as_ptr();
        let addr = CanonicalAddr::from(original);
        assert_eq!(addr.as_slice(), [0u8, 187, 61, 11, 250, 0]);
        assert_eq!(
            (addr.0).as_slice().as_ptr(),
            original_ptr,
            "must not be copied"
        );

        // Into<Vec<u8>> for CanonicalAddr
        // This test is a bit pointless because we get Into from the From implementation
        let original = CanonicalAddr::from(vec![0u8, 187, 61, 11, 250, 0]);
        let original_ptr = (original.0).as_slice().as_ptr();
        let vec: Vec<u8> = original.into();
        assert_eq!(vec.as_slice(), [0u8, 187, 61, 11, 250, 0]);
        assert_eq!(vec.as_ptr(), original_ptr, "must not be copied");

        // From<CanonicalAddr> for Vec<u8>
        let original = CanonicalAddr::from(vec![7u8, 35, 49, 101, 0, 255]);
        let original_ptr = (original.0).as_slice().as_ptr();
        let vec = Vec::<u8>::from(original);
        assert_eq!(vec.as_slice(), [7u8, 35, 49, 101, 0, 255]);
        assert_eq!(vec.as_ptr(), original_ptr, "must not be copied");
    }

    #[test]
    fn canonical_addr_implements_from_and_to_binary() {
        // From<Binary> for CanonicalAddr
        let original = Binary::from([0u8, 187, 61, 11, 250, 0]);
        let original_ptr = original.as_ptr();
        let addr = CanonicalAddr::from(original);
        assert_eq!(addr.as_slice(), [0u8, 187, 61, 11, 250, 0]);
        assert_eq!(
            (addr.0).as_slice().as_ptr(),
            original_ptr,
            "must not be copied"
        );

        // From<CanonicalAddr> for Binary
        let original = CanonicalAddr::from(vec![7u8, 35, 49, 101, 0, 255]);
        let original_ptr = (original.0).as_slice().as_ptr();
        let bin = Binary::from(original);
        assert_eq!(bin.as_slice(), [7u8, 35, 49, 101, 0, 255]);
        assert_eq!(bin.as_ptr(), original_ptr, "must not be copied");
    }

    #[test]
    fn canonical_addr_implements_from_and_to_hex_binary() {
        // From<HexBinary> for CanonicalAddr
        let original = HexBinary::from([0u8, 187, 61, 11, 250, 0]);
        let original_ptr = original.as_ptr();
        let addr = CanonicalAddr::from(original);
        assert_eq!(addr.as_slice(), [0u8, 187, 61, 11, 250, 0]);
        assert_eq!(
            (addr.0).as_slice().as_ptr(),
            original_ptr,
            "must not be copied"
        );

        // From<CanonicalAddr> for HexBinary
        let original = CanonicalAddr::from(vec![7u8, 35, 49, 101, 0, 255]);
        let original_ptr = (original.0).as_slice().as_ptr();
        let bin = HexBinary::from(original);
        assert_eq!(bin.as_slice(), [7u8, 35, 49, 101, 0, 255]);
        assert_eq!(bin.as_ptr(), original_ptr, "must not be copied");
    }

    #[test]
    fn canonical_addr_len() {
        let bytes: &[u8] = &[0u8, 187, 61, 11, 250, 0];
        let canonical_addr = CanonicalAddr::from(bytes);
        assert_eq!(canonical_addr.len(), bytes.len());
    }

    #[test]
    fn canonical_addr_is_empty() {
        let bytes: &[u8] = &[0u8, 187, 61, 11, 250, 0];
        let canonical_addr = CanonicalAddr::from(bytes);
        assert!(!canonical_addr.is_empty());
        let empty_canonical_addr = CanonicalAddr::from(vec![]);
        assert!(empty_canonical_addr.is_empty());
    }

    #[test]
    fn canonical_addr_implements_display() {
        let bytes: &[u8] = &[
            0x12, // two hex digits
            0x03, // small values must be padded to two digits
            0xab, // ensure we get upper case
            0x00, // always test extreme values
            0xff,
        ];
        let address = CanonicalAddr::from(bytes);
        let embedded = format!("Address: {address}");
        assert_eq!(embedded, "Address: 1203AB00FF");
        assert_eq!(address.to_string(), "1203AB00FF");
    }

    #[test]
    fn canonical_addr_implements_deref() {
        // Dereference to [u8]
        let bytes: &[u8] = &[0u8, 187, 61, 11, 250, 0];
        let canonical_addr = CanonicalAddr::from(bytes);
        assert_eq!(*canonical_addr, [0u8, 187, 61, 11, 250, 0]);

        // This checks deref coercions from &CanonicalAddr to &[u8] works
        let bytes: &[u8] = &[0u8, 187, 61, 11, 250, 0];
        let canonical_addr = CanonicalAddr::from(bytes);
        assert_eq!(canonical_addr.len(), 6);
        let canonical_addr_slice: &[u8] = &canonical_addr;
        assert_eq!(canonical_addr_slice, &[0u8, 187, 61, 11, 250, 0]);
    }

    /// Tests that `CanonicalAddr` implements `EQ` and `Hash` correctly and thus
    /// can be used with hash maps and sets.
    #[test]
    fn canonical_addr_implements_hash_eq() {
        let alice = CanonicalAddr::from([0, 187, 61, 11, 250, 0]);
        let bob = CanonicalAddr::from([16, 21, 33, 0, 255, 9]);
        assert_hash_works!(alice, bob);
    }

    // helper to show we can handle Addr and &Addr equally
    fn flexible<'a>(a: impl Into<Cow<'a, Addr>>) -> String {
        a.into().into_owned().to_string()
    }

    #[test]
    fn addr_into_cow() {
        // owned Addr
        let value = "wasmeucn0ur0ncny2308ry";
        let addr = Addr::unchecked(value);

        // pass by ref
        assert_eq!(value, &flexible(&addr));
        // pass by value
        assert_eq!(value, &flexible(addr));
    }

    #[test]
    fn instantiate2_address_impl_works() {
        let checksum1 =
            HexBinary::from_hex("13a1fc994cc6d1c81b746ee0c0ff6f90043875e0bf1d9be6b7d779fc978dc2a5")
                .unwrap();
        let creator1 = CanonicalAddr::from(hex!("9999999999aaaaaaaaaabbbbbbbbbbcccccccccc"));
        let salt1 = hex!("61");
        let salt2 = hex!("aabbccddeeffffeeddbbccddaa66551155aaaabbcc787878789900aabbccddeeffffeeddbbccddaa66551155aaaabbcc787878789900aabbbbcc221100acadae");
        let msg1: &[u8] = b"";
        let msg2: &[u8] = b"{}";
        let msg3: &[u8] = b"{\"some\":123,\"structure\":{\"nested\":[\"ok\",true]}}";

        // No msg
        let expected = CanonicalAddr::from(hex!(
            "5e865d3e45ad3e961f77fd77d46543417ced44d924dc3e079b5415ff6775f847"
        ));
        assert_eq!(
            instantiate2_address_impl(&checksum1, &creator1, &salt1, msg1, 32).unwrap(),
            expected
        );

        // With msg
        let expected = CanonicalAddr::from(hex!(
            "0995499608947a5281e2c7ebd71bdb26a1ad981946dad57f6c4d3ee35de77835"
        ));
        assert_eq!(
            instantiate2_address_impl(&checksum1, &creator1, &salt1, msg2, 32).unwrap(),
            expected
        );

        // Long msg
        let expected = CanonicalAddr::from(hex!(
            "83326e554723b15bac664ceabc8a5887e27003abe9fbd992af8c7bcea4745167"
        ));
        assert_eq!(
            instantiate2_address_impl(&checksum1, &creator1, &salt1, msg3, 32).unwrap(),
            expected
        );

        // Long salt
        let expected = CanonicalAddr::from(hex!(
            "9384c6248c0bb171e306fd7da0993ec1e20eba006452a3a9e078883eb3594564"
        ));
        assert_eq!(
            instantiate2_address_impl(&checksum1, &creator1, &salt2, b"", 32).unwrap(),
            expected
        );

        // Salt too short or too long
        let empty = Vec::<u8>::new();
        assert!(matches!(
            instantiate2_address_impl(&checksum1, &creator1, &empty, b"", 32).unwrap_err(),
            Instantiate2AddressError::InvalidSaltLength
        ));
        let too_long = vec![0x11; 65];
        assert!(matches!(
            instantiate2_address_impl(&checksum1, &creator1, &too_long, b"", 32).unwrap_err(),
            Instantiate2AddressError::InvalidSaltLength
        ));

        // invalid checksum length
        let broken_cs = hex!("13a1fc994cc6d1c81b746ee0c0ff6f90043875e0bf1d9be6b7d779fc978dc2");
        assert!(matches!(
            instantiate2_address_impl(&broken_cs, &creator1, &salt1, b"", 32).unwrap_err(),
            Instantiate2AddressError::InvalidChecksumLength
        ));
        let broken_cs = hex!("");
        assert!(matches!(
            instantiate2_address_impl(&broken_cs, &creator1, &salt1, b"", 32).unwrap_err(),
            Instantiate2AddressError::InvalidChecksumLength
        ));
        let broken_cs = hex!("13a1fc994cc6d1c81b746ee0c0ff6f90043875e0bf1d9be6b7d779fc978dc2aaaa");
        assert!(matches!(
            instantiate2_address_impl(&broken_cs, &creator1, &salt1, b"", 32).unwrap_err(),
            Instantiate2AddressError::InvalidChecksumLength
        ));
    }

    #[test]
    fn instantiate2_address_impl_truncates_address_data_to_first_len_bytes() {
        // test data from above
        let checksum =
            HexBinary::from_hex("13a1fc994cc6d1c81b746ee0c0ff6f90043875e0bf1d9be6b7d779fc978dc2a5")
                .unwrap();
        let creator = CanonicalAddr::from(hex!("9999999999aaaaaaaaaabbbbbbbbbbcccccccccc"));
        let salt = hex!("61");

        let data = [
            (
                32,
                "5e865d3e45ad3e961f77fd77d46543417ced44d924dc3e079b5415ff6775f847",
            ),
            (
                31,
                "5e865d3e45ad3e961f77fd77d46543417ced44d924dc3e079b5415ff6775f8",
            ),
            (
                30,
                "5e865d3e45ad3e961f77fd77d46543417ced44d924dc3e079b5415ff6775",
            ),
            (21, "5e865d3e45ad3e961f77fd77d46543417ced44d924"),
            (20, "5e865d3e45ad3e961f77fd77d46543417ced44d9"),
            (19, "5e865d3e45ad3e961f77fd77d46543417ced44"),
            (16, "5e865d3e45ad3e961f77fd77d4654341"),
            (8, "5e865d3e45ad3e96"),
            (1, "5e"),
        ];

        for (len, expected) in data {
            let expected = CanonicalAddr::from(HexBinary::from_hex(expected).unwrap());
            assert_eq!(
                instantiate2_address_impl(&checksum, &creator, &salt, b"", len).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn instantiate2_address_impl_matches_wasmd_for_len_24() {
        // Manual test data generated with wasmd and bech32 CLI as follows
        // 1. Install https://github.com/cmoog/bech32
        // 2. Check out wasmd main and change to `var ContractAddrLen = 24`
        // 3. Run `make build`
        // 4. Run `./build/wasmd q wasm build-address 1122112211221122112211221122112211221122112211221122112211221122 wasm1xvenxvenxvenxvenxvenxvenxvenxvenkz5vxp aabbaabb | bech32 -d | xxd -p`

        let checksum =
            HexBinary::from_hex("1122112211221122112211221122112211221122112211221122112211221122")
                .unwrap();
        let creator = CanonicalAddr::from(hex!("3333333333333333333333333333333333333333"));
        let salt = hex!("aabbaabb");

        let expected =
            CanonicalAddr::from(hex!["da1aaec9d0ddc75b873079eb1b4f7ddd73a0e3170225fec4"]);
        assert_eq!(
            instantiate2_address_impl(&checksum, &creator, &salt, b"", 24).unwrap(),
            expected
        );
    }

    #[test]
    fn instantiate2_address_impl_works_for_cosmjs_test_vectors() {
        // Test data from https://github.com/cosmos/cosmjs/pull/1253
        const COSMOS_ED25519_TESTS_JSON: &str = "./testdata/instantiate2_addresses.json";

        #[derive(Deserialize, Debug)]
        #[serde(rename_all = "camelCase")]
        #[allow(dead_code)]
        struct In {
            checksum: HexBinary,
            creator: String,
            creator_data: HexBinary,
            salt: HexBinary,
            msg: Option<String>,
        }

        #[derive(Deserialize, Debug)]
        #[serde(rename_all = "camelCase")]
        #[allow(dead_code)]
        struct Intermediate {
            key: HexBinary,
            address_data: HexBinary,
        }

        #[derive(Deserialize, Debug)]
        #[serde(rename_all = "camelCase")]
        #[allow(dead_code)]
        struct Out {
            address: String,
        }

        #[derive(Deserialize, Debug)]
        #[allow(dead_code)]
        struct Row {
            #[serde(rename = "in")]
            input: In,
            intermediate: Intermediate,
            out: Out,
        }

        fn read_tests() -> Vec<Row> {
            use std::fs::File;
            use std::io::BufReader;

            // Open the file in read-only mode with buffer.
            let file = File::open(COSMOS_ED25519_TESTS_JSON).unwrap();
            let reader = BufReader::new(file);

            serde_json::from_reader(reader).unwrap()
        }

        for Row {
            input,
            intermediate,
            out: _,
        } in read_tests()
        {
            let msg = input.msg.map(|msg| msg.into_bytes()).unwrap_or_default();
            let addr = instantiate2_address_impl(
                &input.checksum,
                &input.creator_data.into(),
                &input.salt,
                &msg,
                32,
            )
            .unwrap();
            assert_eq!(addr, intermediate.address_data);
        }
    }

    #[test]
    fn hash_works() {
        // Test case from https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-alpha1/types/address/hash_test.go#L19-L24
        let expected = [
            195, 235, 23, 251, 9, 99, 177, 195, 81, 122, 182, 124, 36, 113, 245, 156, 76, 188, 221,
            83, 181, 192, 227, 82, 100, 177, 161, 133, 240, 160, 5, 25,
        ];
        assert_eq!(hash("1", &[1]), expected);
    }
}
