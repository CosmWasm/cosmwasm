/// Never can never be instantiated. This can be used in places
/// where we want to ensure that no error is returned, such as
/// the `ibc_packet_receive` entry point.
///
/// Once the ! type is stable, this is not needed anymore.
/// See <https://github.com/rust-lang/rust/issues/35121>.
pub enum Never {}

impl core::fmt::Debug for Never {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // This is unreachable because no instance of Never can exist
        unreachable!()
    }
}

impl core::fmt::Display for Never {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // This is unreachable because no instance of Never can exist
        unreachable!()
    }
}
