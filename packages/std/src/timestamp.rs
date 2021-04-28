use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::math::Uint64;

/// A point in time in nanosecond precision.
///
/// This type can represent times from 1970-01-01T00:00:00Z to 2554-07-21T23:34:33Z.
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

    pub const fn plus_seconds(&self, addition: u64) -> Timestamp {
        let nanos = Uint64::new(self.0.u64() + addition * 1_000_000_000);
        Timestamp(nanos)
    }

    pub const fn plus_nanos(&self, addition: u64) -> Timestamp {
        let nanos = Uint64::new(self.0.u64() + addition);
        Timestamp(nanos)
    }

    /// Returns nanoseconds since epoch
    pub fn nanos(&self) -> u64 {
        self.0.u64()
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

    #[test]
    fn timestamp_plus_seconds() {
        let sum = Timestamp::from_nanos(123).plus_seconds(42);
        assert_eq!(sum.0.u64(), 42_000_000_123);
        let sum = Timestamp::from_nanos(123).plus_seconds(0);
        assert_eq!(sum.0.u64(), 123);
    }

    #[test]
    fn timestamp_plus_nanos() {
        let sum = Timestamp::from_nanos(123).plus_nanos(3);
        assert_eq!(sum.0.u64(), 126);
        let sum = Timestamp::from_nanos(123).plus_nanos(0);
        assert_eq!(sum.0.u64(), 123);
    }

    #[test]
    fn timestamp_nanos() {
        let sum = Timestamp::from_nanos(123);
        assert_eq!(sum.nanos(), 123);
        let sum = Timestamp::from_nanos(0);
        assert_eq!(sum.nanos(), 0);
    }
}
