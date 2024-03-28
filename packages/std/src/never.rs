/// `Never` represents a type that can never be instantiated.
/// It's primarily used in contexts where it's important to signify that an error cannot occur.
/// For example, it can be used in the `ibc_packet_receive` entry point to indicate
/// that no error is expected to be returned.
///
/// Unlike the `Empty` type, `Never` is distinct in that it doesn't have an associated JSON schema.
/// Consequently, it's not suitable for use in message or query types where JSON representation is required.
///
/// The existence of `Never` is a temporary necessity. It is anticipated to be deprecated
/// once the Rust `!` type, which represents a never type in the Rust language, becomes stable.
/// The stabilization of the `!` type is an ongoing discussion, which you can follow here:
/// <https://github.com/rust-lang/rust/issues/35121>.
/// ```
/// use cosmwasm_std::Never;
///
/// pub fn safe_unwrap<T>(res: Result<T, Never>) -> T {
///     match res {
///         Ok(value) => value,
///         Err(err) => match err {},
///     }
/// }
///
/// let res: Result<i32, Never> = Ok(5);
/// assert_eq!(safe_unwrap(res), 5);
/// ```
pub enum Never {}

// The Debug implementation is needed to allow the use of `Result::unwrap`.
impl core::fmt::Debug for Never {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Unreachable because no instance of Never can exist
        match *self {}
    }
}

// The Display implementation is needed to fulfill the ToString requirement of
// entry point errors: `Result<IbcReceiveResponse<C>, E>` with `E: ToString`.
impl core::fmt::Display for Never {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Unreachable because no instance of Never can exist
        match *self {}
    }
}
