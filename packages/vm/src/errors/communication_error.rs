use snafu::Snafu;
use std::fmt::Debug;

/// An error in the communcation between contract and host. Those happen around imports and exports.
#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum CommunicationError {
    #[snafu(display(
        "The Wasm memory address {} provided by the contract could not be dereferenced: {}",
        offset,
        msg
    ))]
    DerefErr {
        /// the position in a Wasm linear memory
        offset: u32,
        msg: String,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Got an invalid value for iteration order: {}", value))]
    InvalidOrder {
        value: i32,
        backtrace: snafu::Backtrace,
    },
    /// Whenever UTF-8 bytes cannot be decoded into a unicode string, e.g. in String::from_utf8 or str::from_utf8.
    #[snafu(display("Cannot decode UTF8 bytes into string: {}", msg))]
    InvalidUtf8 {
        msg: String,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Region length too big. Got {}, limit {}", length, max_length))]
    // Note: this only checks length, not capacity
    RegionLengthTooBig {
        length: usize,
        max_length: usize,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display(
        "Region length exceeds capacity. Length {}, capacity {}",
        length,
        capacity
    ))]
    RegionLengthExceedsCapacity {
        length: u32,
        capacity: u32,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display(
        "Region exceeds address space. Offset {}, capacity {}",
        offset,
        capacity
    ))]
    RegionOutOfRange {
        offset: u32,
        capacity: u32,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Region too small. Got {}, required {}", size, required))]
    RegionTooSmall {
        size: usize,
        required: usize,
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Got a zero Wasm address"))]
    ZeroAddress { backtrace: snafu::Backtrace },
}

impl CommunicationError {
    pub(crate) fn deref_err<S: Into<String>>(offset: u32, msg: S) -> Self {
        DerefErr {
            offset,
            msg: msg.into(),
        }
        .build()
    }

    #[allow(dead_code)]
    pub(crate) fn invalid_order(value: i32) -> Self {
        InvalidOrder { value }.build()
    }

    #[allow(dead_code)]
    pub(crate) fn invalid_utf8<S: ToString>(msg: S) -> Self {
        InvalidUtf8 {
            msg: msg.to_string(),
        }
        .build()
    }

    pub(crate) fn region_length_too_big(length: usize, max_length: usize) -> Self {
        RegionLengthTooBig { length, max_length }.build()
    }

    pub(crate) fn region_length_exceeds_capacity(length: u32, capacity: u32) -> Self {
        RegionLengthExceedsCapacity { length, capacity }.build()
    }

    pub(crate) fn region_out_of_range(offset: u32, capacity: u32) -> Self {
        RegionOutOfRange { offset, capacity }.build()
    }

    pub(crate) fn region_too_small(size: usize, required: usize) -> Self {
        RegionTooSmall { size, required }.build()
    }

    pub(crate) fn zero_address() -> Self {
        ZeroAddress {}.build()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // constructors

    #[test]
    fn communication_error_deref_err() {
        let error = CommunicationError::deref_err(345, "broken stuff");
        match error {
            CommunicationError::DerefErr { offset, msg, .. } => {
                assert_eq!(offset, 345);
                assert_eq!(msg, "broken stuff");
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn communication_error_invalid_order() {
        let error = CommunicationError::invalid_order(-745);
        match error {
            CommunicationError::InvalidOrder { value, .. } => assert_eq!(value, -745),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn communication_error_invalid_utf8() {
        let error = CommunicationError::invalid_utf8("broken");
        match error {
            CommunicationError::InvalidUtf8 { msg, .. } => assert_eq!(msg, "broken"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn communication_error_region_length_too_big_works() {
        let error = CommunicationError::region_length_too_big(50, 20);
        match error {
            CommunicationError::RegionLengthTooBig {
                length, max_length, ..
            } => {
                assert_eq!(length, 50);
                assert_eq!(max_length, 20);
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn communication_error_region_length_exceeds_capacity_works() {
        let error = CommunicationError::region_length_exceeds_capacity(50, 20);
        match error {
            CommunicationError::RegionLengthExceedsCapacity {
                length, capacity, ..
            } => {
                assert_eq!(length, 50);
                assert_eq!(capacity, 20);
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn communication_error_region_out_of_range_works() {
        let error = CommunicationError::region_out_of_range(u32::MAX, 1);
        match error {
            CommunicationError::RegionOutOfRange {
                offset, capacity, ..
            } => {
                assert_eq!(offset, u32::MAX);
                assert_eq!(capacity, 1);
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn communication_error_region_too_small_works() {
        let error = CommunicationError::region_too_small(12, 33);
        match error {
            CommunicationError::RegionTooSmall { size, required, .. } => {
                assert_eq!(size, 12);
                assert_eq!(required, 33);
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn communication_error_zero_address() {
        let error = CommunicationError::zero_address();
        match error {
            CommunicationError::ZeroAddress { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }
}
