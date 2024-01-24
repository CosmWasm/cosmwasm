use std::mem::{size_of, MaybeUninit};

use wasmer::{ValueType, WasmPtr};

use crate::conversion::to_u32;
use crate::errors::{
    CommunicationError, CommunicationResult, RegionValidationError, RegionValidationResult,
    VmResult,
};

/****** read/write to wasm memory buffer ****/

/// Describes some data allocated in Wasm's linear memory.
/// A pointer to an instance of this can be returned over FFI boundaries.
///
/// This is the same as `cosmwasm_std::memory::Region`
/// but defined here to allow Wasmer specific implementation.
#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct Region {
    /// The beginning of the region expressed as bytes from the beginning of the linear memory
    pub offset: u32,
    /// The number of bytes available in this region
    pub capacity: u32,
    /// The number of bytes used in this region
    pub length: u32,
}

/// Byte representation of a [Region] struct in Wasm memory.
type RegionBytes = [u8; size_of::<Region>()];

impl Region {
    fn from_wasm_bytes(bytes: RegionBytes) -> Self {
        let offset = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let capacity = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let length = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        Region {
            offset,
            capacity,
            length,
        }
    }

    fn into_wasm_bytes(self) -> RegionBytes {
        let Region {
            offset,
            capacity,
            length,
        } = self;

        let mut bytes = [0u8; 12];
        // wasm is little endian
        bytes[0..4].copy_from_slice(&offset.to_le_bytes());
        bytes[4..8].copy_from_slice(&capacity.to_le_bytes());
        bytes[8..12].copy_from_slice(&length.to_le_bytes());
        bytes
    }
}

unsafe impl ValueType for Region {
    fn zero_padding_bytes(&self, _bytes: &mut [MaybeUninit<u8>]) {
        // The size of Region is exactly 3x4=12 bytes with no padding.
        // The `size_of::<Region>()` test below ensures that.
        // So we do not need to zero any bytes here.
    }
}

// Wasm is little endian, and we want to be able to just reinterpret slices of
// wasm memory as a Region struct, so we only support little endian systems.
// If we ever need to support big endian systems, we can use more fine-grained checks
// in the places where we read/write the Region struct
// (and possibly other interactions between Wasm and host).
#[cfg(target_endian = "big")]
compile_error!("big endian systems are not supported");

/// Expects a (fixed size) Region struct at ptr, which is read. This links to the
/// memory region, which is copied in the second step.
/// Errors if the length of the region exceeds `max_length`.
pub fn read_region(memory: &wasmer::MemoryView, ptr: u32, max_length: usize) -> VmResult<Vec<u8>> {
    let region = get_region(memory, ptr)?;

    if region.length > to_u32(max_length)? {
        return Err(
            CommunicationError::region_length_too_big(region.length as usize, max_length).into(),
        );
    }

    let mut result = vec![0u8; region.length as usize];
    memory
        .read(region.offset as u64, &mut result)
        .map_err(|_err| CommunicationError::region_access_err(region, memory.size().bytes().0))?;
    Ok(result)
}

/// maybe_read_region is like read_region, but gracefully handles null pointer (0) by returning None
/// meant to be used where the argument is optional (like scan)
#[cfg(feature = "iterator")]
pub fn maybe_read_region(
    memory: &wasmer::MemoryView,
    ptr: u32,
    max_length: usize,
) -> VmResult<Option<Vec<u8>>> {
    if ptr == 0 {
        Ok(None)
    } else {
        read_region(memory, ptr, max_length).map(Some)
    }
}

/// A prepared and sufficiently large memory Region is expected at ptr that points to pre-allocated memory.
///
/// Returns number of bytes written on success.
pub fn write_region(memory: &wasmer::MemoryView, ptr: u32, data: &[u8]) -> VmResult<()> {
    let mut region = get_region(memory, ptr)?;

    let region_capacity = region.capacity as usize;
    if data.len() > region_capacity {
        return Err(CommunicationError::region_too_small(region_capacity, data.len()).into());
    }

    memory
        .write(region.offset as u64, data)
        .map_err(|_err| CommunicationError::region_access_err(region, memory.size().bytes().0))?;

    region.length = data.len() as u32;
    set_region(memory, ptr, region)?;

    Ok(())
}

