use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::math::Uint64;

/// A point in time in nanosecond precision.
///
/// This type cannot represent any time before the UNIX epoch because both fields are unsigned.
#[derive(
    Serialize, Deserialize, Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, JsonSchema,
)]
pub struct Timestamp(Uint64);

impl Timestamp {
    /// Creates a timestamp from nanoseconds since epoch
    pub const fn from_nanos(nanos_since_epoch: u64) -> Self {
        Timestamp(Uint64::new(nanos_since_epoch))
    }

    /// Creates a timestamp from seconds since epoch
    pub const fn from_seconds(seconds_since_epoch: u64) -> Self {
        Timestamp(Uint64::new(seconds_since_epoch * 1_000_000_000))
    }

    pub fn plus_seconds(&self, addition: u64) -> Timestamp {
        let nanos = self.0 + Uint64::from(addition);
        Timestamp(nanos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_from_nanos() {
        let t = Timestamp::from_nanos(123);
        assert_eq!(t.0.u64(), 123);
        let t = Timestamp::from_nanos(0);
        assert_eq!(t.0.u64(), 0);
    }

    #[test]
    fn timestamp_from_seconds() {
        let t = Timestamp::from_seconds(123);
        assert_eq!(t.0.u64(), 123_000_000_000);
        let t = Timestamp::from_seconds(0);
        assert_eq!(t.0.u64(), 0);
    }
}
