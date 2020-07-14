mod communication_error;
mod ffi_error;
mod region_validation_error;
mod vm_error;

pub use communication_error::CommunicationError;
pub use ffi_error::FfiError;
pub use region_validation_error::RegionValidationError;
pub use vm_error::VmError;

pub type CommunicationResult<T> = core::result::Result<T, CommunicationError>;
pub type FfiResult<T> = core::result::Result<T, FfiError>;
pub type RegionValidationResult<T> = core::result::Result<T, RegionValidationError>;
pub type VmResult<T> = core::result::Result<T, VmError>;
