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
    pub fn plus_seconds(&self, addition: u64) -> Timestamp {
        let nanos = self.0 + Uint64::from(addition);
        Timestamp(nanos)
    }
}

impl From<Uint64> for Timestamp {
    fn from(original: Uint64) -> Self {
        Self(original)
    }
}

impl From<u64> for Timestamp {
    fn from(original: u64) -> Self {
        Self(original.into())
    }
}

impl From<Timestamp> for Uint64 {
    fn from(original: Timestamp) -> Uint64 {
        original.0
    }
}

impl From<Timestamp> for u64 {
    fn from(original: Timestamp) -> u64 {
        original.0.u64()
    }
}
