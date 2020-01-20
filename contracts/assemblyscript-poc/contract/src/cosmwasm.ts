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
 * allocate reserves the given number of bytes in wasm memory and returns a pointer
 * to a slice defining this data. This space is managed by the calling process
 * and should be accompanied by a corresponding deallocate
 */
export function allocate(size: usize): usize {
  const dataPtr = __alloc(size, idof<ArrayBuffer>());
  __retain(dataPtr);

  const region: Region = {
    offset: dataPtr,
    len: size,
  };
  const regionPtr = changetype<usize>(region);
  __retain(regionPtr);
  return regionPtr;
}

/**
 * Expects a pointer to a Region created with allocate.
 * It will free both the Region and the memory referenced by the Region.
 */
export function deallocate(regionPtr: usize): void {
  const dataPtr = changetype<Region>(regionPtr).offset;
  __release(regionPtr); // release Region
  __release(dataPtr); // release ArrayBuffer
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

/**
 * Takes ownership of the data at the given pointer
 */
export function takeOwnership(regionPtr: usize): Uint8Array {
  const region = changetype<Region>(regionPtr);

  const out = new Uint8Array(region.len);
  // TODO: is this copy really necessary?
  memory.copy(getDataPtr(out), region.offset, region.len);
  deallocate(regionPtr);

  return out;
}
