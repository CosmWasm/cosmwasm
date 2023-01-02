/// Never can never be instantiated. This can be used in places
/// where we want to ensure that no error is returned, such as
/// the `ibc_packet_receive` entry point.
///
/// In contrast to `Empty`, this does not have a JSON schema
/// and cannot be used for message and query types.
///
/// Once the ! type is stable, this is not needed anymore.
/// See <https://github.com/rust-lang/rust/issues/35121>.
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
