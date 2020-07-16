mod communication_error;
mod ffi_error;
mod region_validation_error;
mod vm_error;

pub use communication_error::CommunicationError;
pub use ffi_error::FfiError;
pub use region_validation_error::RegionValidationError;
pub use vm_error::VmError;

#[derive(Copy, Clone, Debug)]
pub struct GasInfo {
    /// The gas cost of a computation that was executed already but not yet charged
    pub cost: u64,
    /// Gas that was used and charged externally. This is needed to
    /// adjust the VM's gas limit but does not affect the gas usage.
    pub externally_used: u64,
}

impl GasInfo {
    pub fn with_cost(amount: u64) -> Self {
        GasInfo {
            cost: amount,
            externally_used: 0,
        }
    }

    pub fn with_externally_used(amount: u64) -> Self {
        GasInfo {
            cost: 0,
            externally_used: amount,
        }
    }
}

/// A return element and the gas cost of this FFI call
pub type FfiSuccess<T> = (T, GasInfo);

pub type CommunicationResult<T> = core::result::Result<T, CommunicationError>;
pub type FfiResult<T> = core::result::Result<FfiSuccess<T>, FfiError>;
pub type RegionValidationResult<T> = core::result::Result<T, RegionValidationError>;
pub type VmResult<T> = core::result::Result<T, VmError>;
