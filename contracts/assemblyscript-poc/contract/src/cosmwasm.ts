import { getDataPtr } from "./utils";

/**
 * Refers to some heap allocated data in wasm.
 * A pointer to this can be returned over ffi boundaries.
 */
@unmanaged
export class Region {
  offset: u32;
  len: u32;
}

/**
 * Releases ownership of the data without destroying it.
 */
export function releaseOwnership(data: Uint8Array): usize {
  const dataPtr = getDataPtr(data);

  const region: Region = {
    offset: dataPtr,
    len: data.byteLength,
  };
  const regionPtr = changetype<usize>(region);

  // Retain both raw data as well as the Region object referring to it
  __retain(dataPtr);
  __retain(regionPtr);

  return regionPtr;
}
