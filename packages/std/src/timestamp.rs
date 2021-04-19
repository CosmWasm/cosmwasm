/// A point in time in nanosecond precision.
///
/// This type cannot represent any time before the UNIX epoch because both fields are unsigned.
pub struct Timestamp {
    /// Absolute time in seconds since the UNIX epoch (00:00:00 on 1970-01-01 UTC).
    pub seconds: u64,
    /// The fractional part time in nanoseconds since `time` (0 to 999999999).
    pub nanos: u64,
}

impl Timestamp {
    pub fn plus_seconds(&self, addition: u64) -> Timestamp {
        Timestamp {
            seconds: self.seconds + addition,
            nanos: self.nanos,
        }
    }
}