/// Reads in a Region at offset in Wasm memory and returns a copy of it
fn get_region(memory: &wasmer::MemoryView, offset: u32) -> CommunicationResult<Region> {
    let wptr = WasmPtr::<RegionBytes>::new(offset);
    let region = Region::from_wasm_bytes(wptr.deref(memory).read().map_err(|_err| {
        CommunicationError::deref_err(offset, "Could not dereference this pointer to a Region")
    })?);
    validate_region(&region)?;
    Ok(region)
}

/// Performs plausibility checks in the given Region. Regions are always created by the
/// contract and this can be used to detect problems in the standard library of the contract.
fn validate_region(region: &Region) -> RegionValidationResult<()> {
    if region.offset == 0 {
        return Err(RegionValidationError::zero_offset());
    }
    if region.length > region.capacity {
        return Err(RegionValidationError::length_exceeds_capacity(
            region.length,
            region.capacity,
        ));
    }
    if region.capacity > (u32::MAX - region.offset) {
        return Err(RegionValidationError::out_of_range(
            region.offset,
            region.capacity,
        ));
    }
    Ok(())
}

/// Overrides a Region at offset in Wasm memory
fn set_region(memory: &wasmer::MemoryView, offset: u32, data: Region) -> CommunicationResult<()> {
    let wptr = WasmPtr::<RegionBytes>::new(offset);
    wptr.deref(memory)
        .write(data.into_wasm_bytes())
        .map_err(|_err| {
            CommunicationError::deref_err(offset, "Could not dereference this pointer to a Region")
        })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::mem;

    use super::*;

    #[test]
    fn region_has_known_size() {
        // 3x4 bytes with no padding
        assert_eq!(mem::size_of::<Region>(), 12);
    }

    #[test]
    fn validate_region_passes_for_valid_region() {
        // empty
        let region = Region {
            offset: 23,
            capacity: 500,
            length: 0,
        };
        validate_region(&region).unwrap();

        // half full
        let region = Region {
            offset: 23,
            capacity: 500,
            length: 250,
        };
        validate_region(&region).unwrap();

        // full
        let region = Region {
            offset: 23,
            capacity: 500,
            length: 500,
        };
        validate_region(&region).unwrap();

        // at end of linear memory (1)
        let region = Region {
            offset: u32::MAX,
            capacity: 0,
            length: 0,
        };
        validate_region(&region).unwrap();

        // at end of linear memory (2)
        let region = Region {
            offset: 1,
            capacity: u32::MAX - 1,
            length: 0,
        };
        validate_region(&region).unwrap();
    }

    #[test]
    fn validate_region_fails_for_zero_offset() {
        let region = Region {
            offset: 0,
            capacity: 500,
            length: 250,
        };
        let result = validate_region(&region);
        match result.unwrap_err() {
            RegionValidationError::ZeroOffset { .. } => {}
            e => panic!("Got unexpected error: {e:?}"),
        }
    }

    #[test]
    fn validate_region_fails_for_length_exceeding_capacity() {
        let region = Region {
            offset: 23,
            capacity: 500,
            length: 501,
        };
        let result = validate_region(&region);
        match result.unwrap_err() {
            RegionValidationError::LengthExceedsCapacity {
                length, capacity, ..
            } => {
                assert_eq!(length, 501);
                assert_eq!(capacity, 500);
            }
            e => panic!("Got unexpected error: {e:?}"),
        }
    }

    #[test]
    fn validate_region_fails_when_exceeding_address_space() {
        let region = Region {
            offset: 23,
            capacity: u32::MAX,
            length: 501,
        };
        let result = validate_region(&region);
        match result.unwrap_err() {
            RegionValidationError::OutOfRange {
                offset, capacity, ..
            } => {
                assert_eq!(offset, 23);
                assert_eq!(capacity, u32::MAX);
            }
            e => panic!("Got unexpected error: {e:?}"),
        }

        let region = Region {
            offset: u32::MAX,
            capacity: 1,
            length: 0,
        };
        let result = validate_region(&region);
        match result.unwrap_err() {
            RegionValidationError::OutOfRange {
                offset, capacity, ..
            } => {
                assert_eq!(offset, u32::MAX);
                assert_eq!(capacity, 1);
            }
            e => panic!("Got unexpected error: {e:?}"),
        }
    }
}
